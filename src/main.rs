#[macro_use]
extern crate clap;
extern crate glob;
extern crate regex;

mod matcher;
mod args;
mod display;
mod files;
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
        let files = glob(&query)
            .expect("Glob pattern failed")
            .filter(|p| p.is_ok())
            .map(|p| p.expect("An 'ok' file was not found"));
        for file in files {
            let sub_files = files::recurse(file).expect("Could not find files");
            sub_files
                .into_iter()
                .map(|p| matcher::find_matches(p.as_path(), &opts.regex).expect("Could not parse file"))
                .filter(|m| m.has_matches())
                .map(|m| display::MatchDisplay::new(m))
                .for_each(|m| println!("{}", m));
        }
    }
}