use matcher::{Matches, Match};
use std::fmt;
use colored::*;

// MatchDisplay to format a single Match
pub struct MatchDisplay<'a> {
    match_to_display: &'a Match,
    is_colored: bool,
}
// MatchesDisplay for format a result set of MatchDisplay
pub struct MatchesDisplay {
    matches: Matches,
    is_colored: bool,
    is_count_only: bool,
}

impl<'a> MatchDisplay<'a> {
    fn prefix_fmt(&self) -> String {
        if self.is_colored {
            self.match_to_display
                .number
                .to_string()
                .yellow()
                .to_string()
        } else {
            self.match_to_display.number.to_string()
        }
    }

    fn line_fmt(&self) -> String {
        let line = &*self.match_to_display.line;

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

    pub fn new(match_to_display: &'a Match, parent: &MatchesDisplay) -> MatchDisplay<'a> {
        MatchDisplay {
            match_to_display: match_to_display,
            is_colored: parent.is_colored,
        }
    }
}

impl MatchesDisplay {
    pub fn new(matches: Matches) -> MatchesDisplay {
        MatchesDisplay {
            matches: matches,
            is_colored: true,
            is_count_only: false,
        }
    }
    pub fn color(self, is_colored: bool) -> Self {
        Self { is_colored, ..self }
    }
    pub fn count_only(self, is_count_only: bool) -> Self {
        Self { is_count_only, ..self }
    }
}

impl<'a> fmt::Display for MatchDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.prefix_fmt(), self.line_fmt())?;
        Ok(())
    }
}

impl fmt::Display for MatchesDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref path) = self.matches.path {
            let path = path.as_path().to_str().unwrap_or("");
            if self.is_colored {
                write!(f, "{} ", path.bright_green())?;
            } else {
                write!(f, "{} ", path)?;
            };
        }
        if self.is_colored {
            write!(
                f,
                "matched {} time",
                self.matches.count.to_string().yellow()
            )?;
        } else {
            write!(f, "matched {} time", self.matches.count.to_string())?;
        }
        if self.matches.count > 1 { write!(f, "s")?; }

        if !self.is_count_only {
            writeln!(f, "")?;
            for m in &self.matches.matches {
                writeln!(f, "{}", MatchDisplay::new(m, &self))?;
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use matcher::{Matches, Match, Capture};
    use std::path::Path;
    use super::*;

    #[test]
    fn it_formats_a_match_with_just_counts() {
        let m = Matches {
            count: 12,
            path: Some(Path::new("./path/to/something").to_owned()),
            matches: vec![
                Match {
                    number: 23,
                    line: "some text line".to_string(),
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
            matches: Vec::new(),
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
            matches: vec![
                Match {
                    number: 23,
                    line: "some text line".to_string(),
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
            matches: vec![
                Match {
                    number: 23,
                    line: "some text line".to_string(),
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
            matches: vec![
                Match {
                    number: 23,
                    line: "some text line".to_string(),
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
}
