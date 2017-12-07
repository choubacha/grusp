use matcher::{Matches, Match};
use std::fmt;
use colored::*;

// MatchDisplay to format a single Match
pub struct MatchDisplay<'a> {
    match_to_display: &'a Match
}
// MatchesDisplay for format a result set of MatchDisplay
pub struct MatchesDisplay {
    matches: Matches
}

impl<'a> MatchDisplay<'a> {
    fn prefix_fmt(&self) -> ColoredString {
        self.match_to_display.number.to_string().yellow()
    }

    fn line_fmt(&self) -> String {
        let mut output = String::new();
        let line = &*self.match_to_display.line;

        let mut prev_end = 0;
        for cap in &self.match_to_display.captures {
            output.push_str(&line[prev_end..cap.start]);
            output.push_str(&cap.value.black().on_yellow().to_string());
            prev_end = cap.end;
        }
        output.push_str(&line[prev_end..]);

        output.trim_right().to_string()
    }

    pub fn new(match_to_display: &'a Match) -> MatchDisplay {
        MatchDisplay { match_to_display: match_to_display }
    }
}

impl MatchesDisplay {
    pub fn new(matches: Matches) -> MatchesDisplay {
        MatchesDisplay { matches: matches }
    }
}

impl<'a> fmt::Display for MatchDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}:{}", self.prefix_fmt(), self.line_fmt())?;
        Ok(())
    }
}

impl fmt::Display for MatchesDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref path) = self.matches.path {
            write!(f, "{} ", path.as_path().to_str().unwrap_or("").bright_green())?;
        }
        writeln!(f, "matched {} times", self.matches.count.to_string().yellow())?;

        for m in &self.matches.matches {
            write!(f, "{}", MatchDisplay::new(m))?;
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
    fn it_formats_a_match() {
        let m = Matches {
            count: 12,
            path: Some(Path::new("./path/to/something").to_owned()),
            matches: vec![Match { number: 23, line: "some text line".to_string(), captures: vec![Capture { start: 5, end: 9, value: "text".to_string() }]  }],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m)),
            format!("{path} matched {count} times\n{line_number}:some {capture} line\n",
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
            matches: vec![Match { number: 23, line: "some text line".to_string(), captures: vec![Capture { start: 5, end: 9, value: "text".to_string() }]  }],
        };
        assert_eq!(
            format!("{}", MatchesDisplay::new(m)),
            format!("matched {count} times\n{line_number}:some {capture} line\n",
                    count = 12.to_string().yellow(),
                    line_number = 23.to_string().yellow(),
                    capture = "text".to_string().black().on_yellow()
            )
        )
    }
}
