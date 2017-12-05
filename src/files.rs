use std::path::PathBuf;
use std::io::Result;

pub fn recurse(path: PathBuf) -> Result<Vec<PathBuf>>{
    if path.is_dir() {
        let mut files = Vec::new();
        let entries = path.read_dir()?;
        for entry in entries {
            recurse(entry?.path())?
                .into_iter()
                .for_each(|path| files.push(path.to_owned()));
        }

        Ok(files)
    } else {
        Ok(vec![path.to_owned()])
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::recurse;

    #[test]
    fn it_finds_files_inside_directory_if_path_is_dir() {
        let path = Path::new("./example_dir/sub_dir");
        let files = recurse(path.to_owned()).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&Path::new("./example_dir/sub_dir/sub-example-1.txt").to_owned()));
        assert!(!files.contains(&Path::new("./example_dir/example-1.txt").to_owned()));
    }

    #[test]
    fn it_finds_files_inside_sub_directories() {
        let path = Path::new("./example_dir");
        let files = recurse(path.to_owned()).unwrap();
        assert_eq!(files.len(), 4);
        assert!(files.contains(&Path::new("./example_dir/sub_dir/sub-example-1.txt").to_owned()));
        assert!(files.contains(&Path::new("./example_dir/example-1.txt").to_owned()));
    }

    #[test]
    fn it_returns_current_file_if_a_file_is_passed_in() {
        let path = Path::new("./example_dir/example-1.txt");
        let files = recurse(path.to_owned()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files, &[path]);
    }
}