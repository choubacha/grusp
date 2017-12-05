use std;
use std::path::{PathBuf, Path};
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use regex::Regex;

#[derive(Debug)]
pub struct Matches {
    pub path: PathBuf,
    pub count: u32,
    pub matches: Vec<Match>,
}

#[derive(Debug)]
pub struct Match {
    pub number: u32,
    pub line: String,
    pub captures: Vec<String>
}

impl Matches {
    pub fn has_matches(&self) -> bool {
        self.matches.len() > 0
    }

    fn from_path(path: &Path) -> Self {
        Matches {
            path: path.to_owned(),
            count: 0,
            matches: Vec::new(),
        }
    }

    fn add(&mut self, m: Match) {
        self.count += 1;
        self.matches.push(m);
    }
}

impl Match {
    fn new(line: String, number: u32, captures: Vec<String>) -> Match {
        Match { number, line, captures }
    }
}

pub fn find_matches(path: &Path, regex: &Regex) -> std::io::Result<Matches> {
    let handle = File::open(path)?;
    let mut reader = BufReader::new(handle);

    let mut matches = Matches::from_path(path);
    let mut line_number = 1;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(size) if size > 0 => {
                    if regex.is_match(&line) {
                    let mut captures: Vec<String> = Vec::new();
                    for caps in regex.captures_iter(&line) {
                        captures.push(caps.get(0).map_or(String::new(), |m| m.as_str().to_string()))
                    }
                    matches.add(Match::new(line, line_number, captures));
                }
            },
            _ => break,
        }
        line_number += 1;
    }
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn matches_knows_it_has_matches() {
        let path = Path::new("./src/main.rs");
        let mut matches = Matches::from_path(path);
        assert!(!matches.has_matches());
        matches.add(Match::new("some line".to_string(), 10, vec!["some".to_string()]));
        assert!(matches.has_matches());
    }

    #[test]
    fn matches_tracks_count() {
        let path = Path::new("./src/main.rs");
        let mut matches = Matches::from_path(path);
        assert_eq!(matches.count, 0);
        matches.add(Match::new("some line".to_string(), 10, vec!["some".to_string()]));
        assert_eq!(matches.count, 1);
    }

    #[test]
    fn find_main_rs() {
        let reg = Regex::new(r"fn\s+main").unwrap();
        let path = Path::new("./src/main.rs");
        let matches = find_matches(path,&reg).unwrap();
        assert_eq!(matches.path, path);
        assert_eq!(matches.count, 1);
        assert_eq!(matches.matches.len(), 1);
        assert!(reg.is_match(&matches.matches[0].line));
        assert!(matches.has_matches());
    }
}
