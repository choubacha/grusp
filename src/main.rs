#[macro_use]
extern crate clap;
extern crate glob;
extern crate regex;

mod matcher;
mod args;
use std::env;
use glob::glob;
use regex::Regex;


fn main() {
    let opts = match args::get_opts() {
        Ok(o) => o,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    };
    for query in &opts.queries {
        let files = glob(&query).expect("Glob pattern failed");
        files
            .filter(|p| p.is_ok())
            .map(|p| p.unwrap())
            .map(|p| matcher::find_matches(p.as_path(), &opts.regex).expect("Could not parse file"))
            .filter(|matches| matches.has_matches())
            .for_each(|matches| {
                println!("{} matched {} times", matches.path.as_path().to_str().unwrap(), matches.count);
                for m in matches.matches {
                    println!("{}:{}", m.number, m.line.trim_right());
                }
                println!();
            })
    }
}