
use std::fs;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::ops::Deref;
use std::convert::AsRef;

use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

use super::PathAbs;

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An absolute path to a directory that exists, with associated methods.
pub struct PathDir(pub(crate) PathAbs);

impl PathDir {
    /// Instantiate a new `PathDir`. The directory must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a dir returns `io::ErrorKind::InvalidInput`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    ///
    /// # fn main() {
    /// let src = PathDir::new("src").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathDir> {
        let abs = PathAbs::new(path)?;
        PathDir::from_abs(abs)
    }

    /// Consume the `PathAbs` validating that the path is a directory and returning `PathDir`. The
    /// directory must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a file returns `io::ErrorKind::InvalidInput`.
    ///
    /// > This does not call [`Path::cannonicalize()`][1], instead trusting that the input
    /// > already a fully qualified path.
    ///
    /// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathAbs, PathDir};
    ///
    /// # fn main() {
    /// let src_abs = PathAbs::new("src").unwrap();
    /// let src_dir = PathDir::from_abs(src_abs).unwrap();
    /// # }
    /// ```
    pub fn from_abs(abs: PathAbs) -> io::Result<PathDir> {
        if abs.is_dir() {
            Ok(PathDir(abs))
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "path is not a dir"))
        }
    }

    /// Instantiate a new `PathDir` to a directory, creating it first if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example";
    ///
    /// # let _ = ::std::fs::remove_dir(example);
    ///
    /// let dir = PathDir::create(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathDir::create(example).unwrap();
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<PathDir> {
        if let Err(err) = fs::create_dir(&path) {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {},
                _ => return Err(err),
            }
        }
        PathDir::new(path)
    }

    /// Instantiate a new `PathDir` to a directory, recursively recreating it and all of its parent
    /// components if they are missing.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example/long/path";
    ///
    /// # let _ = ::std::fs::remove_dir_all("target/example");
    ///
    /// let path = PathDir::create_all(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathDir::create_all(example).unwrap();
    /// # }
    /// ```
    pub fn create_all<P: AsRef<Path>>(path: P) -> io::Result<PathDir> {
        fs::create_dir_all(&path)?;
        PathDir::new(path)
    }

    /// Join a path onto the `PathDir`, expecting it to exist. Returns the resulting `PathDir`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathDir, PathFile};
    ///
    /// # fn main() {
    /// let src = PathDir::new("src").unwrap();
    /// let lib = src.join_abs("lib.rs").unwrap().to_file().unwrap();
    /// # }
    /// ```
    pub fn join_abs<P: AsRef<Path>>(&self, path: P) -> io::Result<PathAbs> {
        let joined = self.join(path.as_ref());
        PathAbs::new(joined)
    }

    // pub fn list(&self) -> ReadDir {

    // }

}

impl fmt::Debug for PathDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathAbs> for PathDir {
    fn as_ref(&self) -> &PathAbs {
        &self.0
    }
}

impl AsRef<Path> for PathDir {
    fn as_ref(&self) -> &Path {
        &self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathDir {
    fn as_ref(&self) -> &PathBuf {
        &self.0.as_ref()
    }
}

impl Deref for PathDir {
    type Target = PathAbs;

    fn deref(&self) -> &PathAbs {
        &self.0
    }
}

impl Serialize for PathDir {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PathDir {
    fn deserialize<D>(deserializer: D) -> Result<PathDir, D::Error>
    where
        D: Deserializer<'de>,
    {
        let abs = PathAbs::deserialize(deserializer)?;
        PathDir::from_abs(abs).map_err(serde::de::Error::custom)
    }
}
