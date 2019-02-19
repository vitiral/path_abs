/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! The absolute path type, the root type for all `Path*` types in this module.
use std::env;
use std::ffi;
use std::fmt;
use std::io;
use std::path::{Component, PrefixComponent};
use std_prelude::*;

use super::{Error, PathMut, PathOps, Result};

/// Converts any PrefixComponent into verbatim ("extended-length") form.
fn make_verbatim_prefix(prefix: &PrefixComponent<'_>) -> Result<PathBuf> {
    let path_prefix = Path::new(prefix.as_os_str());

    if prefix.kind().is_verbatim() {
        // This prefix already uses the extended-length
        // syntax, so we can use it as-is.
        Ok(path_prefix.to_path_buf())
    } else {
        // This prefix needs canonicalization.
        let res = path_prefix
            .canonicalize()
            .map_err(|e| Error::new(e, "canonicalizing", path_prefix.to_path_buf().into()))?;
        Ok(res)
    }
}

/// Pops the last component from path, returning an error for a root path.
fn pop_or_error(path: &mut PathBuf) -> ::std::result::Result<(), io::Error> {
    if path.pop() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, ".. consumed root"))
    }
}

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An absolute (not _necessarily_ [canonicalized][1]) path that may or may not exist.
///
/// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
pub struct PathAbs(pub(crate) Arc<PathBuf>);

impl PathAbs {
    /// Construct an absolute path from an arbitrary (absolute or relative) one.
    ///
    /// This is different from [`canonicalize`] in that it _preserves_ symlinks
    /// and the destination may or may not exist.
    ///
    /// This function will:
    /// - Resolve relative paths against the current working directory.
    /// - Strip any `.` components (`/a/./c` -> `/a/c`)
    /// - Resolve `..` _semantically_ (not using the file system). So, `a/b/c/../d => a/b/d` will
    ///   _always_ be true regardless of symlinks. If you want symlinks correctly resolved, use
    ///   `canonicalize()` instead.
    ///
    /// > On windows, this will sometimes call `canonicalize()` on the first component to guarantee
    /// > it is the correct canonicalized prefix. For paths starting with root it also has to get
    /// > the [`current_dir`]
    ///
    /// > On linux, the only syscall this will make is to get the [`current_dir`] for relative
    /// > paths.
    ///
    /// [`canonicalize`]: struct.PathAbs.html#method.canonicalize
    /// [`current_dir`]: fn.current_dir.html
    ///
    /// # Examples
    ///
    /// ```rust
    /// use path_abs::{PathAbs, PathInfo};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let lib = PathAbs::new("src/lib.rs")?;
    ///
    /// assert_eq!(lib.is_absolute(), true);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<PathAbs> {
        let path = Arc::new(path.as_ref().to_path_buf());
        let mut res = PathBuf::new();

        fn maybe_init_res(res: &mut PathBuf, resolvee: Arc<PathBuf>) -> Result<()> {
            if !res.as_os_str().is_empty() {
                // res has already been initialized, let's leave it alone.
                return Ok(());
            }

            // res has not been initialized, let's initialize it to the
            // canonicalized current directory.
            let cwd = env::current_dir().map_err(|e| {
                Error::new(e, "getting current_dir while resolving absolute", resolvee)
            })?;
            *res = cwd
                .canonicalize()
                .map_err(|e| Error::new(e, "canonicalizing", cwd.into()))?;

            Ok(())
        };

        for each in path.components() {
            match each {
                Component::Prefix(p) => {
                    // We don't care what's already in res, we can entirely
                    // replace it..
                    res = make_verbatim_prefix(&p)?;
                }

                Component::RootDir => {
                    if cfg!(windows) {
                        // In an ideal world, we would say
                        //
                        //  res = std::fs::canonicalize(each)?;
                        //
                        // ...to get a properly canonicalized path.
                        // Unfortunately, Windows cannot canonicalize `\` if
                        // the current directory happens to use extended-length
                        // syntax (like `\\?\C:\Windows`), so we'll have to do
                        // it manually: initialize `res` with the current
                        // working directory (whatever it is), and truncate it
                        // to its prefix by pushing `\`.
                        maybe_init_res(&mut res, path.clone())?;
                        res.push(each);
                    } else {
                        // On other platforms, a root path component is always
                        // absolute so we can replace whatever's in res.
                        res = Path::new(&each).to_path_buf();
                    }
                }

                // This does nothing and can be ignored.
                Component::CurDir => (),

                Component::ParentDir => {
                    // A parent component is always relative to some existing
                    // path.
                    maybe_init_res(&mut res, path.clone())?;
                    pop_or_error(&mut res)
                        .map_err(|e| Error::new(e, "resolving absolute", path.clone()))?;
                }

                Component::Normal(c) => {
                    // A normal component is always relative to some existing
                    // path.
                    maybe_init_res(&mut res, path.clone())?;
                    res.push(c);
                }
            }
        }

        Ok(PathAbs(Arc::new(res)))
    }

    /// Create a PathAbs unchecked.
    ///
    /// This is mostly used for constructing during tests, or if the path was previously validated.
    /// This is effectively the same as a `Arc<PathBuf>`.
    ///
    /// > Note: This is memory safe, so is not marked `unsafe`. However, it could cause
    /// > panics in some methods if the path was not properly validated.
    pub fn new_unchecked<P: Into<Arc<PathBuf>>>(path: P) -> PathAbs {
        PathAbs(path.into())
    }

    /// Return a reference to a basic `std::path::Path`
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }
}

impl fmt::Debug for PathAbs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PathMut for PathAbs {
    fn append<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.0.append(path)
    }
    fn pop_up(&mut self) -> Result<()> {
        self.0.pop_up()
    }
    fn truncate_to_root(&mut self) {
        self.0.truncate_to_root()
    }
    fn set_file_name<S: AsRef<ffi::OsStr>>(&mut self, file_name: S) {
        self.0.set_file_name(file_name)
    }
    fn set_extension<S: AsRef<ffi::OsStr>>(&mut self, extension: S) -> bool {
        self.0.set_extension(extension)
    }
}

impl PathOps for PathAbs {
    type Output = PathAbs;

    fn concat<P: AsRef<Path>>(&self, path: P) -> Result<Self::Output> {
        Ok(PathAbs(self.0.concat(path)?))
    }

    fn join<P: AsRef<Path>>(&self, path: P) -> Self::Output {
        let buf = Path::join(self.as_path(), path);
        Self::Output::new_unchecked(buf)
    }

    fn with_file_name<S: AsRef<ffi::OsStr>>(&self, file_name: S) -> Self::Output {
        PathAbs(self.0.with_file_name(file_name))
    }

    fn with_extension<S: AsRef<ffi::OsStr>>(&self, extension: S) -> Self::Output {
        PathAbs(self.0.with_extension(extension))
    }
}

impl AsRef<ffi::OsStr> for PathAbs {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_ref().as_ref()
    }
}

impl AsRef<Path> for PathAbs {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathAbs {
    fn as_ref(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

impl Borrow<Path> for PathAbs {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl Borrow<PathBuf> for PathAbs {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl<'a> Borrow<Path> for &'a PathAbs {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl<'a> Borrow<PathBuf> for &'a PathAbs {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl From<PathAbs> for Arc<PathBuf> {
    fn from(path: PathAbs) -> Arc<PathBuf> {
        path.0
    }
}

impl From<PathAbs> for PathBuf {
    fn from(path: PathAbs) -> PathBuf {
        match Arc::try_unwrap(path.0) {
            Ok(p) => p,
            Err(inner) => inner.as_ref().clone(),
        }
    }
}
