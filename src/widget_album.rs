use crate::{app::AppState,scanner};
use std::{fs,path::{Path,PathBuf}};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use serde_json::{json,Value};
struct PendingFiles { files:Vec<PathBuf>, committed:bool }
impl Drop for PendingFiles {fn drop(&mut self){if !self.committed {for path in &self.files {let _=fs::remove_file(path);}}}}

fn checked_directory(path:&Path)->Result<PathBuf,String>{
    // Resolve existing links before checking Desktop; do not create on Desktop.
    let ancestor=path.ancestors().find(|p|p.exists()).ok_or("相册目录不可用")?;
    let resolved=fs::canonicalize(ancestor).map_err(|e|e.to_string())?;
    for desktop in scanner::desktop_paths()? {
        if let Ok(desktop)=fs::canonicalize(desktop){if resolved.starts_with(desktop){return Err("相册不能存储在桌面，请通过菜单选择其他目录".into());}}
    }
    fs::create_dir_all(path).map_err(|e|format!("相册目录不可写，请更换目录：{e}"))?;
    let result=fs::canonicalize(path).map_err(|e|e.to_string())?;
    Ok(result)
}
fn album_data(data:&AppState,id:&str)->Result<serde_json::Map<String,Value>,String>{
    let s=data.snapshot()?;let w=s.config.widgets.iter().find(|w|w.id==id&&w.kind=="album").ok_or("相册不存在")?;
    Ok(w.style.get("widgetData").and_then(Value::as_object).cloned().unwrap_or_default())
}
fn owned(path:&str,directory:&Path)->bool{
    let p=Path::new(path);p.parent()==Some(directory)&&p.extension().is_some_and(|v|v=="png")&&p.file_stem().and_then(|v|v.to_str()).is_some_and(|v|uuid::Uuid::parse_str(v).is_ok())
}
pub fn action(app:&tauri::AppHandle,data:&AppState,id:&str,action:&str,index:Option<usize>,sources:Option<Vec<String>>,directory:Option<String>)->Result<(),String>{
    uuid::Uuid::parse_str(id).map_err(|_|"Invalid widget ID")?;
    let mut value=album_data(data,id)?;
    let previous_photos=value.get("photos").cloned();
    let previous_directory=value.get("albumDirectory").cloned();
    let default=std::env::current_exe().map_err(|e|e.to_string())?.parent().ok_or("程序目录不可用")?.join("Albums");
    let old=value.get("albumDirectory").and_then(Value::as_str).map(PathBuf::from).unwrap_or(default);
    let mut photos=value.get("photos").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut added=PendingFiles{files:vec![],committed:false};
    let root=if action=="directory" {
        let picked=if data.smoke {directory.map(PathBuf::from)}else{app.dialog().file().set_title("选择桌面以外的相册存储目录").blocking_pick_folder().map(|p|p.into_path()).transpose().map_err(|e|e.to_string())?};
        let Some(picked)=picked else{return Ok(());};checked_directory(&picked)?
    }else{checked_directory(&old)?};
    let folder=checked_directory(&root.join(id))?;
    match action {
        "add"=>{
            let selected=if data.smoke {sources.unwrap_or_default().into_iter().map(PathBuf::from).collect()} else {
                app.dialog().file().set_title("添加相册图片").add_filter("图片",&["png","jpg","jpeg","webp","bmp"]).blocking_pick_files().unwrap_or_default().into_iter().map(|p|p.into_path()).collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?
            };
            if photos.len()+selected.len()>200{return Err("每个相册最多200张图片".into());}
            for source in selected {
                let reader=image::ImageReader::open(&source).map_err(|e|e.to_string())?.with_guessed_format().map_err(|e|e.to_string())?;
                let picture=reader.decode().map_err(|e|format!("无法读取图片：{e}"))?;
                let dest=folder.join(format!("{}.png",uuid::Uuid::new_v4()));
                // Bounded display copies; source originals remain unchanged.
                picture.thumbnail(2560,2560).save(&dest).map_err(|e|e.to_string())?;
                added.files.push(dest.clone());photos.push(json!({"path":dest.to_string_lossy(),"name":source.file_name().unwrap_or_default().to_string_lossy()}));
            }
        },
        "remove"=>{let i=index.ok_or("请选择图片")?;if i>=photos.len(){return Err("图片已不存在".into());}photos.remove(i);},
        "up"=>{let i=index.ok_or("请选择图片")?;if i==0||i>=photos.len(){return Ok(());}photos.swap(i,i-1);},
        "directory"=>{
            if root!=old {
                for photo in &mut photos {
                    let source=photo.get("path").and_then(Value::as_str).ok_or("Invalid photo")?;
                    if !owned(source,&old.join(id)){return Err("图片路径不属于此相册".into());}
                    let dest=folder.join(format!("{}.png",uuid::Uuid::new_v4()));
                    fs::copy(source,&dest).map_err(|e|e.to_string())?;photo["path"]=dest.to_string_lossy().into_owned().into();added.files.push(dest);
                }
            }
        },_=>return Err("Unknown album action".into())
    }
    for p in &photos {let path=p.get("path").and_then(Value::as_str).ok_or("Invalid photo")?;if !owned(path,&folder){return Err("图片路径无效".into());}app.asset_protocol_scope().allow_file(path).map_err(|e|e.to_string())?;}
    value.insert("albumDirectory".into(),root.to_string_lossy().into_owned().into());value.insert("photos".into(),photos.into());
    let result=data.mutate(|c|{
        let w=c.widgets.iter_mut().find(|w|w.id==id&&w.kind=="album").ok_or("相册已被删除")?;
        let mut current=w.style.get("widgetData").and_then(Value::as_object).cloned().unwrap_or_default();
        if current.get("photos")!=previous_photos.as_ref()||current.get("albumDirectory")!=previous_directory.as_ref(){return Err("相册在操作期间已改变，请重试".into());}
        current.insert("photos".into(),value.remove("photos").unwrap());current.insert("albumDirectory".into(),value.remove("albumDirectory").unwrap());
        w.style.insert("widgetData".into(),current.into());Ok(())
    });
    result?;added.committed=true;data.publish(app)
}
pub fn allow_saved(app:&tauri::AppHandle,data:&AppState){
    if let Ok(s)=data.snapshot(){for w in s.config.widgets.iter().filter(|w|w.kind=="album"){
        let Some(value)=w.style.get("widgetData")else{continue;};
        let Some(root)=value.get("albumDirectory").and_then(Value::as_str)else{continue;};
        let Some(photos)=value.get("photos").and_then(Value::as_array)else{continue;};
        for photo in photos {if let Some(p)=photo.get("path").and_then(Value::as_str){if owned(p,&Path::new(root).join(&w.id)){let _=app.asset_protocol_scope().allow_file(p);}}}
    }}
}
