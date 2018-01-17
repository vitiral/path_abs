
use std::fs;
use std::fmt;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::ops::Deref;
use std::convert::AsRef;

use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

use super::PathAbs;

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An absolute path to a file that exists, with associated methods.
pub struct PathFile(PathAbs);

impl PathFile {
    /// Instantiate a new `PathFile`. The file must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a dir returns `io::ErrorKind::InvalidInput`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    /// let lib = PathFile::new("src/lib.rs").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathFile> {
        let abs = PathAbs::new(path)?;
        PathFile::from_abs(abs)
    }

    /// Consume the `PathAbs` validating that the path is a file and returning `PathFile`. The file
    /// must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a dir returns `io::ErrorKind::InvalidInput`.
    ///
    /// > This does not call [`Path::cannonicalize()`][1], instead trusting that the input
    /// > already a fully qualified path.
    ///
    /// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathAbs, PathFile};
    ///
    /// # fn main() {
    /// let lib_abs = PathAbs::new("src/lib.rs").unwrap();
    /// let lib_file = PathFile::from_abs(lib_abs).unwrap();
    /// # }
    /// ```
    pub fn from_abs(abs: PathAbs) -> io::Result<PathFile> {
        if abs.is_file() {
            Ok(PathFile(abs))
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "path is not a file"))
        }
    }

    /// Instantiate a new `PathFile`, creating it first if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example.txt";
    ///
    /// # let _ = ::std::fs::remove_file(example);
    ///
    /// let file = PathFile::create(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathFile::create(example).unwrap();
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<PathFile> {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)?;
        PathFile::new(path)
    }

    /// Read the entire contents of the file into a `String`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example.txt";
    /// # let _ = ::std::fs::remove_file(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected).unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn read_string(&self) -> io::Result<String> {
        let mut f = fs::OpenOptions::new()
            .read(true)
            .open(self)?;
        let mut out = String::with_capacity(f.metadata()?.len() as usize);
        f.read_to_string(&mut out)?;
        Ok(out)
    }

    /// Write the `str` to a file, truncating it first if it exist and creating it otherwise.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example.txt";
    /// # let _ = ::std::fs::remove_file(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected).unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn write_str(&self, s: &str) -> io::Result<()> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self)?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_all(s.as_bytes())
    }

    /// Append the `str` to a file, creating it if it doesn't exist.
    ///
    /// If the `str` is empty, this is equivalent to the unix `touch`
    /// except it does NOT update the timestamp
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    /// let example = "target/example.txt";
    /// # let _ = ::std::fs::remove_file(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar\nbaz";
    /// file.append_str("foo\nbar").unwrap();
    /// file.append_str("\nbaz").unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn append_str(&self, s: &str) -> io::Result<()> {
        let mut f = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self)?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_all(s.as_bytes())
    }
}

impl fmt::Debug for PathFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathAbs> for PathFile {
    fn as_ref(&self) -> &PathAbs {
        &self.0
    }
}

impl AsRef<Path> for PathFile {
    fn as_ref(&self) -> &Path {
        &self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathFile {
    fn as_ref(&self) -> &PathBuf {
        &self.0.as_ref()
    }
}

impl Deref for PathFile {
    type Target = PathAbs;

    fn deref(&self) -> &PathAbs {
        &self.0
    }
}

impl Serialize for PathFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PathFile {
    fn deserialize<D>(deserializer: D) -> Result<PathFile, D::Error>
    where
        D: Deserializer<'de>,
    {
        let abs = PathAbs::deserialize(deserializer)?;
        PathFile::from_abs(abs).map_err(serde::de::Error::custom)
    }
}
