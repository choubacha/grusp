extern crate clap;
extern crate glob;
extern crate regex;
extern crate rayon;
extern crate colored;
extern crate atty;

mod matcher;
mod args;
mod display;
mod files;
use glob::glob;
use rayon::prelude::*;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs::File;
use std::io::stdin;

fn main() {
    let opts = match args::get_opts() {
        Ok(o) => o,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    };

    if let Some(ref queries) = opts.queries {
        let stats = matcher::Stats::new();
        let files = collect_files(&queries);

        if opts.is_concurrent {
            files.into_par_iter().for_each(|p| match_file(p, &opts, &stats));
        } else {
            files.into_iter().for_each(|p| match_file(p, &opts, &stats));
        };
        if stats.total() == 0 {
            std::process::exit(1);
        }
    } else {
        let stdin = stdin();
        let mut reader = stdin.lock();
        let matches = matcher::find_matches_wo_line_numbers(&mut reader, &opts.regex)
            .expect("Could not parse file");
        if matches.has_matches() {
            println!("{}", display::MatchesDisplay::new(matches).count_only(opts.is_count_only));
        } else {
            std::process::exit(1);
        }
    }
}

fn collect_files(queries: &Vec<String>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for query in queries {
        glob(&query)
            .expect("Glob pattern failed")
            .filter(|p| p.is_ok())
            .map(|p| p.expect("An 'ok' file was not found"))
            .for_each(|p| {
                files::recurse(p, &mut files).expect("Unknown file error")
            });
    }
    files
}

fn match_file(path: PathBuf, opts: &args::Opts, stats: &matcher::Stats) {
    let handle = File::open(&path).unwrap();
    let mut reader = BufReader::new(handle);
    let matches = matcher::find_matches(&mut reader, &opts.regex)
        .expect("Could not parse file")
        .add_path(&path);
    if matches.has_matches() {
        stats.add(&matches);
        println!(
            "{}",
            display::MatchesDisplay::new(matches)
                .count_only(opts.is_count_only)
                .color(opts.is_colored)
        );
    }
}
