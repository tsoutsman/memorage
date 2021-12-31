use crate::error::{Error, Result};

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

/// Returns the files that have been changed in a directory within a given timeframe.
///
/// # Example
/// ```rust
/// # use soter_client::fs::monitor::changed_files;
/// # use std::time::Duration;
/// # use std::fs::File;
/// # use tempfile::tempdir;
/// # fn main() -> soter_client::Result<()> {
/// # let root_path = tempdir().unwrap().into_path();
/// let file_path = root_path.join("foo");
/// File::create(file_path.clone());
///
/// let changed_files = changed_files(root_path, Duration::from_secs(1))?.collect::<Vec<_>>();
///
/// assert_eq!(changed_files, vec![file_path]);
/// # Ok(())
/// # }
pub fn changed_files<P: AsRef<Path>>(
    path: P,
    dur: Duration,
) -> Result<impl Iterator<Item = PathBuf>> {
    let dir = path.as_ref();
    let mut result = Vec::new();

    let now = SystemTime::now();

    if !dir.is_dir() {
        return Err(Error::NotDirectory(dir.to_owned()));
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            // Recurse into the child directory.
            result.extend(changed_files(entry.path(), dur)?);
        } else if entry.file_type()?.is_file() {
            let time_changed = entry.metadata()?.modified()?;

            if now - dur < time_changed {
                result.push(entry.path());
            }
        }
    }

    Ok(result.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use efes::{gen_fs, gen_paths};
    use tempfile::tempdir;

    #[test]
    fn test_changed_files() {
        let root_path = tempdir().unwrap().into_path();
        // Set 1
        gen_fs!(root_path => bar (x: temp foo));

        // The files were made in the last 0.5 secs so we expect to get everything.
        let mut expected = gen_paths!(root_path => bar (x: temp foo));
        expected.sort();
        let mut result = changed_files(&root_path, Duration::from_secs_f64(0.5))
            .unwrap()
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(expected, result,);

        std::thread::sleep(Duration::from_secs_f64(0.5));

        // The files were made over 0.5 secs ago so we expect to get nothing.
        let expected: Vec<PathBuf> = Vec::new();
        let result = changed_files(&root_path, Duration::from_secs_f64(0.5))
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(expected, result);

        // Set 2
        gen_fs!(root_path => temp2 (y: alice bob));

        // Only set 2 was generated within the last 0.5 secs.
        let mut expected = gen_paths!(root_path => temp2 (y: alice bob));
        expected.sort();
        let mut result = changed_files(&root_path, Duration::from_secs_f64(0.5))
            .unwrap()
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(expected, result);

        // Both sets of files were generated within the last 2 secs.
        let mut expected = gen_paths!(root_path => temp2 (y: alice bob) bar (x: temp foo));
        expected.sort();
        let mut result = changed_files(&root_path, Duration::from_secs_f64(2.))
            .unwrap()
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_changed_files_not_directory() {
        let root_path = tempdir().unwrap().into_path();
        gen_fs!(root_path => foo);

        match changed_files(root_path.join("foo"), Duration::ZERO) {
            Err(Error::NotDirectory(p)) => assert_eq!(p, root_path.join("foo")),
            _ => panic!(),
        }
    }
}
