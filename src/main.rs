extern crate rayon;
extern crate clap;
extern crate atty;
extern crate regex;
extern crate grusp_core;

pub mod args;

use rayon::prelude::*;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs::File;
use std::io::stdin;
use grusp_core::grusp;

fn main() {
    let opts = match args::get_opts() {
        Ok(o) => o,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    };

    if let Some(ref queries) = opts.queries {
        let stats = grusp::StatCollector::new();
        let files = grusp::FileCollector::new(&queries).max_depth(opts.max_depth).collect();

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
        let matches = grusp::Matcher::new(&opts.regex)
            .keep_lines(!(opts.files_with_matches || opts.is_count_only))
            .with_line_numbers(false)
            .collect(&mut reader)
            .expect("Could not parse file");
        if matches.has_matches() {
            println!(
                "{}",
                grusp::Display::new(matches)
                    .count_only(opts.is_count_only)
                    .color(opts.is_colored)
                    .just_file_names(opts.files_with_matches)
            );
        } else {
            std::process::exit(1);
        }
    }
}

fn match_file(path: PathBuf, opts: &args::Opts, stats: &grusp::StatCollector) {
    let handle = File::open(&path).unwrap();
    let mut reader = BufReader::new(handle);
    let matches = grusp::Matcher::new(&opts.regex)
        .keep_lines(!(opts.files_with_matches || opts.is_count_only))
        .collect(&mut reader)
        .expect("Could not parse file")
        .add_path(&path);
    if matches.has_matches() {
        stats.add(&matches);
        println!(
            "{}",
            grusp::Display::new(matches)
                .count_only(opts.is_count_only)
                .color(opts.is_colored)
                .just_file_names(opts.files_with_matches)
        );
    }
}
