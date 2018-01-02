use regex;
use regex::{Regex, RegexBuilder};
use atty;
use atty::Stream;
use clap::{Values, Arg, App, AppSettings};

pub struct Opts {
    pub regex: Regex,
    pub queries: Option<Vec<String>>,
    pub is_count_only: bool,
    pub is_concurrent: bool,
    pub is_colored: bool,
    pub is_inverted: bool,
    pub max_depth: Option<usize>,
    pub just_files: JustFiles,
}

#[derive(Eq, PartialEq)]
pub enum JustFiles {
    WithMatches,
    WithoutMatches,
    None,
}

impl JustFiles {
    #[inline]
    pub fn without_matches(&self) -> bool {
        *self == JustFiles::WithoutMatches
    }

    #[inline]
    pub fn show_matches(&self) -> bool {
        *self != JustFiles::WithoutMatches
    }

    pub fn is_some(&self) -> bool {
        *self != JustFiles::None
    }
}

#[derive(Debug)]
pub enum ArgError {
    InvalidRegex(String),
    _Incomplete,
}

fn get_regex(regex: &str, case_insensitive: bool) -> Result<Regex, ArgError> {
    let regex = match RegexBuilder::new(&regex)
        .case_insensitive(case_insensitive)
        .build() {
        Ok(regex) => regex,
        Err(regex::Error::Syntax(msg)) => return Err(ArgError::InvalidRegex(msg)),
        Err(regex::Error::CompiledTooBig(_)) => {
            return Err(ArgError::InvalidRegex("Regex too large".to_string()))
        }
        Err(_) => {
            return Err(ArgError::InvalidRegex(
                "Unknown regex parsing error".to_string(),
            ))
        }
    };
    Ok(regex)
}

fn collect_queries(values: Option<Values>) -> Option<Vec<String>> {
    values
        .map(|queries| { queries.map(|p| p.to_owned()).collect() })
        .or_else(|| {
            if atty::is(Stream::Stdin) {
                // Search in current directory if it's a TTY
                Some(vec![".".to_string()])
            } else {
                None // There is probably a pipe stream coming in
            }
        })
}

const EXAMPLES: &'static str = "EXAMPLES:

- Use grusp to search from STDIN

    $ history | grusp docker

- Find all instances of the literal string 'fn' in the current directory

    $ grusp fn .

- Find all integers between 0-9 in files in the current directory

    $ grusp [0-9] .

- Find all strings that have 'fn' with a open parentheses and everything between in the current directory

    $ grusp \'fn.*\\(\' .

- Find all strings that have 'fn' or 'FN', ignoring case. This option is incompatible with '-s'.

    $ grusp -i fn .

- Find all strings that have 'fn' only with strict case. This option is incompatible with '-i'.

    $ grusp -s fn .

- Find all strings that have 'fn', using un-colored output. This can be used for an extremely small
speed boost, or compatibility with terminals without ANSI Color support.

    $ grusp --nocolor fn .

- Find all strings that have 'fn', run on a single thread. By default grusp will attempt to use multiple
threads to speed up the search process. If this is un-desired in your environment, set the --unthreaded flag

    $ grusp --unthreaded fn .
";

pub fn get_opts() -> Result<Opts, ArgError> {
    let matches = App::new("Grusp")
        .setting(AppSettings::ArgRequiredElseHelp)
        .after_help(EXAMPLES)
        .author("Kevin C. <chewbacha@gmail.com>; Charlie K. <bringking@gmail.com>")
        .about("Searches with regex through files. For fun!")
        .arg(
            Arg::with_name("case-sensitive")
                .long("case-sensitive")
                .short("s")
                .help("Regex is matched case sensitively"),
        )
        .arg(
            Arg::with_name("ignore-case")
                .long("ignore-case")
                .short("i")
                .conflicts_with("case-sensitive")
                .help("Regex is matched case insensitively"),
        )
        .arg(Arg::with_name("count").short("c").long("count").help(
            "Just counts the matches found",
        ))
        .arg(Arg::with_name("unthreaded").long("unthreaded").help(
            "Runs in a single thread",
        ))
        .arg(Arg::with_name("notcolored").long("nocolor").help(
            "Output is not colored",
        ))
        .arg(Arg::with_name("invert-match").long("invert-match").short("v").help(
            "Match every line not containing the specified pattern"
        ))
        .arg(Arg::with_name("files-with-matches").long("files-with-matches").help(
            "Only print the names of files containing matches, not the matching lines. An empty query will print all files that would be searched.",
        ))
        .arg(
            Arg::with_name("files-without-matches")
                .long("files-without-matches")
                .conflicts_with("files-with-matches")
                .help(
                "Only print the names of files not containing matches. An empty query will print no files.",
                )
        )
        .arg(
            Arg::with_name("depth")
                .takes_value(true)
                .value_name("NUM")
                .long("depth")
                .help("Search up to NUM directories deep")
        )
        .arg(
            Arg::with_name("REGEX")
                .index(1)
                .value_name("REGEX")
                .required(true)
                .help("The pattern that should be matched. This can be any valid Perl-style
Regular expression, with a few caveats. See the \
Rust Regex documentation \
for detailed information https://doc.rust-lang.org/regex/regex/index.html."),
        )
        .arg(
            Arg::with_name("PATTERN")
                .index(2)
                .multiple(true)
                .value_name("PATTERN")
                .help("The files to search. This is optional and not used if grusp is searching from stdin"),
        )
        .get_matches();

    let regex = matches.value_of("REGEX").expect("Regex required!");
    let is_colored = !matches.is_present("notcolored");
    let queries = collect_queries(matches.values_of("PATTERN"));
    let is_concurrent = !matches.is_present("unthreaded");
    let case_insensitive = matches.is_present("ignore-case") &&
        !matches.is_present("case-sensitive");
    let is_count_only = matches.is_present("count");
    let max_depth: Option<usize> = matches.value_of("depth").map(|v| v.parse().expect("Depth must be an valid integer"));
    let just_files = if matches.is_present("files-with-matches") {
        JustFiles::WithMatches
    } else if matches.is_present("files-without-matches") {
        JustFiles::WithoutMatches
    } else {
        JustFiles::None
    };
    let is_inverted = matches.is_present("invert-match");
    Ok(Opts {
        regex: get_regex(regex, case_insensitive)?,
        queries,
        is_concurrent,
        is_colored,
        is_count_only,
        max_depth,
        just_files,
        is_inverted,
    })
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
