//! Learning key normalization recovered from original scanner control flow.
pub fn key(name: &str, target: &str, url: &str) -> String {
    if !target.is_empty() {
        let normalized=target.to_lowercase().replace('/', "\\");
        format!("t:{}",normalized.rsplit('\\').next().unwrap_or(""))
    } else if !url.is_empty() { format!("u:{}",url.to_lowercase()) }
    else {format!("n:{}",name.to_lowercase())}
}
pub fn internet_url(content: &str) -> String {
    content.lines().find_map(|line|line.trim().trim_start_matches('\u{feff}').strip_prefix("URL=")).unwrap_or("").trim().to_lowercase()
}
