#[macro_use]
extern crate clap;
extern crate glob;
extern crate regex;
extern crate rayon;
extern crate colored;

mod matcher;
mod args;
mod display;
mod files;
use glob::glob;
use rayon::prelude::*;
use std::path::PathBuf;

fn main() {
    let opts = match args::get_opts() {
        Ok(o) => o,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    };
    let mut files = Vec::new();
    for query in &opts.queries {
        glob(&query)
            .expect("Glob pattern failed")
            .filter(|p| p.is_ok())
            .map(|p| p.expect("An 'ok' file was not found"))
            .for_each(|p| files::recurse(p, &mut files).expect("Unknown file error"));
    }

    if opts.is_concurrent {
        files.into_par_iter().for_each(|p| match_file(p, &opts));
    } else {
        files.into_iter().for_each(|p| match_file(p, &opts));
    };
}

fn match_file(path: PathBuf, opts: &args::Opts) {
    let matches = matcher::find_matches(path.as_path(), &opts.regex).expect("Could not parse file");
    if matches.has_matches() {
        println!("{}", display::MatchesDisplay::new(matches));
    }
}
