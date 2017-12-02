#[macro_use]
extern crate clap;
extern crate glob;
extern crate regex;

mod matcher;
mod args;
mod display;
use glob::glob;

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
            .filter(|m| m.has_matches())
            .map(|m| display::MatchDisplay::new(m))
            .for_each(|m| println!("{}", m));
    }
}