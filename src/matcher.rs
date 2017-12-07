use std;
use std::path::{PathBuf, Path};
use std::io::prelude::*;
use regex::Regex;

pub struct Matches {
    pub path: Option<PathBuf>,
    pub count: u32,
    pub matches: Vec<Match>,
}

pub struct Match {
    pub number: u32,
    pub line: String,
    pub captures: Vec<Capture>,
}

pub struct Capture {
    pub start: usize,
    pub end: usize,
    pub value: String,
}

impl Matches {
    pub fn has_matches(&self) -> bool {
        self.matches.len() > 0
    }

    pub fn add_path(self, path: &Path) -> Self {
        Matches { path: Some(path.to_owned()), .. self }
    }

    fn new() -> Self {
        Matches {
            path: None,
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
    fn new(line: String, number: u32, captures: Vec<Capture>) -> Match {
        Match { number, line, captures }
    }
}

pub fn find_matches<T: BufRead>(reader: &mut T, regex: &Regex) -> std::io::Result<Matches> {
    let mut matches = Matches::new();
    let mut line_number = 1;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(size) if size > 0 => {
                let cap_matches = regex.captures_iter(&line);
                let captures: Vec<Capture> = cap_matches
                    .map(|caps| caps.get(0))
                    .filter(|m| m.is_some())
                    .map(|m| m.unwrap())
                    .map(|m| Capture { start: m.start(), end: m.end(), value: m.as_str().to_string() })
                    .collect();
                if captures.len() > 0 {
                    matches.add(Match::new(line.to_string(), line_number, captures));
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
        let mut matches = Matches::new();
        assert!(!matches.has_matches());
        matches.add(
            Match::new("some line".to_string(),
                           10,
                           vec![Capture { start: 0, end: 1, value: "some".to_string() }]
            )
        );
        assert!(matches.has_matches());
    }

    #[test]
    fn matches_tracks_count() {
        let mut matches = Matches::new();
        assert_eq!(matches.count, 0);
        matches.add(
            Match::new("some line".to_string(),
                       10,
                       vec![Capture { start: 0, end: 1, value: "some".to_string() }]
            )
        );
        assert_eq!(matches.count, 1);
    }

    #[test]
    fn matches_can_add_path() {
        let path = Path::new("./src/main.rs");
        let matches = Matches::new().add_path(path);
        assert_eq!(matches.path, Some(path.to_owned()));
    }

    #[test]
    fn find_main_rs() {
        let reg = Regex::new(r"fn\s+main").unwrap();
        use std::io::Cursor;
        let mut buf_read = Cursor::new("some text\nfn    main() {}\nhello");
        let matches = find_matches(&mut buf_read, &reg).unwrap();
        assert_eq!(matches.path, None);
        assert_eq!(matches.count, 1);
        assert_eq!(matches.matches.len(), 1);
        assert!(reg.is_match(&matches.matches[0].line));
        assert!(matches.has_matches());
    }

    #[test]
    fn it_tracks_the_line_numbers_from_one() {
        let reg = Regex::new(r"test").unwrap();
        use std::io::Cursor;
        let mut buf_read = Cursor::new("test\nnot\ntest");
        let matches = find_matches(&mut buf_read, &reg).unwrap();
        assert_eq!(matches.count, 2);
        assert_eq!(matches.matches.len(), 2);
        assert_eq!(matches.matches[0].number, 1);
        assert_eq!(matches.matches[1].number, 3);
    }

    #[test]
    fn finds_all_the_captures() {
        let reg = Regex::new(r"test").unwrap();
        use std::io::Cursor;
        let mut buf_read = Cursor::new("test a test b test");
        let matches = find_matches(&mut buf_read, &reg).unwrap();
        assert!(matches.has_matches());
        assert_eq!(matches.matches[0].captures[0].value, "test".to_string());
    }
}
