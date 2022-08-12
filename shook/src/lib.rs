#[allow(dead_code)]
pub mod another_viewer;
pub mod builtin;
pub mod crates;
pub mod user_defined;
pub mod what_song;

pub mod config;
pub mod local;

mod queue;

include!(concat!(env!("OUT_DIR"), "/", "version.rs"));
