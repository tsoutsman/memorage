//! This module contains useful functions and macros for unit tests.

// TODO if someone can find a way to only export the macros below when running tests or compiling
// documentation that would be great - `#[cfg(any(test, doc))]` doesn't seem to work. In order to
// run their doctests, the macros are currently being exported publicly.

/// A macro that creates files and directories based on the given input.
/// # Example
/// ```
/// # use tempfile::tempdir;
/// # #[macro_use] extern crate oxalis;
/// # fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
/// let root_dir: std::path::PathBuf = tempdir()?.into_path();
/// gen_fs!(root_dir => (a: file1 file2) b c (another_directory: foo bar));
/// # Ok(())
/// # }
/// ```
/// would create a directory structure in the `tempdir` equivalent to the following:
/// ```shell
/// .
/// ├── a
/// │   ├── file1
/// │   └── file2
/// ├── another_directory
/// │   ├── bar
/// │   └── foo
/// ├── b
/// └── c
/// ```
#[macro_export]
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

/// A macro that creates a Vec<PathBuf> containing paths to all files specified in the input.
/// # Example
/// ```
/// # use tempfile::tempdir;
/// # #[macro_use] extern crate oxalis;
/// # fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
/// let root_dir: std::path::PathBuf = tempdir()?.into_path();
/// let expected = gen_expected!(root_dir => (a: file1 file2) b c (another_directory: foo bar));
/// assert_eq!(expected, vec![
///     root_dir.join("a").join("file1"),
///     root_dir.join("a").join("file2"),
///     root_dir.join("b"),
///     root_dir.join("c"),
///     root_dir.join("another_directory").join("foo"),
///     root_dir.join("another_directory").join("bar"),
/// ]);
/// # Ok(())
/// # }
#[macro_export]
macro_rules! gen_expected {
        // The first 2 patterns are only called the first time by the user. The tt is only munched
        // by the last 3 patterns.
        ($root:ident => $f:ident $($tail:tt)*) => {
            {
                #[allow(clippy::vec_init_then_push)]
                {
                    let mut z_macro = Vec::new();
                    gen_expected!(@$root | z_macro => $f $($tail)*);
                    z_macro
                }
            }
        };
        ($root:ident => ($dir:ident : $($inner:tt)*)$($tail:tt)*) => {
            {
                #[allow(clippy::vec_init_then_push)]
                {
                    let mut z_macro = Vec::new();
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
