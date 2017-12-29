use matcher::{Matches, Line};
use std::fmt;
use colored::*;

/// MatchDisplay to format a single Match
#[derive(Debug)]
pub struct LineDisplay<'a> {
    match_to_display: &'a Line,
    is_colored: bool,
}

/// A struct used to wrap the matches that are found and then
/// display them to a command line interface. It follows a builder pattern
/// to allow setting things like `is_colored` and `is_count_only`.
#[derive(Debug)]
pub struct MatchesDisplay {
    matches: Matches,
    is_colored: bool,
    is_count_only: bool,
    just_file_names: bool,
}

impl<'a> LineDisplay<'a> {
    fn prefix_fmt(&self) -> Option<String> {
        self.match_to_display.number.map(|line_number| {
            if self.is_colored {
                line_number.to_string().yellow().to_string()
            } else {
                line_number.to_string()
            }
        })
    }

    fn line_fmt(&self) -> String {
        let line = &*self.match_to_display.value;

        if self.is_colored {
            let mut output = String::new();
            let mut prev_end = 0;
            for cap in &self.match_to_display.captures {
                output.push_str(&line[prev_end..cap.start]);
                output.push_str(&cap.value.black().on_yellow().to_string());
                prev_end = cap.end;
            }
            output.push_str(&line[prev_end..]);
            output.trim_right().to_string()
        } else {
            line.trim_right().to_string()
        }
    }

    pub fn new(match_to_display: &'a Line, parent: &MatchesDisplay) -> LineDisplay<'a> {
        LineDisplay {
            match_to_display: match_to_display,
            is_colored: parent.is_colored,
        }
    }
}

impl MatchesDisplay {
    /// So that you can configure how a set of matches should be displayed, you
    /// can use this wrapper struct. It consumes a `Matches` struct and returns
    /// a display struct. Use the builder functions to configure.
    pub fn new(matches: Matches) -> MatchesDisplay {
        MatchesDisplay {
            matches: matches,
            is_colored: true,
            is_count_only: false,
            just_file_names: false,
        }
    }

    /// Consumes self and enables/disables displaying just the file names
    pub fn just_file_names(self, just_file_names: bool) -> Self {
        Self { just_file_names, ..self }
    }

    /// Consumes the display and enables/disables colored output.
    pub fn color(self, is_colored: bool) -> Self {
        Self { is_colored, ..self }
    }

    /// Consumes the display and enables showing just the counts.
    pub fn count_only(self, is_count_only: bool) -> Self {
        Self { is_count_only, ..self }
    }
}

impl<'a> fmt::Display for LineDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(prefix) = self.prefix_fmt() {
            write!(f, "{}:{}", prefix, self.line_fmt())?;
        } else {
            write!(f, "{}", self.line_fmt())?;
        }
        Ok(())
    }
}

impl fmt::Display for MatchesDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = String::new();

        if let Some(ref path) = self.matches.path {
            let path = path.as_path().to_str().unwrap_or("");
            if self.is_colored {
                ret.push_str(&format!("{} ", path.bright_green()));
            } else {
                ret.push_str(&format!("{} ", path));
            };
        }
        if self.just_file_names {
            return write!(f, "{}", ret.trim())
        }
        if self.is_colored {
            ret.push_str(&format!("matched {} time", self.matches.count.to_string().yellow()));
        } else {
            ret.push_str(&format!("matched {} time", self.matches.count.to_string()));
        }
        if self.matches.count > 1 { ret.push_str("s"); }
        write!(f, "{}", ret)?;

        if !self.is_count_only {
            writeln!(f, "")?;
            for m in &self.matches.lines {
                writeln!(f, "{}", LineDisplay::new(m, &self))?;
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use matcher::{Matches, Line, Capture};
    use std::path::Path;
    use super::*;

    #[test]
    fn it_formats_a_match_with_just_counts() {
        let m = Matches {
            count: 12,
            path: Some(Path::new("./path/to/something").to_owned()),
            lines: vec![
                Line {
                    number: Some(23),
                    value: "some text line".to_string(),
                    captures: vec![
                        Capture {
                            start: 5,
                            end: 9,
                            value: "text".to_string(),
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m).count_only(true).color(false)),
            format!(
                "{path} matched {count} times",
                path = "./path/to/something".to_string(),
                count = 12.to_string(),
            )
        )
    }

    #[test]
    fn it_formats_a_match_with_just_count_but_single_time() {
        let m = Matches {
            count: 1,
            path: Some(Path::new("./path/to/something").to_owned()),
            lines: Vec::new(),
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m).count_only(true).color(false)),
            format!(
                "{path} matched {count} time",
                path = "./path/to/something",
                count = "1",
            )
        )
    }

    #[test]
    fn it_formats_a_match_without_color() {
        let m = Matches {
            count: 12,
            path: Some(Path::new("./path/to/something").to_owned()),
            lines: vec![
                Line {
                    number: Some(23),
                    value: "some text line".to_string(),
                    captures: vec![
                        Capture {
                            start: 5,
                            end: 9,
                            value: "text".to_string(),
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m).color(false)),
            format!(
                "{path} matched {count} times\n{line_number}:some {capture} line\n",
                path = "./path/to/something".to_string(),
                count = 12.to_string(),
                line_number = 23.to_string(),
                capture = "text".to_string()
            )
        )
    }

    #[test]
    fn it_formats_a_match() {
        let m = Matches {
            count: 12,
            path: Some(Path::new("./path/to/something").to_owned()),
            lines: vec![
                Line {
                    number: Some(23),
                    value: "some text line".to_string(),
                    captures: vec![
                        Capture {
                            start: 5,
                            end: 9,
                            value: "text".to_string(),
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m)),
            format!(
                "{path} matched {count} times\n{line_number}:some {capture} line\n",
                path = "./path/to/something".to_string().bright_green(),
                count = 12.to_string().yellow(),
                line_number = 23.to_string().yellow(),
                capture = "text".to_string().black().on_yellow()
            )
        )
    }

    #[test]
    fn it_formats_a_match_without_a_path() {
        let m = Matches {
            count: 12,
            path: None,
            lines: vec![
                Line {
                    number: Some(23),
                    value: "some text line".to_string(),
                    captures: vec![
                        Capture {
                            start: 5,
                            end: 9,
                            value: "text".to_string(),
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m)),
            format!(
                "matched {count} times\n{line_number}:some {capture} line\n",
                count = 12.to_string().yellow(),
                line_number = 23.to_string().yellow(),
                capture = "text".to_string().black().on_yellow()
            )
        )
    }

    #[test]
    fn it_prints_just_matching_files() {
        let m = Matches {
            count: 12,
            path: Some(Path::new("./path/to/something").to_owned()),
            lines: vec![
                Line {
                    number: Some(23),
                    value: "some text line".to_string(),
                    captures: vec![
                        Capture {
                            start: 5,
                            end: 9,
                            value: "text".to_string(),
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m).just_file_names(true)),
            format!("{path}", path = "./path/to/something".to_string().bright_green())
        );
    }

    #[test]
    fn it_prints_nothing_when_just_file_names_but_no_path() {
        let m = Matches {
            count: 12,
            path: None,
            lines: vec![
                Line {
                    number: Some(23),
                    value: "some text line".to_string(),
                    captures: vec![
                        Capture {
                            start: 5,
                            end: 9,
                            value: "text".to_string(),
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m).just_file_names(true)),
            ""
        );
    }
}
