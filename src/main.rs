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
    let matcher = grusp::Matcher::new(&opts.regex)
        .keep_lines(!(opts.just_files.is_some() || opts.is_count_only))
        .invert_match(opts.is_inverted);

    if let Some(ref queries) = opts.queries {
        let stats = grusp::StatCollector::new();
        let files = grusp::FileCollector::new(&queries).max_depth(opts.max_depth).collect();
        let has_files = !files.is_empty();

        if opts.is_concurrent {
            files
                .into_par_iter()
                .for_each(|p| {
                    match_file(p, &opts, &matcher, &stats)
                });
        } else {
            files
                .into_iter()
                .for_each(|p| {
                    match_file(p, &opts, &matcher, &stats)
                });
        };
        if stats.total() == 0 && !(has_files && opts.just_files.without_matches()) {
            std::process::exit(1);
        }
    } else {
        let stdin = stdin();
        let mut reader = stdin.lock();
        let matches = matcher
            .with_line_numbers(false)
            .collect(&mut reader)
            .expect("Could not parse stdin");
        if matches.has_matches() {
            println!(
                "{}",
                grusp::Display::new(matches)
                    .count_only(opts.is_count_only)
                    .color(opts.is_colored)
                    .just_file_names(opts.just_files.is_some())
            );
        } else {
            std::process::exit(1);
        }
    }
}

fn match_file(path: PathBuf,
              opts: &args::Opts,
              matcher: &grusp::Matcher,
              stats: &grusp::StatCollector) {
    let handle = File::open(&path).unwrap();
    let mut reader = BufReader::new(handle);
    let matches = matcher
        .collect(&mut reader)
        .expect("Could not parse file")
        .add_path(&path);
    stats.add(&matches);
    if (matches.has_matches() && opts.just_files.show_matches()) ||
        (!matches.has_matches() && opts.just_files.without_matches()) {
        println!(
            "{}",
            grusp::Display::new(matches)
                .count_only(opts.is_count_only)
                .color(opts.is_colored)
                .just_file_names(opts.just_files.is_some())
        );
    }
}
