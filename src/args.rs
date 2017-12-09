use regex;
use regex::{Regex, RegexBuilder};

pub struct Opts {
    pub regex: Regex,
    pub queries: Option<Vec<String>>,
    pub is_concurrent: bool,
    pub is_colored: bool,
}

#[derive(Debug)]
pub enum ArgError {
    InvalidRegex(String),
    _Incomplete,
}

fn get_regex(regex: &str, case_insensitive: bool) -> Result<Regex, ArgError> {
    let regex = match RegexBuilder::new(&regex).case_insensitive(case_insensitive).build() {
        Ok(regex) => regex,
        Err(regex::Error::Syntax(msg)) => return Err(ArgError::InvalidRegex(msg)),
        Err(regex::Error::CompiledTooBig(_)) => return Err(ArgError::InvalidRegex("Regex too large".to_string())),
        Err(_) => return Err(ArgError::InvalidRegex("Unknown regex parsing error".to_string())),
    };
    Ok(regex)
}

pub fn get_opts() -> Result<Opts, ArgError> {
    use clap::{Arg, App};
    let matches = App::new("Grusp")
        .author("Kevin C. <chewbacha@gmail.com>; Charlie K")
        .about("Searches with regex through files. For fun!")
        .arg(Arg::with_name("case-sensitive")
            .long("case-sensitive")
            .short("s")
            .help("Regex is matched case sensitively"))
        .arg(Arg::with_name("ignore-case")
            .long("ignore-case")
            .short("i")
            .conflicts_with("case-sensitive")
            .help("Regex is matched case insensitively"))
        .arg(Arg::with_name("unthreaded")
            .long("unthreaded")
            .help("Runs in a single thread"))
        .arg(Arg::with_name("notcolored")
            .long("nocolor")
            .help("Output is not colored"))
        .arg(Arg::with_name("REGEX")
            .index(1)
            .value_name("REGEX")
            .required(true)
            .help("The pattern that should be matched"))
        .arg(Arg::with_name("PATTERN")
            .index(2)
            .multiple(true)
            .value_name("PATTERN")
            .help("The files to search"))
        .get_matches();

    let regex = matches.value_of("REGEX").expect("Regex required!");
    let is_colored = !matches.is_present("notcolored");
    let queries = matches.values_of("PATTERN").map(|queries| {
        queries.map(|p| p.to_owned()).collect()
    });
    let is_concurrent = !matches.is_present("unthreaded");
    let case_insensitive =
        matches.is_present("ignore-case") && !matches.is_present("case-sensitive");
    Ok(Opts { regex: get_regex(regex, case_insensitive)?, queries, is_concurrent, is_colored })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_into_a_regex() {
        let regex = get_regex("test", false).unwrap();
        assert!(regex.is_match("test"));
        assert!(!regex.is_match("tEst"));
    }

    #[test]
    fn it_errors_when_parsing_bad_regex() {
        let result = get_regex("test(", false);
        assert!(result.is_err());
    }

    #[test]
    fn it_can_be_case_insensitive() {
        let regex = get_regex("test", true).unwrap();
        assert!(regex.is_match("TEST"));
    }
}
