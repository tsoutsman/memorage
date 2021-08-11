use crate::error::{Error, Result};

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

#[allow(dead_code)]
pub fn changed_files<P: AsRef<Path>>(path: P, dur: Duration) -> Result<Vec<PathBuf>> {
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
        } else {
            let time_changed = entry.metadata()?.modified()?;

            if now - dur < time_changed {
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

    use crate::{gen_expected, gen_fs};

    #[test]
    fn test_changed_files() {
        let root_path = tempdir().unwrap().into_path();
        // Set 1
        gen_fs!(root_path => bar (x: temp foo));

        // The files were made in the last 0.5 secs so we expect to get everything.
        let mut expected = gen_expected!(root_path => bar (x: temp foo));
        expected.sort();
        let mut result = changed_files(&root_path, Duration::from_secs_f64(0.5)).unwrap();
        result.sort();
        assert_eq!(expected, result,);

        std::thread::sleep(Duration::from_secs_f64(0.5));

        // The files were made over 0.5 secs ago so we expect to get nothing.
        let expected: Vec<PathBuf> = Vec::new();
        let result = changed_files(&root_path, Duration::from_secs_f64(0.5)).unwrap();
        assert_eq!(expected, result);

        // Set 2
        gen_fs!(root_path => temp2 (y: alice bob));

        // Only set 2 was generated within the last 0.5 secs.
        let mut expected = gen_expected!(root_path => temp2 (y: alice bob));
        expected.sort();
        let mut result = changed_files(&root_path, Duration::from_secs_f64(0.5)).unwrap();
        result.sort();
        assert_eq!(expected, result);

        // Both sets of files were generated within the last 2 secs.
        let mut expected = gen_expected!(root_path => temp2 (y: alice bob) bar (x: temp foo));
        expected.sort();
        let mut result = changed_files(&root_path, Duration::from_secs_f64(2.)).unwrap();
        result.sort();
        assert_eq!(expected, result);
    }
}
