use crate::{classify, model::*};
use std::collections::HashSet;

pub fn item_id(path: &str) -> String { path.to_lowercase() }

pub fn legacy_item_id(path: &str) -> String {
    let hash = path.to_lowercase().bytes().fold(0xcbf29ce484222325u64, |hash, x| (hash ^ x as u64).wrapping_mul(0x100000001b3));
    format!("{hash:016x}")
}

pub fn adopt_legacy_ids(c: &mut Config, items: &[Item]) {
    for item in items {
        let old = legacy_item_id(&item.path);
        crate::fence_merge::remap(c, &old, &item.id);
        for fence in &mut c.fences {
            for id in &mut fence.items { if *id == old { *id = item.id.clone(); } }
        }
        let old_key = match item.kind.as_str() {
            "shortcut" => format!("name:{}", item.name.to_lowercase()),
            "folder" | "system" => format!("path:{}", item.path.to_lowercase()),
            _ if !item.ext.is_empty() => format!("ext:{}", item.ext.to_lowercase()),
            _ => format!("name:{}", item.name.to_lowercase()),
        };
        if let Some(category) = c.learned.get(&old_key).cloned() { c.learned.entry(learning_key(item)).or_insert(category); }
    }
}

pub fn remap_item(c: &mut Config, old: &Item, new: &Item) {
    crate::fence_merge::remap(c, &old.id, &new.id);
    for fence in &mut c.fences {
        for id in &mut fence.items { if *id == old.id { *id = new.id.clone(); } }
    }
    let old_key = learning_key(old);
    let new_key = learning_key(new);
    // Extension learning is shared by all files; renaming one file must not move it.
    if old_key != new_key && !old_key.starts_with("ext:") && !new_key.starts_with("ext:") {
        if let Some(category) = c.learned.remove(&old_key) { c.learned.insert(new_key, category); }
    }
}

pub fn create_fence(c: &mut Config, monitor: &str, origin: [f64; 2], title: &str, category: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    c.fences.push(Fence { id: id.clone(), title: title.into(), x: origin[0], y: origin[1],
        w: 320., h: 260., style: Style::new(), category: (!category.is_empty()).then(|| category.into()),
        monitor: monitor.into(), collapsed: false, hidden: false, items: vec![] });
    id
}

pub fn learning_key(item: &Item) -> String {
    item.learning_key.clone().unwrap_or_else(||crate::learning::key(&item.name, "", ""))
}

pub fn move_item(c: &mut Config, item: &Item, destination: &str, index: Option<usize>) -> Result<(), String> {
    let category = c.fences.iter().find(|f| f.id == destination)
        .ok_or("Destination fence does not exist")?.category.clone();
    for fence in &mut c.fences { fence.items.retain(|id| id != &item.id); }
    let target = c.fences.iter_mut().find(|f| f.id == destination).unwrap();
    target.items.insert(index.unwrap_or(target.items.len()).min(target.items.len()), item.id.clone());
    if let Some(category) = category.filter(|x| x != "inbox") { c.learned.insert(learning_key(item), category); }
    Ok(())
}

pub fn delete_fence(c: &mut Config, id: &str) -> Result<(), String> {
    let index = c.fences.iter().position(|f| f.id == id).ok_or("Fence does not exist")?;
    let removed = c.fences.remove(index);
    crate::fence_merge::deleted(c, id);
    if !removed.items.is_empty() {
        let other = c.fences.iter().find(|f| f.category.as_deref() == Some("other")).map(|f| f.id.clone())
            .unwrap_or_else(|| create_fence(c, &removed.monitor, [removed.x, removed.y], "其他", "other"));
        let destination = c.fences.iter_mut().find(|f| f.id == other).unwrap();
        destination.items.extend(removed.items.into_iter().filter(|id| !destination.items.contains(id)).collect::<Vec<_>>());
    }
    Ok(())
}

fn free_position(c: &Config, monitor: &str, size: [f64; 2]) -> [f64; 2] {
    let cols = ((size[0] - 36.) / 332.).floor().max(1.) as usize;
    let rows = ((size[1] - 36.) / 272.).floor().max(1.) as usize;
    (0..cols * rows).map(|i| [24. + (i % cols) as f64 * 332., 24. + (i / cols) as f64 * 272.])
        .find(|p| c.fences.iter().filter(|f| f.monitor == monitor && !f.hidden).all(|f|
            p[0] + 320. <= f.x || f.x + f.w <= p[0] || p[1] + 260. <= f.y || f.y + f.h <= p[1]))
        .unwrap_or([24., 24.])
}

