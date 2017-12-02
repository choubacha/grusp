use matcher::Matches;
use std::fmt;

pub struct MatchDisplay {
    matches: Matches
}

impl MatchDisplay {
    pub fn new(matches: Matches) -> MatchDisplay {
        MatchDisplay { matches: matches }
    }
}

impl fmt::Display for MatchDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} matched {} times",
                 self.matches.path.as_path().to_str().unwrap(),
                 self.matches.count)?;
        for m in &self.matches.matches {
            writeln!(f, "{}:{}", m.number, m.line.trim_right())?;
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
