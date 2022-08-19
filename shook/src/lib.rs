pub mod another_viewer;
pub use another_viewer::AnotherViewer;

pub mod builtin;
pub use builtin::Builtin;

pub mod crates;
pub use crates::Crates;

pub mod user_defined;
pub use user_defined::UserDefined;

pub mod what_song;
pub use what_song::WhatSong;

pub mod config;

mod queue;

include!(concat!(env!("OUT_DIR"), "/", "version.rs"));