pub fn collect_inbox(c: &mut Config, items: &[Item], monitor: &str, size: [f64; 2]) {
    let valid: HashSet<_> = items.iter().map(|i| i.id.as_str()).collect();
    let mut assigned = HashSet::new();
    for fence in &mut c.fences {
        fence.items.retain(|id| valid.contains(id.as_str()) && assigned.insert(id.clone()));
    }
    let pending: Vec<_> = items.iter().filter(|i| !assigned.contains(&i.id)).map(|i| i.id.clone()).collect();
    if pending.is_empty() { return; }
    let destination = c.fences.iter().find(|f| f.category.as_deref() == Some("inbox")).map(|f| f.id.clone())
        .unwrap_or_else(|| create_fence(c, monitor, free_position(c, monitor, size), "新文件", "inbox"));
    c.fences.iter_mut().find(|f| f.id == destination).unwrap().items.extend(pending);
}

pub fn organize(c: &mut Config, items: &[Item], monitor: &str, size: [f64; 2], full: bool) {
    let valid: HashSet<&str> = items.iter().map(|i| i.id.as_str()).collect();
    let mut assigned = HashSet::new();
    let grouped: HashSet<_> = c.fence_groups.iter().flat_map(|g|g.members.iter().cloned()).collect();
    for fence in &mut c.fences {
        if full || (fence.category.as_deref() == Some("inbox") && !grouped.contains(&fence.id)) { fence.items.clear(); }
        else { fence.items.retain(|id| valid.contains(id.as_str()) && assigned.insert(id.clone())); }
    }
    let categories = classify::categories();
    for item in items {
        if !full && assigned.contains(&item.id) { continue; }
        let category = c.learned.get(&learning_key(item)).cloned().unwrap_or_else(|| item.category.clone());
        let category = if categories.iter().any(|x| x.id == category) { category } else { "other".into() };
        let destination = c.fences.iter().find(|f| f.category.as_deref() == Some(&category)).map(|f| f.id.clone())
            .unwrap_or_else(|| {
                let title = &categories.iter().find(|x| x.id == category).unwrap().name;
                let position = free_position(c, monitor, size);
                create_fence(c, monitor, position, title, &category)
            });
        c.fences.iter_mut().find(|f| f.id == destination).unwrap().items.push(item.id.clone());
    }
    // Inferred layout: category order, responsive grid. No files are physically moved.
    let cols = ((size[0] - 36.) / 332.).floor().max(1.) as usize;
    let mut on_monitor: Vec<_> = c.fences.iter().filter(|f|
        crate::fence_merge::container_monitor(c,f)==monitor && crate::fence_merge::visible_fence(c,f)
    ).map(|f|f.id.clone()).collect();
    on_monitor.sort_by_key(|id| {
        let f=c.fences.iter().find(|f|&f.id==id).unwrap();
        categories.iter().find(|x| Some(&x.id)==f.category.as_ref()).map(|x|x.order).unwrap_or(14)
    });
    if full || !c.initialized {
        let rows = on_monitor.len().div_ceil(cols).max(1);
        let height = ((size[1] - 48.) / rows as f64 - 12.).clamp(100., 260.);
        for (i, id) in on_monitor.into_iter().enumerate() {
            let fence=c.fences.iter_mut().find(|f|f.id==id).unwrap();
            fence.x = 24. + (i % cols) as f64 * 332.;
            fence.y = 24. + (i / cols) as f64 * (height + 12.);
            fence.w = (size[0] - 48.).clamp(140., 320.);
            fence.h = height;
            if let Some(group)=c.fence_groups.iter_mut().find(|g|g.active==id) {
                group.x=fence.x;group.y=fence.y;
            }
        }
    }
    c.initialized = true;
}

pub fn monitor_assignment_valid(name: &str, existing: Option<&str>, monitors: &[Monitor]) -> bool {
    monitors.iter().any(|m|m.name==name) || existing == Some(name)
}
pub fn locate_monitor(monitors: &[Monitor], point: [f64;2]) -> Result<(String,[f64;2],[f64;2]),String> {
    if !point.iter().all(|x|x.is_finite()) {return Err("Invalid screen origin".into());}
    let primary=monitors.iter().find(|m|m.primary).or(monitors.first()).ok_or("No monitor available")?;
    let monitor=monitors.iter().find(|m|point[0]>=m.x as f64 && point[1]>=m.y as f64
        && point[0]<m.x as f64+m.w as f64 && point[1]<m.y as f64+m.h as f64).unwrap_or(primary);
    Ok((monitor.name.clone(),[(point[0]-monitor.x as f64)/monitor.scale,(point[1]-monitor.y as f64)/monitor.scale],
        [monitor.w as f64/monitor.scale,monitor.h as f64/monitor.scale]))
}
