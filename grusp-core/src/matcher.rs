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
            counts.lines += m.lines.len() as u64;
            let capture_count: u64 = m.lines.iter().map(|m| m.captures.len() as u64).sum();
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
    pub lines: Vec<Line>,
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
        self.count > 0
    }

    pub fn add_path(mut self, path: &Path) -> Self {
        self.path = Some(path.to_owned());
        self
    }

    fn new() -> Self {
        Matches {
            path: None,
            count: 0,
            lines: Vec::new(),
        }
    }

    fn add(&mut self, m: Line) {
        self.increment();
        self.lines.push(m);
    }

    #[inline]
    fn increment(&mut self) {
        self.count += 1;
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
#[derive(Debug)]
pub struct Matcher<'a> {
    line_number: usize,
    matches: Matches,
    regex: &'a Regex,
    with_line_numbers: bool,
    track_lines: bool,
}

impl<'a> Matcher<'a> {
    /// Creates a new matcher with default values
    pub fn new(regex: &'a Regex) -> Self {
        Matcher {
            line_number: 0,
            matches: Matches::new(),
            regex,
            with_line_numbers: true,
            track_lines: true,
        }
    }

    /// Toggle the tracking of line numbers. If set to false, the returned matches
    /// will not include the line numbers. Useful when the buffer is not actually a file.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate regex;
    /// # extern crate grusp_core;
    /// # fn main() {
    /// use grusp_core::grusp::Matcher;
    /// use std::io::Cursor;
    ///
    /// let reg = regex::Regex::new(r"test").unwrap();
    /// let mut buf_read = Cursor::new("test\nnot\ntest");
    /// let matches = Matcher::new(&reg).with_line_numbers(false).collect(&mut buf_read).unwrap();
    /// assert_eq!(matches.lines[0].number, None);
    /// # }
    /// ```
    pub fn with_line_numbers(mut self, w: bool) -> Self {
        self.with_line_numbers = w;
        self
    }

    /// Toggles the tracking of lines/captures
    pub fn keep_lines(mut self, track_lines: bool) -> Self {
        self.track_lines = track_lines;
        self
    }

    fn add(&mut self, m: Line) {
        if self.track_lines {
            if self.with_line_numbers {
                self.matches.add(m.line_number(self.line_number));
            } else {
                self.matches.add(m);
            }
        } else {
            self.matches.increment();
        }
    }

    fn increment_line_number(&mut self) {
        self.line_number += 1;
    }

    fn match_line(&self, line: &str) -> Option<Line> {
        let captures: Vec<Capture> = self.regex
            .captures_iter(&line)
            .filter_map(|caps| caps.get(0))
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

    /// Consumes the matcher and returns a result with the matches.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate regex;
    /// # extern crate grusp_core;
    /// # fn main() {
    /// use grusp_core::grusp::Matcher;
    /// use std::io::Cursor;
    ///
    /// let reg = regex::Regex::new(r"test").unwrap();
    /// let mut buf_read = Cursor::new("test\nnot\ntest");
    /// let matches = Matcher::new(&reg).collect(&mut buf_read).unwrap();
    /// assert_eq!(matches.count, 2);
    /// # }
    /// ```
    pub fn collect<T: BufRead>(mut self, reader: &mut T) -> std::io::Result<Matches> {
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(size) if size > 0 => {
                    self.increment_line_number();
                    if let Some(m) = self.match_line(&line) {
                        self.add(m);
                    }
                }
                _ => break,
            }
        }
        Ok(self.matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::io::Cursor;

    #[test]
    fn finding_matches_on_a_line() {
        let reg = Regex::new(r"test").unwrap();
        let m = Matcher::new(&reg).match_line("some test line with test matching").unwrap();
        assert_eq!(m.number, None);
        assert_eq!(m.captures.len(), 2);
        assert_eq!(m.value, "some test line with test matching");
    }

    #[test]
    fn finding_matches_on_a_line_returns_none() {
        let reg = Regex::new(r"asdf").unwrap();
        let m = Matcher::new(&reg).match_line("some test line with test matching");
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
        let mut buf_read = Cursor::new("some text\nfn    main() {}\nhello");
        let matches = Matcher::new(&reg).collect(&mut buf_read).unwrap();
        assert_eq!(matches.path, None);
        assert_eq!(matches.count, 1);
        assert_eq!(matches.lines.len(), 1);
        assert!(reg.is_match(&matches.lines[0].value));
        assert!(matches.has_matches());
    }

    #[test]
    fn it_can_skip_line_numbers() {
        let reg = Regex::new(r"test").unwrap();
        let mut buf_read = Cursor::new("test\nnot\ntest");
        let matches = Matcher::new(&reg).with_line_numbers(false).collect(&mut buf_read).unwrap();
        assert_eq!(matches.count, 2);
        assert_eq!(matches.lines.len(), 2);
        assert_eq!(matches.lines[0].number, None);
        assert_eq!(matches.lines[1].number, None);
    }

    #[test]
    fn it_tracks_the_line_numbers_from_one() {
        let reg = Regex::new(r"test").unwrap();
        let mut buf_read = Cursor::new("test\nnot\ntest");
        let matches = Matcher::new(&reg).collect(&mut buf_read).unwrap();
        assert_eq!(matches.count, 2);
        assert_eq!(matches.lines.len(), 2);
        assert_eq!(matches.lines[0].number, Some(1));
        assert_eq!(matches.lines[1].number, Some(3));
    }

    #[test]
    fn finds_all_the_captures() {
        let reg = Regex::new(r"test").unwrap();
        let mut buf_read = Cursor::new("test a test b test");
        let matches = Matcher::new(&reg).collect(&mut buf_read).unwrap();
        assert!(matches.has_matches());
        assert_eq!(matches.lines[0].captures[0].value, "test".to_string());
    }

    #[test]
    fn skips_tracking_lines_and_captures() {
        let reg = Regex::new(r"test").unwrap();
        let mut buf_read = Cursor::new("test a test b test");
        let matches = Matcher::new(&reg).keep_lines(false).collect(&mut buf_read).unwrap();
        assert!(matches.has_matches());
        assert_eq!(matches.lines.len(), 0)
    }
}
