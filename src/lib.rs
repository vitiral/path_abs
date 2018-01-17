//! Extensions to stdlib `Path` types, plus the `PathAbs` type.
//!
//! [`PathAbs`](structs.PathAbs.html) adds a much needed type to the rust ecosystem:
//! a path which is guaranteed to exist (at least on creation), is serializable, and has
//! extension methods like `create[file/dir/dir_all]`, `read_*` and `write_*`.
//!
//! In addition, `PathAbs` is serializable through serde (even on windows!) by using the crate
//! [`stfu8`](https://crates.io/crates/stfu8) to encode/decode any ill-formed UTF-16.
//! See that crate for more details on how the resulting encoding can be edited (by hand)
//! even in the case of what *would be* ill-formed UTF-16.

extern crate serde;
extern crate stfu8;

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate tempdir;
#[cfg(test)]
extern crate serde_json;

use std::io;
use std::fs;
use std::fmt;
use std::ops::Deref;
use std::convert::AsRef;
use std::path::{Path, PathBuf};

mod file;
// mod ser;

pub use file::PathFile;

// #[cfg(test)]
// mod tests;

// ------------------------------
// -- EXPORTED TYPES / METHODS

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An path which is guaranteed to:
/// - Exist (on creation, the file may or may not be deleted later).
/// - Be absolute (cannonicalized). On linux this means it will start with root (`/`) and
///   have no symlinks.
///
/// > Implemented by calling `Path::canonicalize()` under the hood.
pub struct PathAbs(PathBuf);

impl PathAbs {
    /// Instantiate a new `PathAbs`. The path must exist or `io::Error` will be returned.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    /// let lib = PathAbs::new("src/lib.rs").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        Ok(PathAbs(path.as_ref().canonicalize()?))
    }


    /// Instantiate a new `PathAbs` to a directory, creating it first if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example";
    ///
    /// # let _ = ::std::fs::remove_dir(example);
    ///
    /// let path = PathAbs::create_dir(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathAbs::create_dir(example).unwrap();
    /// # }
    /// ```
    pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        if let Err(err) = fs::create_dir(&path) {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {},
                _ => return Err(err),
            }
        }
        PathAbs::new(path)
    }

    /// Instantiate a new `PathAbs` to a directory, recursively recreating it and all of its parent
    /// components if they are missing.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example/long/path";
    ///
    /// # let _ = ::std::fs::remove_dir_all("target/example");
    ///
    /// let path = PathAbs::create_dir_all(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathAbs::create_dir_all(example).unwrap();
    /// # }
    /// ```
    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        fs::create_dir_all(&path)?;
        PathAbs::new(path)
    }

    /// Join a path onto the `PathAbs`, expecting it to exist. Returns the resulting `PathAbs`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    /// let src = PathAbs::new("src").unwrap();
    /// let lib = src.join_abs("lib.rs").unwrap();
    /// # }
    /// ```
    pub fn join_abs<P: AsRef<Path>>(&self, path: P) -> io::Result<PathAbs> {
        let joined = self.join(path.as_ref());
        Ok(PathAbs::new(joined)?)
    }


    /// For constructing mocked paths during tests. This is effectively the same as a `PathBuf`.
    ///
    /// This is NOT checked for validity so the file may or may not actually exist and will
    /// NOT be, in any way, an absolute or canonicalized path.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    /// // this file exist
    /// let lib = PathAbs::new("src/lib.rs").unwrap();
    ///
    /// let lib_mocked = PathAbs::mock("src/lib.rs");
    ///
    /// // in this case, the mocked file exists
    /// assert!(lib_mocked.exists());
    ///
    /// // However, it is NOT equivalent to `lib`
    /// assert_ne!(lib, lib_mocked);
    ///
    /// // this file doesn't exist at all
    /// let dne = PathAbs::mock("src/dne.rs");
    /// assert!(!dne.exists());
    /// # }
    /// ```
    pub fn mock<P: AsRef<Path>>(fake_path: P) -> PathAbs {
        PathAbs(fake_path.as_ref().to_path_buf())
    }
}

impl fmt::Debug for PathAbs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathBuf> for PathAbs {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for PathAbs {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Deref for PathAbs {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

