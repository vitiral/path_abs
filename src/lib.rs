/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Absolute serializable path types and associated methods and better errors.
//!
//! ## Better Errors
//!
//! `set_len`:
//! - [`std::fs::File::set_len(0)     `][file_set_len]: `Invalid argument (os error 22)`
//! - [`path_abs::PathOpen::set_len(0)`][path_set_len]: `Invalid argument (os error 22) when setting
//!   len for /path/to/example/foo.txt`
//!
//! `read` (open file for reading):
//! - [`std::fs::File::read(path)     `][file_read]: `No such file or directory (os error 2)`
//! - [`path_abs::PathOpen::read(path)`][path_read]: `No such file or directory (os error 2) when
//!   opening example/foo.txt`
//!
//! And every other method. If a method does not have pretty error messages please open a ticket.
//!
//! ## Exported Types
//!
//! This library provides the following types:
//! - [`PathArc`](struct.PathArc.html): a reference counted `PathBuf` with methods reimplemented
//!   with better error messages. Use this for a "generic serialized path".
//! - [`PathAbs`](struct.PathAbs.html): a reference counted absolute (canonicalized) path that is
//!   guaranteed (when created) to exist.
//! - [`PathFile`](struct.PathFile.html): a `PathAbs` that is guaranteed to be a file, with
//!   associated methods.
//! - [`PathDir`](struct.PathDir.html): a `PathAbs` that is guaranteed to be a directory, with
//!   associated methods.
//! - [`PathType`](struct.PathType.html): an enum containing either a file or a directory. Returned
//!   by `PathDir::list`.
//! - [`PathOpen`](struct.PathOpen.html): an open file with the `path()` attached and error
//!   messages which include the path information.
//!
//! In addition, all types (expect `PathOpen`) are serializable through serde (even on windows!) by
//! using the crate [`stfu8`](https://crates.io/crates/stfu8) to encode/decode, allowing ill-formed
//! UTF-16. See that crate for more details on how the resulting encoding can be edited (by hand)
//! even in the case of what *would be* ill-formed UTF-16.
//!
//! Also see the [project repo](https://github.com/vitiral/path_abs) and consider leaving a star!
//!
//! > All types are internally `Arc<PathBuf>` so they are extremely cheap to clone. When working
//! > with paths a reference count is NOT an expensive operation for you!
//!
//! [file_set_len]: https://doc.rust-lang.org/std/fs/struct.File.html#method.set_len
//! [file_read]: https://doc.rust-lang.org/std/fs/struct.File.html#method.read
//! [path_set_len]: struct.PathOpen.html#method.set_len)
//! [path_read]: struct.PathOpen.html#method.read)
//!
//! # Examples
//! Recreating `Cargo.init` in `target/example`
//!
//! ```rust
//! # extern crate path_abs;
//! use std::collections::HashSet;
//! use path_abs::{PathAbs, PathDir, PathFile, PathType};
//!
//! # fn main() {
//!
//! let example = "target/example";
//!
//! # let _ = ::std::fs::remove_dir_all(example);
//!
//! // Create your paths
//! let project = PathDir::create_all(example).unwrap();
//! let src = PathDir::create(project.join("src")).unwrap();
//! let lib = PathFile::create(src.join("lib.rs")).unwrap();
//! let cargo = PathFile::create(project.join("Cargo.toml")).unwrap();
//!
//! // Write the templates
//! lib.write_str(r#"
//! #[cfg(test)]
//! mod tests {
//!     #[test]
//!     fn it_works() {
//!         assert_eq!(2 + 2, 4);
//!     }
//! }"#).unwrap();
//!
//! cargo.write_str(r#"
//! [package]
//! name = "example"
//! version = "0.1.0"
//! authors = ["Garrett Berg <googberg@gmail.com>"]
//!
//! [dependencies]
//! "#).unwrap();
//!
//! let mut result = HashSet::new();
//! for p in project.list().unwrap() {
//!     result.insert(p.unwrap());
//! }
//!
//! let mut expected = HashSet::new();
//! expected.insert(PathType::Dir(src));
//! expected.insert(PathType::File(cargo));
//!
//! assert_eq!(expected, result);
//!
//! // Get a file
//! let abs = PathAbs::new("target/example/src/lib.rs").unwrap();
//!
//! // or get the file of known type
//! let file = PathType::new("target/example/src/lib.rs")
//!     .unwrap()
//!     .unwrap_file();
//!
//! // or use `into_file`
//! let file2 = abs.clone().into_file().unwrap();
//!
//! assert!(abs.is_file());
//! assert!(file.is_file());
//! assert!(file2.is_file());
//! # }
//! ```

#[cfg(feature = "serialize")]
extern crate serde;
#[macro_use]
#[cfg(feature = "serialize")]
extern crate serde_derive;
#[cfg(feature = "serialize")]
extern crate stfu8;

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate regex;
#[cfg(test)]
extern crate serde_json;
#[cfg(test)]
extern crate tempdir;

mod abs;
mod arc;
mod dir;
mod file;
mod open;
#[cfg(feature = "serialize")]
mod ser;
mod ty;

pub use abs::PathAbs;
pub use arc::PathArc;
pub use dir::{ListDir, PathDir};
pub use file::PathFile;
pub use open::PathOpen;
pub use ty::PathType;

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;

    use tempdir::TempDir;
    use regex::{self, Regex};

    use super::*;

    macro_rules! assert_match { ($re: expr, $err: expr) => {{
        let re = Regex::new(&$re).unwrap();
        let err = $err.to_string();
        assert!(
            re.is_match(&err), "\nGot Err         : {:?}\nMatching against: {:?}",
            err.to_string(),
            $re
        );
    }}}

    fn escape<P: AsRef<Path>>(path: P) -> String {
        regex::escape(&format!("{}", path.as_ref().display()))
    }

    #[test]
    /// Tests to make sure the error messages look like we expect.
    fn sanity_errors() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        let tmp_abs = PathDir::new(tmp_dir.path()).expect("tmp_abs");

        {
            let foo = PathFile::create(tmp_abs.join("foo.txt")).expect("foo.txt");
            let mut open = foo.read().unwrap();
            assert_match!(
                format!(
                    r"Bad file descriptor \(os error \d+\) when writing to {}",
                    escape(&foo)
                ),
                open.write_all(b"can't write").unwrap_err()
            );
            assert_match!(
                format!(
                    r"Invalid argument \(os error \d+\) when setting len for {}",
                    escape(&foo)
                ),
                open.set_len(0).unwrap_err()
            );
            foo.clone().remove().unwrap();
            assert_match!(
                format!(
                    r"No such file or directory \(os error \d+\) when opening {}",
                    escape(&foo)
                ),
                foo.edit().unwrap_err()
            );
        }
    }
}
