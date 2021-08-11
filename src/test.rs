//! This module contains useful functions and macros for unit tests. When declaring this module as a
//! module (i.e. `mod test` in `main.rs` or `lib.rs`) make sure to enable the module only for tests
//! like so:
//! ```rust,ignore
//! #[cfg(test)]
//! mod test;
//! ```
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
pub(crate) use gen_fs;

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
pub(crate) use gen_expected;
