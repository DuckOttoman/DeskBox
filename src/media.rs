use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}, process::{Command, Stdio}, sync::Mutex};
static PREPARATION: Mutex<()> = Mutex::new(());
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaBackground { pub kind: String, pub original: String, pub eco: Option<String>, pub poster: String, pub mode: String }
impl MediaBackground {
    pub fn validate_mode(&self, mode: &str, managed: &Path) -> Result<(), String> {
        if self.kind != "video" || !matches!(mode,"eco"|"original") { return Err("Invalid video mode".into()); }
        let path = if mode == "eco" { self.eco.as_deref().ok_or("Eco variant unavailable")? } else { &self.original };
        for value in [path, self.poster.as_str()] { validate_managed_file(Path::new(value), managed)?; }
        Ok(())
    }
}
fn validate_managed_file(path: &Path, managed: &Path) -> Result<(), String> {
    let root=managed.canonicalize().map_err(|e|e.to_string())?;
    let actual=path.canonicalize().map_err(|e|e.to_string())?;
    if !actual.starts_with(root) || !actual.is_file() || fs::metadata(&actual).map_err(|e|e.to_string())?.len()==0 {return Err("Invalid managed media file".into());} Ok(())
}
pub fn is_animated(source: &Path)->bool {matches!(source.extension().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase().as_str(),"gif"|"mp4"|"webm"|"m4v"|"mov")}
fn run(ffmpeg: &Path, args: &[&str], input: &Path, output: Option<&Path>) -> Result<(),String> {
    let mut cmd=Command::new(ffmpeg);
    cmd.args(["-hide_banner","-v","error","-nostdin","-threads","2","-i"]).arg(input).args(args);
    if let Some(output)=output { cmd.arg(output); }
    cmd.stdin(Stdio::null());
    #[cfg(windows)] { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }
    let result=cmd.output().map_err(|e|format!("FFmpeg unavailable: {e}"))?;
    if !result.status.success() {return Err(format!("Media preparation failed: {}",String::from_utf8_lossy(&result.stderr).chars().take(1800).collect::<String>()));} Ok(())
}
pub fn prepare_background(source: &Path, managed_dir: &Path, ffmpeg: &Path)->Result<MediaBackground,String>{
    if !is_animated(source) || !source.is_file() {return Err("Unsupported or missing media source".into());}
    let _serial=PREPARATION.lock().map_err(|_|"Media preparation lock poisoned")?;
    fs::create_dir_all(managed_dir).map_err(|e|e.to_string())?;
    let id=uuid::Uuid::new_v4().to_string(); let stage=managed_dir.join(format!(".staging-{id}")); let destination=managed_dir.join(&id);
    fs::create_dir(&stage).map_err(|e|e.to_string())?;
    let result=(||{
        let ext=source.extension().unwrap().to_string_lossy().to_ascii_lowercase();
        let original=stage.join(format!("original.{ext}"));let poster=stage.join("poster.png");let eco=stage.join("eco.mp4");
        fs::copy(source,&original).map_err(|e|e.to_string())?;
        run(ffmpeg,&["-map","0:v:0","-frames:v","1","-an","-filter_threads","2","-vf","scale='min(854,iw)':'min(480,ih)':force_original_aspect_ratio=decrease","-threads","2","-y"],&original,Some(&poster))?;
        if ext!="gif" {
            run(ffmpeg,&["-map","0:v:0","-an","-sn","-dn","-filter_threads","2","-vf","scale='min(854,iw)':'min(480,ih)':force_original_aspect_ratio=decrease:force_divisible_by=2,fps=fps='if(isnan(source_fps),30,min(30,max(24,source_fps)))'","-c:v","libx264","-preset","veryfast","-crf","25","-pix_fmt","yuv420p","-threads","2","-movflags","+faststart","-y"],&original,Some(&eco))?;
            run(ffmpeg,&["-map","0:v:0","-frames:v","1","-f","null","-"],&eco,None)?;
            validate_managed_file(&eco,&stage)?;
        }
        validate_managed_file(&poster,&stage)?;validate_managed_file(&original,&stage)?;
        fs::rename(&stage,&destination).map_err(|e|e.to_string())?;
        let path=|name:&str|destination.join(name).to_string_lossy().into_owned();
        Ok(MediaBackground{kind:if ext=="gif"{"gif"}else{"video"}.into(),original:path(&format!("original.{ext}")),eco:(ext!="gif").then(||path("eco.mp4")),poster:path("poster.png"),mode:if ext=="gif"{"original"}else{"eco"}.into()})
    })();
    if result.is_err(){let _=fs::remove_dir_all(&stage);}result
}
pub fn ffmpeg_path(diagnostic: bool)->Result<PathBuf,String>{
    if diagnostic {if let Some(path)=std::env::var_os("DESKBOX_FFMPEG"){return Ok(PathBuf::from(path));}}
    Ok(std::env::current_exe().map_err(|e|e.to_string())?.parent().ok_or("Executable directory unavailable")?.join("media-tools").join("ffmpeg.exe"))
}
