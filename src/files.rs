use std::path::PathBuf;
use std::io::Result;
use glob::glob;

pub struct Collecter<'a> {
    queries: &'a Vec<String>,
    max_depth: Option<usize>,
}

impl<'a> Collecter<'a> {
    pub fn new(queries: &'a Vec<String>) -> Self {
        Self { queries: &queries, max_depth: None }
    }

    pub fn max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn collect(self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for query in self.queries {
            glob(&query)
                .expect("Glob pattern failed")
                .filter(|p| p.is_ok())
                .map(|p| p.expect("An 'ok' file was not found"))
                .for_each(|p| {
                    self.recurse(p, &mut files, 0).expect("Unknown file error")
                });
        }
        files
    }

    fn recurse(&self, path: PathBuf, files: &mut Vec<PathBuf>, depth: usize) -> Result<()> {
        if path.is_dir() {
            if let Some(max_depth) = self.max_depth {
                if max_depth < depth { return Ok(()); };
            }

            let entries = path.read_dir()?;
            for entry in entries {
                self.recurse(entry?.path(), files, depth + 1)?
            }
            Ok(())
        } else {
            files.push(path.to_owned());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn it_globs_down_a_directory() {
        let queries = vec!["./example_dir/**/*.txt".to_string()];
        let files = Collecter::new(&queries).collect();

        assert_eq!(files.len(), 4);
        assert!(files.contains(
            &Path::new("example_dir/sub_dir/sub-example-1.txt").to_owned(),
        ));
    }

    #[test]
    fn it_finds_files_inside_directory_if_path_is_dir() {
        let query = vec!["./example_dir/sub_dir".to_string()];
        let files = Collecter::new(&query).collect();

        assert_eq!(files.len(), 2);
        assert!(files.contains(
            &Path::new("example_dir/sub_dir/sub-example-1.txt").to_owned(),
        ));
        assert!(!files.contains(
            &Path::new("example_dir/example-1.txt").to_owned(),
        ));
    }

    #[test]
    fn it_finds_files_inside_sub_directories() {
        let query = vec!["./example_dir".to_string()];
        let files = Collecter::new(&query).collect();

        assert_eq!(files.len(), 4);
        assert!(files.contains(
            &Path::new("example_dir/sub_dir/sub-example-1.txt").to_owned(),
        ));
        assert!(files.contains(
            &Path::new("example_dir/example-1.txt").to_owned(),
        ));
    }

    #[test]
    fn it_returns_current_file_if_a_file_is_passed_in() {
        let query = vec!["./example_dir/example-1.txt".to_string()];
        let files = Collecter::new(&query).collect();
        assert_eq!(files.len(), 1);

        let path = Path::new("example_dir/example-1.txt");
        assert_eq!(files, &[path]);
    }

    #[test]
    fn it_can_restrict_the_depth() {
        let query = vec!["./example_dir".to_string()];
        let files = Collecter::new(&query).max_depth(Some(0)).collect();

        assert_eq!(files.len(), 2);
        assert!(!files.contains(
            &Path::new("example_dir/sub_dir/sub-example-1.txt").to_owned(),
        ));
        assert!(files.contains(
            &Path::new("example_dir/example-1.txt").to_owned(),
        ));
    }
}
