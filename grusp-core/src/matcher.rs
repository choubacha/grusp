use std;
use std::path::{PathBuf, Path};
use std::io::prelude::*;
use regex::Regex;
use std::sync::{Arc, Mutex};

/// A struct that tallies and maintains an aggregated stats history of matches
/// even across threads.
#[derive(Clone, Debug)]
pub struct Stats {
    counts: Arc<Mutex<Counts>>,
}

#[derive(Debug)]
struct Counts {
    total: u64,
    lines: u64,
    captures: u64,
}

impl Stats {
    /// Creates a new stat collector struct to tally and keep track of how many
    /// lines, captures, and files match
    pub fn new() -> Self {
        Self { counts: Arc::new(Mutex::new(Counts { total: 0, lines: 0, captures: 0 })) }
    }

    /// Adds a set of matches for a given file to the stats.
    pub fn add(&self, m: &Matches) -> () {
        if m.has_matches() {
            let mut counts = self.counts.lock().unwrap();
            counts.total += 1;
            counts.lines += m.matches.len() as u64;
            let capture_count: u64 = m.matches.iter().map(|m| m.captures.len() as u64).sum();
            counts.captures += capture_count;
        }
    }

    /// Returns the total number of matched files.
    pub fn total(&self) -> u64 {
        self.counts.lock().unwrap().total
    }

    /// Returns the total number of captures.
    pub fn captures(&self) -> u64 {
        self.counts.lock().unwrap().captures
    }

    /// Returns the total number of matched lines.
    pub fn lines(&self) -> u64 {
        self.counts.lock().unwrap().lines
    }
}

#[derive(Debug)]
pub struct Matches {
    pub path: Option<PathBuf>,
    pub count: u32,
    pub matches: Vec<Line>,
}

#[derive(Debug)]
pub struct Line {
    pub number: Option<usize>,
    pub value: String,
    pub captures: Vec<Capture>,
}

#[derive(Debug)]
pub struct Capture {
    pub start: usize,
    pub end: usize,
    pub value: String,
}

impl Matches {
    pub fn has_matches(&self) -> bool {
        self.matches.len() > 0
    }

    pub fn add_path(mut self, path: &Path) -> Self {
        self.path = Some(path.to_owned());
        self
    }

    fn new() -> Self {
        Matches {
            path: None,
            count: 0,
            matches: Vec::new(),
        }
    }

    fn add(&mut self, m: Line) {
        self.count += 1;
        self.matches.push(m);
    }
}

impl Line {
    fn new(value: String, captures: Vec<Capture>) -> Self {
        Self {
            number: None,
            value,
            captures,
        }
    }

    fn line_number(self, number: usize) -> Self {
        Self { number: Some(number), ..self }
    }
}

/// A struct for accumulating and building the matches.
struct Matcher<'a> {
    line_number: usize,
    matches: Matches,
    regex: &'a Regex,
    with_line_numbers: bool,
}

impl<'a> Matcher<'a> {
    /// Creates a new matcher with default values
    fn new(regex: &'a Regex) -> Self {
        Matcher {
            line_number: 0,
            matches: Matches::new(),
            regex,
            with_line_numbers: true,
        }
    }

    /// Toggle the tracking of line numbers. If set to false, the returned matches
    /// will not include the line numbers. Useful when the buffer is not actually a file.
    fn with_line_numbers(mut self, w: bool) -> Self {
        self.with_line_numbers = w;
        self
    }

    fn add(&mut self, m: Line) {
        if self.with_line_numbers {
            self.matches.add(m.line_number(self.line_number));
        } else {
            self.matches.add(m);
        }
    }

    fn increment_line_number(&mut self) {
        self.line_number += 1;
    }

    /// Mutably consumes the matcher and returns a result with the matches.
    fn collect<T: BufRead>(mut self, reader: &mut T) -> std::io::Result<Matches> {
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(size) if size > 0 => {
                    self.increment_line_number();
                    if let Some(m) = match_line(&line, &self.regex) {
                        self.add(m);
                    }
                }
                _ => break,
            }
        }
        Ok(self.matches)
    }
}

/// Finds matches against a bufreader using the available regex. Does not collect line numbers
pub fn find_matches_wo_line_numbers<T: BufRead>(mut reader: T, regex: &Regex) -> std::io::Result<Matches> {
    Matcher::new(&regex).with_line_numbers(false).collect(&mut reader)
}

/// Finds matches against a bufreader using the available regex.
pub fn find_matches<T: BufRead>(mut reader: T, regex: &Regex) -> std::io::Result<Matches> {
    Matcher::new(&regex).collect(&mut reader)
}

fn match_line(line: &str, regex: &Regex) -> Option<Line> {
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
        Some(Line::new(line.to_string(), captures))
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
        assert_eq!(m.value, "some test line with test matching");
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
            matches.add(Line::new(
                "some line".to_string(),
                vec![
                    Capture { start: 0, end: 1, value: "some".to_string(), },
                    Capture { start: 0, end: 1, value: "some".to_string(), },
                ],
            ));
            matches.add(Line::new(
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
        matches.add(Line::new(
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
        matches.add(Line::new(
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
        assert!(reg.is_match(&matches.matches[0].value));
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
