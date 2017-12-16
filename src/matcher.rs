use std;
use std::path::{PathBuf, Path};
use std::io::prelude::*;
use regex::Regex;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Stats {
    counts: Arc<Mutex<Counts>>,
}

struct Counts {
    total: u64,
    lines: u64,
    captures: u64,
}

impl Stats {
    pub fn new() -> Self {
        Stats { counts: Arc::new(Mutex::new(Counts { total: 0, lines: 0, captures: 0 })) }
    }

    pub fn add(&self, m: &Matches) -> () {
        if m.has_matches() {
            let mut counts = self.counts.lock().unwrap();
            counts.total += 1;
            counts.lines += m.matches.len() as u64;
            let capture_count: u64 = m.matches.iter().map(|m| m.captures.len() as u64).sum();
            counts.captures += capture_count;
        }
    }

    pub fn total(&self) -> u64 {
        self.counts.lock().unwrap().total
    }

    pub fn captures(&self) -> u64 {
        self.counts.lock().unwrap().captures
    }

    pub fn lines(&self) -> u64 {
        self.counts.lock().unwrap().lines
    }
}

pub struct Matches {
    pub path: Option<PathBuf>,
    pub count: u32,
    pub matches: Vec<Match>,
}

pub struct Match {
    pub number: Option<u32>,
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
        Matches {
            path: Some(path.to_owned()),
            ..self
        }
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
    fn new(line: String, captures: Vec<Capture>) -> Self {
        Self {
            number: None,
            line,
            captures,
        }
    }

    fn line_number(self, number: u32) -> Self {
        Self { number: Some(number), ..self }
    }
}

pub fn find_matches_wo_line_numbers<T: BufRead>(reader: &mut T, regex: &Regex) -> std::io::Result<Matches> {
    let mut matches = Matches::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(size) if size > 0 => {
                if let Some(m) = match_line(&line, &regex) {
                    matches.add(m);
                }
            }
            _ => break,
        }
    }
    Ok(matches)
}

pub fn find_matches<T: BufRead>(reader: &mut T, regex: &Regex) -> std::io::Result<Matches> {
    let mut matches = Matches::new();
    let mut line_number = 1;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(size) if size > 0 => {
                if let Some(m) = match_line(&line, &regex) {
                    matches.add(m.line_number(line_number));
                }
            }
            _ => break,
        }
        line_number += 1;
    }
    Ok(matches)
}

fn match_line(line: &str, regex: &Regex) -> Option<Match> {
    let cap_matches = regex.captures_iter(&line);
    let captures: Vec<Capture> = cap_matches
        .map(|caps| caps.get(0))
        .filter(|m| m.is_some())
        .map(|m| m.unwrap())
        .map(|m| {
            Capture {
                start: m.start(),
                end: m.end(),
                value: m.as_str().to_string(),
            }
        })
        .collect();
    if captures.len() > 0 {
        Some(Match::new(line.to_string(), captures))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn finding_matches_on_a_line() {
        let reg = Regex::new(r"test").unwrap();
        let m = match_line("some test line with test matching", &reg).unwrap();
        assert_eq!(m.number, None);
        assert_eq!(m.captures.len(), 2);
        assert_eq!(m.line, "some test line with test matching");
    }

    #[test]
    fn finding_matches_on_a_line_returns_none() {
        let reg = Regex::new(r"asdf").unwrap();
        let m = match_line("some test line with test matching", &reg);
        assert!(m.is_none());
    }

    #[test]
    fn can_safely_count_matches() {
        use std::thread;
        let count = Stats::new();
        let mut children = Vec::new();
        for _ in 0..10 {
            let count = count.clone();
            let mut matches = Matches::new();
            matches.add(Match::new(
                "some line".to_string(),
                vec![
                    Capture { start: 0, end: 1, value: "some".to_string(), },
                    Capture { start: 0, end: 1, value: "some".to_string(), },
                ],
            ));
            matches.add(Match::new(
                "some line".to_string(),
                vec![
                    Capture { start: 0, end: 1, value: "some".to_string(), },
                    Capture { start: 0, end: 1, value: "some".to_string(), },
                ],
            ));
            children.push(thread::spawn(move || count.add(&matches)))
        };
        for t in children {
            t.join().unwrap();
        }
        assert_eq!(count.total(), 10);
        assert_eq!(count.lines(), 20);
        assert_eq!(count.captures(), 40);
    }

    #[test]
    fn matches_knows_it_has_matches() {
        let mut matches = Matches::new();
        assert!(!matches.has_matches());
        matches.add(Match::new(
            "some line".to_string(),
            vec![
                Capture { start: 0, end: 1, value: "some".to_string(), },
            ],
        ));
        assert!(matches.has_matches());
    }

    #[test]
    fn matches_tracks_count() {
        let mut matches = Matches::new();
        assert_eq!(matches.count, 0);
        matches.add(Match::new(
            "some line".to_string(),
            vec![
                Capture { start: 0, end: 1, value: "some".to_string(), },
            ],
        ));
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
    fn it_can_skip_line_numbers() {
        let reg = Regex::new(r"test").unwrap();
        use std::io::Cursor;
        let mut buf_read = Cursor::new("test\nnot\ntest");
        let matches = find_matches_wo_line_numbers(&mut buf_read, &reg).unwrap();
        assert_eq!(matches.count, 2);
        assert_eq!(matches.matches.len(), 2);
        assert_eq!(matches.matches[0].number, None);
        assert_eq!(matches.matches[1].number, None);
    }

    #[test]
    fn it_tracks_the_line_numbers_from_one() {
        let reg = Regex::new(r"test").unwrap();
        use std::io::Cursor;
        let mut buf_read = Cursor::new("test\nnot\ntest");
        let matches = find_matches(&mut buf_read, &reg).unwrap();
        assert_eq!(matches.count, 2);
        assert_eq!(matches.matches.len(), 2);
        assert_eq!(matches.matches[0].number, Some(1));
        assert_eq!(matches.matches[1].number, Some(3));
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
