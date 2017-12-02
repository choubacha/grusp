use regex;
use regex::{Regex};

pub struct Opts {
    pub regex: Regex,
    pub queries: Vec<String>,
}

#[derive(Debug)]
pub enum ArgError {
    InvalidRegex(String),
    _Incomplete,
}

fn get_regex(regex: &str) -> Result<Regex, ArgError> {
    let regex = match Regex::new(&regex) {
        Ok(regex) => regex,
        Err(regex::Error::Syntax(msg)) => return Err(ArgError::InvalidRegex(msg)),
        Err(regex::Error::CompiledTooBig(_)) => return Err(ArgError::InvalidRegex("Regex too large".to_string())),
        Err(_) => return Err(ArgError::InvalidRegex("Unknown regex parsing error".to_string())),
    };
    Ok(regex)
}

pub fn get_opts() -> Result<Opts, ArgError> {
    let matches = clap_app!(myapp =>
        (author: "Kevin C. <chewbacha@gmail.com>")
        (about: "Searches with regex through files. For fun!")
        (@arg REGEX: +required "The pattern that should be matched")
        (@arg PATTERN: ... "The files to search")
    ).get_matches();

    let regex = matches.value_of("REGEX").expect("Regex required!");
    let queries = matches.values_of("PATTERN").unwrap().map(|p| p.to_owned()).collect();
    Ok(Opts { regex: get_regex(regex)?, queries: queries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_into_a_regex() {
        let regex = get_regex("test").unwrap();
        assert!(regex.is_match("test"));
    }

    #[test]
    fn it_errors_when_parsing_bad_regex() {
        let result = get_regex("test(");
        assert!(result.is_err());
    }
}