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
use std::io::BufReader;
use std::fs::File;

fn main() {
    let opts = match args::get_opts() {
        Ok(o) => o,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    };

    if let Some(ref queries) = opts.queries {
        let mut files = Vec::new();
        let match_stats = matcher::MatchCount::new();

        for query in queries {
            glob(&query)
                .expect("Glob pattern failed")
                .filter(|p| p.is_ok())
                .map(|p| p.expect("An 'ok' file was not found"))
                .for_each(|p| {
                    files::recurse(p, &mut files).expect("Unknown file error")
                });
        }

        if opts.is_concurrent {
            files.into_par_iter().for_each(
                |p| match_file(p, &opts, &match_stats),
            );
        } else {
            files.into_iter().for_each(
                |p| match_file(p, &opts, &match_stats),
            );
        };
        if match_stats.total() == 0 {
            std::process::exit(1);
        }
    } else {
        use std::io::stdin;
        let stdin = stdin();
        let mut reader = stdin.lock();
        let matches =
            matcher::find_matches(&mut reader, &opts.regex).expect("Could not parse file");
        if matches.has_matches() {
            println!("{}", display::MatchesDisplay::new(matches));
        } else {
            std::process::exit(1);
        }
    }
}

fn match_file(path: PathBuf, opts: &args::Opts, match_stats: &matcher::MatchCount) {
    let handle = File::open(&path).unwrap();
    let mut reader = BufReader::new(handle);
    let matches = matcher::find_matches(&mut reader, &opts.regex)
        .expect("Could not parse file")
        .add_path(&path);
    if matches.has_matches() {
        match_stats.add(&matches);
        println!(
            "{}",
            display::MatchesDisplay::new(matches).color(opts.is_colored)
        );
    }
}
