//! Rule tables are original data; precedence/longest-match selection is reconstructed.
use crate::model::Category;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize)]
struct Rules {
    categories: Vec<Category>,
    extensions: BTreeMap<String, Vec<String>>,
    keywords: BTreeMap<String, Vec<String>>,
}
fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(|| serde_json::from_str(include_str!("../evidence/classification-rules.json"))
        .expect("verified original rule tables must be valid JSON"))
}
pub fn categories() -> Vec<Category> { rules().categories.clone() }

pub fn classify(name: &str, target: &str, kind: &str) -> &'static str {
    match kind {
        "system" => return "system",
        "folder" => return "folder",
        _ => (),
    }
    let lower = name.to_lowercase();
    let ext = lower.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
    if kind != "shortcut" {
        for category in ["document", "image", "media", "archive", "dev", "tools"] {
            if rules().extensions[category].iter().any(|x| x == ext) { return category; }
        }
    }
    let text = format!("{} {}", lower, target.to_lowercase());
    let mut best = (0, "other");
    for category in ["game", "design", "dev", "office", "social", "entertainment", "tools"] {
        for keyword in &rules().keywords[category] {
            let matches = text.match_indices(keyword.as_str()).any(|(start, _)| {
                // Prevent short English product names (e.g. "zed") matching
                // unrelated words (e.g. "unrecognized"). Chinese stays substring-based.
                if !keyword.is_ascii() { return true; }
                let end = start + keyword.len();
                let before = text[..start].chars().next_back();
                let after = text[end..].chars().next();
                let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
                (!keyword.chars().next().is_some_and(is_word) || !before.is_some_and(is_word))
                    && (!keyword.chars().next_back().is_some_and(is_word) || !after.is_some_and(is_word))
            });
            if keyword.len() > best.0 && matches {
                best = (keyword.len(), category);
            }
        }
    }
    best.1
}
