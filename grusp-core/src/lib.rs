#![deny(warnings, missing_docs, missing_debug_implementations)]
//! The core library that allows you to match a regex against buffers and collect
//! the results. It also provides the ability to display this in a colorful way
//! to a terminal.

extern crate glob;
extern crate regex;
extern crate colored;

mod matcher;
mod display;
mod files;

/// The core module for finding matches within files.
pub mod grusp {
    pub use matcher::{find_matches_wo_line_numbers, find_matches, Stats as StatCollector};
    pub use display::{MatchesDisplay as Display};
    pub use files::{Collecter as FileCollector};
}
