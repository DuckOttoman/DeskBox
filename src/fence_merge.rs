//! Persistent mutations only. Pointer previews never call these functions.
use crate::model::*;
use std::collections::HashSet;

pub fn visible_fence(c: &Config, f: &Fence) -> bool {
    !f.hidden && c.fence_groups.iter().find(|g|g.members.contains(&f.id)).is_none_or(|g|g.active==f.id)
}
pub fn geometry(c: &Config, f: &Fence) -> [f64;4] {
    c.fence_groups.iter().find(|g|g.members.contains(&f.id)).map(|g|[g.x,g.y,f.w,f.h]).unwrap_or([f.x,f.y,f.w,f.h])
}
pub fn container_monitor<'a>(c: &'a Config, f: &'a Fence) -> &'a str {
    c.fence_groups.iter().find(|g|g.members.contains(&f.id)).map(|g|g.monitor.as_str()).unwrap_or(&f.monitor)
}
fn pair(c: &Config, source: &str, target: &str) -> Result<(Fence,Fence),String> {
    if source==target {return Err("Cannot merge a fence into itself".into());}
    let a=c.fences.iter().find(|f|f.id==source).ok_or("Source fence does not exist")?;
    let b=c.fences.iter().find(|f|f.id==target).ok_or("Target fence does not exist")?;
    if !visible_fence(c,a)||!visible_fence(c,b) {return Err("Fence is not currently visible".into());}
    if c.fence_groups.iter().any(|g|g.members.contains(&a.id)&&g.members.contains(&b.id)){return Err("Fences already share a group".into());}
    // Monitor IDs may be stale after disconnect; the command adapter checks the actual WebView.
    Ok((a.clone(),b.clone()))
}
pub fn remove_member(c: &mut Config, member: &str) {
    for g in &mut c.fence_groups {
        g.members.retain(|id|id!=member);
        if g.active==member {g.active=g.members.first().cloned().unwrap_or_default();}
    }
    for g in c.fence_groups.iter().filter(|g|g.members.len()==1) {
        if let Some(f)=c.fences.iter_mut().find(|f|f.id==g.members[0]) {f.x=g.x;f.y=g.y;f.monitor=g.monitor.clone();}
    }
    c.fence_groups.retain(|g|g.members.len()>1);
}
pub fn group(c: &mut Config, source: &str, target: &str) -> Result<String,String> {
    let (_,b)=pair(c,source,target)?;
    let p=geometry(c,&b);
    remove_member(c,source);
    if let Some(g)=c.fence_groups.iter_mut().find(|g|g.members.contains(&b.id)) {
        g.members.push(source.into());return Ok(g.id.clone());
    }
    let id=uuid::Uuid::new_v4().to_string();
    c.fence_groups.push(FenceGroup{id:id.clone(),members:vec![target.into(),source.into()],active:target.into(),x:p[0],y:p[1],monitor:b.monitor});Ok(id)
}
pub fn select(c: &mut Config, group: &str, member: &str) -> Result<(),String> {
    let g=c.fence_groups.iter_mut().find(|g|g.id==group).ok_or("Group does not exist")?;
    if !g.members.iter().any(|id|id==member)||!c.fences.iter().any(|f|f.id==member&&!f.hidden){return Err("Member does not exist or is hidden".into());}
    g.active=member.into();Ok(())
}
pub fn detach(c: &mut Config, member: &str, point: [f64;2]) -> Result<(),String> {
    if !point.iter().all(|x|x.is_finite()){return Err("Invalid position".into());}
    if !c.fences.iter().any(|f|f.id==member){return Err("Fence does not exist".into());}
    let monitor=c.fence_groups.iter().find(|g|g.members.iter().any(|id|id==member)).map(|g|g.monitor.clone());
    remove_member(c,member);
    let f=c.fences.iter_mut().find(|f|f.id==member).unwrap();f.x=point[0];f.y=point[1];if let Some(m)=monitor{f.monitor=m;}Ok(())
}
pub fn merge_contents(c: &mut Config, source: &str, target: &str) -> Result<String,String> {
    let (a,b)=pair(c,source,target)?;
    let id=uuid::Uuid::new_v4().to_string();
    let record=MergeRestore{id:id.clone(),source:a.clone(),target:target.into(),target_items:b.items.clone(),target_title:b.title,created_at:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64};
    remove_member(c,source);c.fences.retain(|f|f.id!=source);
    let b=c.fences.iter_mut().find(|f|f.id==target).unwrap();let mut seen=HashSet::new();
    b.items=b.items.iter().chain(a.items.iter()).filter(|id|seen.insert((*id).clone())).cloned().collect();
    c.merge_restores.push(record);Ok(id)
}
fn overlap(a:[f64;4],b:[f64;4])->f64 {
    (a[0]+a[2]).min(b[0]+b[2]).sub(a[0].max(b[0])).max(0.) * (a[1]+a[3]).min(b[1]+b[3]).sub(a[1].max(b[1])).max(0.)
}
use std::ops::Sub;
fn clamp(p:[f64;2],size:[f64;2],work:[f64;4])->[f64;2] {
    [p[0].clamp(work[0],(work[0]+work[2]-size[0]).max(work[0])),p[1].clamp(work[1],(work[1]+work[3]-size[1]).max(work[1]))]
}
fn candidates(origin:[f64;2],size:[f64;2],work:[f64;4],occupied:&[[f64;4]])->Vec<[f64;2]> {
    let mut xs=vec![origin[0],work[0],work[0]+work[2]-size[0]];
    let mut ys=vec![origin[1],work[1],work[1]+work[3]-size[1]];
    for r in occupied {xs.extend([r[0]-size[0]-12.,r[0]+r[2]+12.]);ys.extend([r[1]-size[1]-12.,r[1]+r[3]+12.]);}
    xs.into_iter().flat_map(|x|ys.iter().map(move|y|[x,*y])).map(|p|clamp(p,size,work)).collect()
}
fn best(origin:[f64;2],size:[f64;2],work:[f64;4],occupied:&[[f64;4]])->[f64;2] {
    candidates(origin,size,work,occupied).into_iter().min_by(|a,b| {
        let score=|p:&[f64;2]|(occupied.iter().map(|r|overlap([p[0],p[1],size[0],size[1]],*r)).sum::<f64>(),(p[0]-origin[0]).hypot(p[1]-origin[1]));
        let x=score(a);let y=score(b);x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1))
    }).unwrap_or([work[0],work[1]])
}
pub fn restore_on_display(c: &mut Config, record: &str, valid: &HashSet<String>, work:[f64;4], display_name:&str, monitors:&[Monitor]) -> Result<(),String> {
    if !monitors.iter().any(|m|m.name==display_name) {return Err("Display does not exist".into());}
    restore_internal(c,record,valid,work,Some((display_name,monitors)))
}
pub fn restore(c: &mut Config, record: &str, valid: &HashSet<String>, work:[f64;4]) -> Result<(),String> {
    restore_internal(c,record,valid,work,None)
}
fn restore_internal(c: &mut Config, record: &str, valid: &HashSet<String>, work:[f64;4], display:Option<(&str,&[Monitor])>) -> Result<(),String> {
    if !work.iter().all(|x|x.is_finite())||work[2]<=0.||work[3]<=0. {return Err("Invalid work area".into());}
    let index=c.merge_restores.iter().position(|r|r.id==record).ok_or("Restore point does not exist")?;
    let r=c.merge_restores[index].clone();
    if c.merge_restores[index+1..].iter().any(|x|x.target==r.target){return Err("Restore the most recent merge first".into());}
    let target=c.fences.iter().find(|f|f.id==r.target).cloned().ok_or("Target fence does not exist")?;
    let target_monitor=container_monitor(c,&target).to_string();
    let on_display=|c:&Config,f:&Fence| {
        let name=container_monitor(c,f);
        if let Some((display_name,monitors))=display {
            let actual=monitors.iter().find(|m|m.name==name).or_else(||monitors.iter().find(|m|m.primary)).or(monitors.first());
            actual.is_some_and(|m|m.name==display_name)
        } else {name==target_monitor}
    };
    if !on_display(c,&target){return Err("Target is on another display".into());}
    if c.fences.iter().any(|f|f.id==r.source.id){return Err("Source identity already exists".into());}
    if ![target.w,target.h].iter().all(|v|v.is_finite()&&*v>0.){return Err("Invalid fence size".into());}
    let original_target:HashSet<_>=r.target_items.iter().collect();let current:HashSet<_>=target.items.iter().collect();
    let elsewhere:HashSet<_>=c.fences.iter().filter(|f|f.id!=target.id).flat_map(|f|f.items.iter()).collect();
    let mut source=target.clone();source.id=r.source.id;source.title=r.source.title;source.monitor=target_monitor.clone();
    let mut seen=HashSet::new();source.items=r.source.items.iter().filter(|id|valid.contains(*id)&&current.contains(id)&&!original_target.contains(id)&&!elsewhere.contains(id)&&seen.insert((*id).clone())).cloned().collect();
    let recovered:HashSet<_>=source.items.iter().cloned().collect();let mut result=target.clone();
    result.title=if r.target_title.is_empty(){target.title.clone()}else{r.target_title};
    let mut seen=HashSet::new();
    result.items=r.target_items.iter().chain(target.items.iter()).filter(|id|
        current.contains(id)&&valid.contains(*id)&&!recovered.contains(*id)&&!elsewhere.contains(id)&&seen.insert((*id).clone())
    ).cloned().collect();
    result.w=result.w.min(work[2]);result.h=result.h.min(work[3]);source.w=result.w;source.h=result.h;
    let geom=geometry(c,&target);let p=clamp([geom[0],geom[1]],[result.w,result.h],work);result.x=p[0];result.y=p[1];
    let neighbors:Vec<_>=c.fences.iter().filter(|f|f.id!=target.id&&on_display(c,f)&&visible_fence(c,f)).map(|f|geometry(c,f)).collect();
    let mut occupied=neighbors.clone();occupied.push([result.x,result.y,result.w,result.h]);
    // Compare remaining area in each direction; obstacles reduce its useful space.
    let right=work[0]+work[2];let bottom=work[1]+work[3];
    let directions=[
        ([result.x+result.w+12.,result.y],[result.x+result.w,work[1],(right-result.x-result.w).max(0.),work[3]]),
        ([result.x-source.w-12.,result.y],[work[0],work[1],(result.x-work[0]).max(0.),work[3]]),
        ([result.x,result.y+result.h+12.],[work[0],result.y+result.h,work[2],(bottom-result.y-result.h).max(0.)]),
        ([result.x,result.y-source.h-12.],[work[0],work[1],work[2],(result.y-work[1]).max(0.)]),
    ];
    let preferred=directions.iter().max_by(|a,b| {
        let free=|r:[f64;4]|(r[2]*r[3]-neighbors.iter().map(|n|overlap(r,*n)).sum::<f64>()).max(0.);
        free(a.1).total_cmp(&free(b.1))
    }).unwrap().0;
    let p=best(preferred,[source.w,source.h],work,&occupied);source.x=p[0];source.y=p[1];
    // Keep the split pair fixed and move only neighbors that intersect it.
    let mut placed=vec![[result.x,result.y,result.w,result.h],[source.x,source.y,source.w,source.h]];
    let neighbors:Vec<_>=c.fences.iter().filter(|f|f.id!=target.id&&on_display(c,f)&&visible_fence(c,f)).map(|f|f.id.clone()).collect();
    for id in neighbors {
        let f=c.fences.iter().find(|f|f.id==id).unwrap();let rect=geometry(c,f);let size=[f.w.min(work[2]),f.h.min(work[3])];
        let p=if placed.iter().any(|r|overlap(rect,*r)>0.){best([rect[0],rect[1]],size,work,&placed)}else{clamp([rect[0],rect[1]],size,work)};
        let f=c.fences.iter_mut().find(|f|f.id==id).unwrap();f.x=p[0];f.y=p[1];f.w=size[0];f.h=size[1];
        if let Some(g)=c.fence_groups.iter_mut().find(|g|g.active==id){g.x=p[0];g.y=p[1];}
        placed.push([p[0],p[1],size[0],size[1]]);
    }
    if let Some(g)=c.fence_groups.iter_mut().find(|g|g.members.contains(&target.id)){g.x=result.x;g.y=result.y;}
    *c.fences.iter_mut().find(|f|f.id==target.id).unwrap()=result;c.fences.push(source);c.merge_restores.remove(index);Ok(())
}
pub fn remap(c:&mut Config,old:&str,new:&str) {
    for r in &mut c.merge_restores {for id in r.source.items.iter_mut().chain(r.target_items.iter_mut()){if id==old{*id=new.into();}}}
}
pub fn deleted(c:&mut Config,id:&str) {
    remove_member(c,id);c.merge_restores.retain(|r|r.target!=id&&r.source.id!=id);
}

pub fn move_group(c:&mut Config,id:&str,point:[f64;2])->Result<(),String>{
 if !point.iter().all(|n|n.is_finite()){return Err("Invalid position".into());}
 let g=c.fence_groups.iter_mut().find(|g|g.id==id).ok_or("Group does not exist")?;g.x=point[0];g.y=point[1];Ok(())
}
