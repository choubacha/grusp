extern crate clap;
extern crate glob;
extern crate regex;

mod matcher;
use std::env;
use glob::glob;
use regex::Regex;


fn main() {
    let args: Vec<_> = env::args().collect();
    let regex = &args[1];
    let regex = Regex::new(regex);
    if let Err(parse_error) = regex {
        println!("{}", parse_error);
        std::process::exit(1);
    }
    let regex = regex.unwrap();

    let default_search = [".".to_string()];
    let queries = if args.len() > 2 { &args[2..] } else { &default_search };
    for query in queries {
        let files = glob(query).expect("Glob pattern failed");
        files
            .filter(|p| p.is_ok())
            .map(|p| p.unwrap())
            .map(|p| matcher::find_matches(p.as_path(), &regex).expect("Could not parse file"))
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