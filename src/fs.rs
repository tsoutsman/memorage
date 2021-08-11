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

    macro_rules! gen_fs {
        ($root:ident => ) => {};
        ($root:ident => $f:ident $($tail:tt)*) => {
            ::std::fs::File::create($root.join(stringify!($f))).unwrap();
            gen_fs!($root => $($tail)*);
        };
        ($root:ident => ($dir:ident : $($inner:tt)*)$($tail:tt)*) => {
            ::std::fs::create_dir($root.join(stringify!($dir))).unwrap();
            let $dir = $root.join(stringify!($dir));
            gen_fs!($dir => $($inner)*);
            gen_fs!($root => $($tail)*);
        };
    }

    macro_rules! gen_expected {
        // The first 2 patterns are only called the first time by the user. The tt is only munched
        // by the last 3 patterns.
        ($root:ident => $f:ident $($tail:tt)*) => {
            {
                #[allow(clippy::vec_init_then_push)]
                {
                    let mut z_macro: Vec<PathBuf> = Vec::new();
                    gen_expected!(@$root | z_macro => $f $($tail)*);
                    z_macro
                }
            }
        };
        ($root:ident => ($dir:ident : $($inner:tt)*)$($tail:tt)*) => {
            {
                #[allow(clippy::vec_init_then_push)]
                {
                    let mut z_macro: Vec<PathBuf> = Vec::new();
                    gen_expected!(@$root | z_macro => ($dir : $($inner)*) $($tail)*);
                    z_macro
                }
            }
        };
        (@$root:ident | $vec:ident => ) => {};
        (@$root:ident | $vec:ident => $f:ident $($tail:tt)*) => {
            $vec.push($root.join(stringify!($f)));
            gen_expected!(@$root | $vec => $($tail)*);
        };
        (@$root:ident | $vec:ident => ($dir:ident : $($inner:tt)*)$($tail:tt)*) => {
            let $dir = $root.join(stringify!($dir));
            gen_expected!(@$dir | $vec => $($inner)*);
            gen_expected!(@$root | $vec => $($tail)*);
        };
    }

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
