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
        writeln!(f, "{}:{}", self.match_to_display.number.to_string().bright_yellow(), self.match_to_display.line.trim_right())?;
        Ok(())
    }
}

impl fmt::Display for MatchesDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} matched {} times",
                 self.matches.path.as_path().to_str().unwrap().bright_green(),
                 self.matches.count.to_string().bright_yellow())?;

        for m in &self.matches.matches {
            write!(f, "{}", MatchDisplay::new(m));
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use matcher::{Matches, Match};
    use std::path::Path;
    use super::*;

    #[test]
    fn it_formats_a_match() {
        let m = Matches {
            count: 12,
            path: Path::new("./path/to/something").to_owned(),
            matches: vec![Match { number: 23, line: "some text line".to_string() }],
        };
        assert_eq!(
            format!("{}", MatchDisplay::new(m)),
            "./path/to/something matched 12 times\n23:some text line\n"
        )
    }
}
