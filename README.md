# grusp
This is a very simple regex cli tool written in rust. It's not meant to replace grep
or ack or ag but simply be a test bed for developing and learning rust.

[![CircleCI](https://circleci.com/gh/kbacha/grusp.svg?style=svg)](https://circleci.com/gh/kbacha/grusp)

### Usage

```
$ grusp --help
Grusp
Kevin C. <chewbacha@gmail.com>; Charlie K. <bringking@gmail.com>
Searches with regex through files. For fun!

USAGE:
    grusp [FLAGS] [OPTIONS] <REGEX> [PATTERN]...

FLAGS:
    -s, --case-sensitive           Regex is matched case sensitively
    -c, --count                    Just counts the matches found
        --files-with-matches       Only print the names of files containing matches, not the matching lines. An empty
                                   query will print all files that would be searched.
        --files-without-matches    Only print the names of files not containing matches. An empty query will print no
                                   files.
    -h, --help                     Prints help information
    -i, --ignore-case              Regex is matched case insensitively
    -v, --invert-match             Match every line not containing the specified pattern
        --nocolor                  Output is not colored
        --unthreaded               Runs in a single thread
    -V, --version                  Prints version information

OPTIONS:
        --depth <NUM>    Search up to NUM directories deep

ARGS:
    <REGEX>         The pattern that should be matched. This can be any valid Perl-style
                    Regular expression, with a few caveats. See the Rust Regex documentation for detailed
                    information https://doc.rust-lang.org/regex/regex/index.html.
    <PATTERN>...    The files to search. This is optional and not used if grusp is searching from stdin
```

```
grusp 'fn.*\(' src/*.rs
src/args.rs matched 4 times
14:fn get_regex(regex: &str) -> Result<Regex, ArgError> {
24:pub fn get_opts() -> Result<Opts, ArgError> {
42:    fn it_parses_into_a_regex() {
48:    fn it_errors_when_parsing_bad_regex() {

src/display.rs matched 3 times
8:    pub fn new(matches: Matches) -> MatchDisplay {
14:    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
32:    fn it_formats_a_match() {

src/main.rs matched 1 times
10:fn main() {

src/matcher.rs matched 9 times
21:    pub fn has_matches(&self) -> bool {
25:    fn from_path(path: &Path) -> Self {
33:    fn add(&mut self, m: Match) {
40:    fn new(line: String, number: u32) -> Match {
45:pub fn find_matches(path: &Path, regex: &Regex) -> std::io::Result<Matches> {
73:    fn matches_knows_it_has_matches() {
82:    fn matches_tracks_count() {
91:    fn find_main_rs() {
92:        let reg = Regex::new(r"fn\s+main").unwrap();
```

### Contributing

Feel free to fork and propose changes as you see opportunities. PRs will be reviewed
and merged after they are approved.

### License

The MIT License (MIT)

Copyright (c) 2017-present Kevin Choubacha

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
