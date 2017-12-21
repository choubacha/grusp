extern crate glob;
extern crate regex;
extern crate colored;

mod matcher;
mod display;
mod files;

pub mod grusp {
    pub use matcher::{find_matches_wo_line_numbers, find_matches, Stats as StatCollector};
    pub use display::{MatchesDisplay as Display};
    pub use files::{Collecter as FileCollector};
}
