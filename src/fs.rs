use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

#[allow(dead_code)]
pub fn changed_files<P: AsRef<Path>>(path: P, dur: Duration) -> io::Result<Vec<PathBuf>> {
    let dir = path.as_ref();
    let mut result = Vec::new();

    let now = SystemTime::now();

    if !dir.is_dir() {
        // TODO return custom error
        panic!("path is not a dir");
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            // Recurse into the child directory.
            result.extend(changed_files(entry.path(), dur)?);
        } else {
            let time_changed = entry.metadata()?.modified()?;

            if now - dur > time_changed {
                result.push(entry.path());
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_changed_files() {
        // TODO
        // The setup in this test is very janky. It will definetly be fixed.
        let root_path = tempdir().unwrap().into_path();

        let files = vec!["one", "two"];
        let directories = vec![("temp-directory", vec!["foo", "bar"])];

        for file in files {
            let file_path = root_path.join(file);
            fs::File::create(file_path).unwrap();
        }
        for (dir, children) in directories {
            let dir_path = root_path.join(dir);
            fs::create_dir(dir_path.clone()).unwrap();

            for file in children {
                let file_path = dir_path.join(file);
                fs::File::create(file_path).unwrap();
            }
        }

        let expected: Vec<PathBuf> = Vec::new();
        // Actual testing
        assert_eq!(
            expected,
            changed_files(root_path.clone(), Duration::from_secs(1)).unwrap()
        );

        std::thread::sleep(Duration::from_secs(1));

        let mut expected: Vec<PathBuf> = vec![
            root_path.join("one"),
            root_path.join("two"),
            root_path.join("temp-directory").join("foo"),
            root_path.join("temp-directory").join("bar"),
        ];
        expected.sort();

        let mut result = changed_files(root_path, Duration::from_secs(1)).unwrap();
        result.sort();
        assert_eq!(expected, result);
    }
}
