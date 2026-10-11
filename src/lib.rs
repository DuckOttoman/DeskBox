pub mod classify;
pub mod learning;
pub mod config;
pub mod model;
pub mod state;
pub mod fence_merge;
pub mod rename;
pub mod pixels;
#[cfg(feature = "gui")]
pub mod app;
#[cfg(feature = "gui")]
mod commands;
#[cfg(feature = "gui")]
mod scanner;
#[cfg(feature = "gui")]
mod sysmon;
#[cfg(feature = "gui")]
mod win;
pub mod media;
pub mod widgets;
#[cfg(feature = "gui")]
mod widget_album;

pub mod memory;

pub mod thermal;

pub mod layout_backup;
