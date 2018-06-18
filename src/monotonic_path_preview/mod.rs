//! Cleaned-up cross-platform path handling
//!
//! Most operating systems accept a complex syntax for specifying filesystem
//! paths, including special notation for things like "the current directory"
//! and "the parent directory" that make path-handling code intricate. If
//! filesystem paths always described a straight-line path from the root to
//! the file or directory in question, path-handling code could be much simpler.
//!
//! This module contains types representing exactly those kinds of paths.
//!
//! # Examples
//!
//! ```rust
//! # fn foo() -> Result<(), Box<std::error::Error>> {
//! let install_structure = vec![
//!     Relative::new("bin")?;
//!     Relative::new("lib")?;
//!     Relative::new("share/applications")?;
//!     Relative::new("share/icons")?;
//!     Relative::new("share/man")?;
//! ];
//!
//! let raw_install_path = std::env::os_args().next()?;
//! let install_path = Absolute::new(raw_install_path)?;
//!
//! for each in install_structure.iter() {
//!     std::fs::create_dir_all(install_path.join_relative(each))?;
//! }
//! # Ok(())
//! # }
//! ```
use std::collections;
use std::error;
use std::ffi;
use std::fmt;
use std::io;
use std::path;

/// An error encountered during path handling.
#[derive(Debug)]
pub enum Error {
    /// An error returned by the operating system.
    ///
    /// `err` is the underlying error returned by the operating system.
    ///
    /// `at` is the path that provoked the error.
    IoError {
        err: io::Error,
        at: path::PathBuf,
    },

    /// Returned by [`Absolute::new()`] when given a path that involves a
    /// symlink loop.
    ///
    /// [`Absolute::new()`]: struct.Absolute.html#method.new
    SymlinkLoops(path::PathBuf),

    /// Returned by [`Relative::new()`] when given a (partially or fully) absolute
    /// path.
    ///
    /// [`Relative::new()`]: struct.Relative.html#method.new
    PathIsAbsolute(path::PathBuf),

    /// Returned by [`Relative::new()`] when given a path with enough `/../`
    /// components to escape whatever prefix it's joined to.
    ///
    /// [`Relative::new()`]: struct.Relative.html#method.new
    RelativePathEscapesPrefix(path::PathBuf),

    #[doc(hidden)]
    __NonExhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::IoError { err, at } => write!(f, "{}: {:?}", err, at),
            Error::SymlinkLoops(p) => {
                write!(f, "Found an infinite symlink loop: {:?}", p)
            }
            Error::PathIsAbsolute(p) => write!(
                f,
                "Tried to make a relative path from absolute path {:?}",
                p
            ),
            Error::RelativePathEscapesPrefix(p) => write!(
                f,
                "Tried to make a relative path with leading '..': {:?}",
                p
            ),
            _ => write!(f, "Unknown error"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::IoError { err: e, at: _ } => e.description(),
            Error::SymlinkLoops(_) => "Found an infinite symlink loop",
            Error::PathIsAbsolute(_) => {
                "Tried to make a relative path from an absolute path"
            }
            Error::RelativePathEscapesPrefix(_) => {
                "Tried to make a relative path with leading '..'"
            }
            _ => "Unknown error",
        }
    }
}

/// Splits a path into a head and a tail.
///
/// The "head" consists of the Prefix and RootDir components, if any,
/// and is fully canonicalized.
///
/// The "tail" consists of all the other components that follow the head.
///
/// # Errors
///
/// Returns [`Error::IoError`] if the head cannot be canonicalized.
/// For example, if the process' current working directory has been deleted,
/// or the given path includes a syntactically invalid prefix.
///
/// [`Error::IoError`]: struct.Error.html#variant.IoError
fn split_head_and_tail<P: AsRef<path::Path>>(
    path: P,
) -> Result<
    (
        path::PathBuf,
        collections::VecDeque<ffi::OsString>,
    ),
    Error,
> {
    let path = path.as_ref();

    debug!("Splitting head and tail of {:?}", path);

    // The path's head is the prefix and root components (if any).
    fn is_head_part(c: &path::Component) -> bool {
        match c {
            path::Component::Prefix(_) => true,
            path::Component::RootDir => true,
            _ => false,
        }
    }

    let mut head: path::PathBuf = path.components()
        .take_while(is_head_part)
        .collect();

    // If this path is purely relative, it's relative to our current
    // directory.
    if head.as_os_str() == "" {
        head.push(".");
    }

    debug!("Raw head: {:?}", head);

    // The path's head can be safely converted into an absolute
    // path with .canonicalize(), since it must exist.
    head = head.canonicalize()
        .map_err(|err| Error::IoError {
            err: err,
            at: path.into(),
        })?;

    // The tail is kind of a queue of components to check. Since we will
    // be adding and removing things from diffferent sources, we convert
    // everything to an `OsString` to make the lifetimes easier.
    let tail: collections::VecDeque<ffi::OsString> = path.components()
        .skip_while(is_head_part)
        .map(|each| each.as_os_str().to_os_string())
        .collect();

    debug!("Head: {:?}, tail: {:?}", head, tail);
    Ok((head, tail))
}

/// Returns the target of the symlink at `path`, if it exists and is one.
///
/// If `path` is a symlink and we can read the target, returns
/// `Ok(Some(target))`.
/// If `path` is not a symlink, returns `Ok(None)`.
/// If `path` does not exist, it's still not a symlink and this function
/// returns `Ok(None)`.
/// Otherwise, returns the relevant error.
fn read_link_if_exists<P: AsRef<path::Path>>(
    path: P,
) -> Result<Option<path::PathBuf>, Error> {
    let path = path.as_ref();

    // In theory, we could just call .read_link() directly and handle the error
    // result, but:
    //
    // - on Windows, the "can't read this because it isn't a symlink"
    //   error is returned as an io::ErrorKind::Other rather than
    //   io::ErrorKind::InvalidInput.
    // - on Windows, some not-smart filesystems (like VirtualBox's shared
    //   folders) don't understand the API call, and just return "invalid
    //   operation".
    //
    // Therefore, we'll check if the thing is a symlink before trying to read
    // it.
    path.symlink_metadata()
        // If we successfully got metadata, check if it's a symlink.
        .map(|metadata| metadata.file_type().is_symlink())
        // If we failed to get metadata...
        .or_else(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                // ...and we failed because the path doesn't exist, by
                // definition it cannot be a symlink.
                debug!(
                    "{:?} does not exist, it's not a symlink",
                    path,
                );
                Ok(false)
            } else {
                // Any other error we can return as-is.
                error!("Could not check {:?}", path);
                Err(err)
            }
        })
        // If we know for sure whether this is a symlink...
        .and_then(|is_symlink| {
            // ...and it *is* a symlink...
            if is_symlink {
                // ...let's read it to find the target.
                debug!("{:?} exists and is a symlink", path);
                Ok(Some(path.read_link()?))

            // ...and it *isn't* a symlink...
            } else {
                // ...then obviously we don't have a target.
                debug!("{:?} exists and isn't a symlink", path);
                Ok(None)
            }
        })
        .map_err(|err| Error::IoError {
            err: err,
            at: path.into(),
        })
}

/// An absolute path that may or may not exist.
///
/// This path obeys the following invariants:
///
/// - It is absolute, having a prefix (on Windows) and a root directory
///   component.
/// - It contains only named path components, no `/./` or `/../` ones.
/// - It uses the platform-native path-component separator (`/` on POSIX,
///   `\` on Windows).
///
/// Therefore:
///
/// - It's always reasonably straight-forward for humans to understand.
/// - On Windows, it uses [extended-length path syntax], so cross-platform
///   applications don't need to worry about most traditional Windows path
///   limitations.
/// - You can join more named path components on the end without having to
///   check the filesystem or re-normalize the path.
///
/// Since this type implements `AsRef<Path>`, it can be used with almost any
/// standard library function that expects a path.
///
/// [extended-length path syntax]: https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247(v=vs.85).aspx#maxpath
///
/// # Examples
///
/// ```rust
/// # fn example() -> Result<(), Box<std::error::Error>> {
/// use std::fs;
/// use std::fs::Write;
///
/// let log_storage = Absolute::new("/var/log/myapp")?;
///
/// let current_log = log_storage.join("events")?;
/// let previous_log = log_storage.join("events.old")?;
///
/// fs::rename(current_log, previous_log)?;
///
/// let log_file = fs::OpenOptions::new()
///     .write(true)
///     .create(true)
///     .open(current_log)?;
///
/// write!(&mut log_file, "Log rotated.")?;
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Absolute(path::PathBuf);

impl Absolute {
    /// Convert an arbitrary path to follow the rules for an `Absolute` path.
    ///
    /// - If the path is relative, it is interpreted relative to the process'
    ///   current working directory.
    /// - Any `/./` components in the path are removed.
    /// - If a component that refers to an existing, readable symlink is
    ///   followed by a `/../` component, it will be resolved so that the
    ///   overall path's meaning is unchanged.
    /// - If a component that does not exist in the filesystem, or which refers
    ///   to an ordinary file or directory, is followed by a `/../` component,
    ///   they cancel each other out and both are removed.
    /// - Other components are left alone.
    ///
    /// # Performance
    ///
    /// In the best-case, the given path already follows the rules, and
    /// we only call [`canonicalize()`] on the head (the prefix and root
    /// directory, if any) to convert it to canonical syntax.
    ///
    /// In general, we will call [`symlink_metadata()`] on every component
    /// preceding a `/../` component, and (if it turns out to be a symlink)
    /// [`read_link()`]. The process repeats if the symlink target includes
    /// any `/../` components of its own.
    ///
    /// [`canonicalize()`]: https://doc.rust-lang.org/stable/std/fs/fn.canonicalize.html
    /// [`symlink_metadata()`]: https://doc.rust-lang.org/stable/std/fs/fn.symlink_metadata.html
    /// [`read_link()`]: https://doc.rust-lang.org/stable/std/fs/fn.read_link.html
    ///
    /// # Platform-specific behaviour
    ///
    /// On Windows, this method correctly handles partially-absolute paths like
    /// `D:foo\bar.txt` that are relative to a path other than the current
    /// working directory.
    ///
    /// On Windows, the resulting path uses [extended-length path syntax], so
    /// it may confuse other applications not designed to handle such paths.
    /// For example, if you pass such a path on another application's command
    /// line, or write it to a configuration ile.
    ///
    /// [extended-length path syntax]: https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247(v=vs.85).aspx#maxpath
    ///
    /// # Errors
    ///
    /// Returns [`Error::IoError`] if the head of the given path cannot be
    /// canonicalized. For example, if the process' current working directory
    /// has been deleted, or the given path includes a syntactically invalid
    /// prefix.
    ///
    /// The same variant is returned if a problem is encountered while checking
    /// if a given path is a symlink, or while trying to read a symlink. For
    /// example, if the current user does not have permission to read the
    /// directory containing it, or the path is on a network-mounted filesystem
    /// that stopped responding.
    ///
    /// Returns [`Error::SymlinkLoops`] if resolving a symlink takes us back
    /// to a previously-resolved symlink. For example, if `/example/path/a`
    /// is a symlink to `/example/path/b`, and `b` is a symlink back to `a`,
    /// then giving this method a path like `/example/path/a/../c` will return
    /// this error. It's like the POSIX `ELOOP` error, but cross-platform.
    ///
    /// Note that "does not exist" is *not* a fatal error for this function;
    /// path components that do not exist by definition are not symlinks, and
    /// are treated the same way as every other component that is not a symlink.
    ///
    /// [`Error::IoError`]: struct.Error.html#variant.IoError
    /// [`Error::SymlinkLoops`]: struct.Error.html#variant.SymlinkLoops
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example() -> Result<(), Error> {
    /// let real_current_directory = Absolute::new(std::env::current_dir()?)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<path::Path>>(path: P) -> Result<Absolute, Error> {
        // This function steps through the components of path, checking each one
        // for monotonicity and building up a proper monotonic path in `res`.
        // `tail` contains the path components we've yet to check and clean up.
        let (mut res, mut tail) = split_head_and_tail(path)?;

        let mut seen_paths = collections::BTreeSet::new();

        // While there's still components left to check...
        while tail.len() > 0 {
            debug!(
                "Monotonic path: {:?}, to check: {:?}",
                res, tail
            );

            // Grab the component at the front of the tail, so we can check it.
            let current = tail.pop_front()
                .expect("len() > 0 but vec is empty?");

            if &current == "." {
                // A "." component can be ignored.
                //
                //    blah/./blah -> blah/blah

            } else if &current == ".." {
                // If we get a ParentDir component, the component at the end of
                // `res` might be a symlink. in which case we'll have to splice
                // the target path in at the beginning of `tail` so we have the
                // full, monotonic path.
                //
                // If `foo` is not a symlink:
                //
                //    blah/foo/../blah -> blah/blah
                //
                // If `foo` is a symlink to `relative/target`:
                //
                //    blah/symlink/../blah -> blah/relative/target/../blah
                //
                // If `foo` is a symlink to `/absolute/target`:
                //
                //    blah/symlink/../blah -> /absolute/target/../blah
                //

                // But first, let's check for symlink loops. If we've already
                // dereferenced this symlink before, we've hit a loop and we
                // will never find a suitable answer.
                if seen_paths.contains(&res) {
                    return Err(Error::SymlinkLoops(res));
                }

                // This is the first time we've dereferenced this symlink, note
                // it down.
                seen_paths.insert(res.clone());

                match read_link_if_exists(&res)? {
                    // The component before the ParentDir component was not
                    // a symlink, so we don't need to do anything to `tail`.
                    None => (),

                    // The component before the ParentDir component *was* a
                    // symlink, so we need to add it to tail so we'll get around
                    // to checking it.
                    Some(target) => {
                        debug!("Symlink target: {:?}", target);

                        // We'll re-check this ".." component once we've
                        // checked all the components from target. Since we're
                        // pushing to the front, we're pushing components in
                        // reverse order, and so we have to push this first.
                        tail.push_front("..".into());

                        // If the target is an absolute path, then the target's
                        // head replaces our current result. Conveniently,
                        // Windows will only create symlinks with fully-relative
                        // or fully-absolute targets, not root-relative
                        // (`\\foo`) or drive-relative (`C:foo`) paths, so we
                        // don't need to handle those cases.
                        let target = if target.is_absolute() {
                            debug!("Target is absolute");
                            let (new_head, new_target) =
                                split_head_and_tail(target)?;
                            res.push(new_head);

                            new_target

                        // The target is relative, so we can use it as-is.
                        } else {
                            debug!("Target is relative");
                            target
                                .components()
                                .map(|each| each.as_os_str().to_os_string())
                                .collect::<collections::VecDeque<_>>()
                        };

                        // Push all the components of target in reverse order
                        // so we'll check them the next time through the loop.
                        for each in target.into_iter().rev() {
                            tail.push_front(each);
                        }
                    }
                }

                // If the component at the end of `res` is a symlink, its
                // target has been pushed to the front of `tail` and will
                // be processed in due course. If it's not a symlink, this
                // ParentDir component nullifies it. Either way, we don't want
                // the last component of `res` anymore.
                res.pop();
            } else {
                // This component is a normal name. We'll trust it... for now.
                res.push(current);
            }
        }

        Ok(Absolute(res))
    }

    /// Clone this path, attempting to add an arbitrary relative path on the
    /// end.
    ///
    /// # Performance
    ///
    /// An expression like:
    ///
    /// ```rust
    /// absolute_path.join(path)?
    /// ```
    ///
    /// ...is the same as doing:
    ///
    /// ```rust
    /// absolute_path.join_relative(&Relative::new(path)?)
    /// ```
    ///
    /// ...and therefore involves the same allocation and other costs as
    /// calling [`Relative::new()`] yourself.
    ///
    /// If you plan on joining the same relative path to many `Absolute` paths,
    /// it's more efficient to call `Relative::new()` once yourself then use
    /// [`.join_relative()`] each time.
    ///
    /// [`Relative::new()`]: struct.Relative.html#method.new
    /// [`.join_relative()`]: #method.join_relative
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Relative::new()`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example() -> Result<(), Error> {
    /// let metadata_path = extraction_path.join("META-INF/MANIFEST.MF")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn join<P: AsRef<path::Path>>(
        &self,
        path: P,
    ) -> Result<Absolute, Error> {
        Ok(self.join_relative(&Relative::new(path)?))
    }

    /// Clone this path, adding the given [`Relative`] path on the end.
    ///
    /// If the thing you want to join isn't already a `Relative`, you may find
    /// it more ergonomic to call [`.join()`] instead.
    ///
    /// [`.join()`]: #method.join
    ///
    /// # Performance
    ///
    /// Since a `Relative` is guaranteed to follow the rules for `Absolute`
    /// paths (except for being absolute), no additional checks or processing
    /// need to be done, just straight concatenation.
    ///
    /// [`Relative`]: struct.Relative.html
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example() -> Result<Absolute, Error> {
    /// let search_path = vec![
    ///     Absolute::new("/usr/local/bin")?,
    ///     Absolute::new("/bin")?,
    ///     Absolute::new("/usr/bin")?,
    ///     Absolute::new("/sbin")?,
    ///     Absolute::new("/usr/sbin")?,
    /// ];
    ///
    /// let binary = Relative::new("cargo")?;
    ///
    /// for prefix in search_path {
    ///     let guess = prefix.join_relative(binary);
    ///
    ///     if guess.as_path().is_file() {
    ///         return Ok(guess)
    ///     }
    /// }
    ///
    /// # Ok(Absolute::new(".").unwrap())
    /// # }
    /// ```
    pub fn join_relative(&self, tail: &Relative) -> Absolute {
        Absolute(self.0.join(tail))
    }

    /// Coerces to a [`Path`] slice.
    ///
    /// Since `Absolute` implements `AsRef<Path>`, this method is not needed
    /// very often—you can often just pass it directly to the thing that needs
    /// a [`Path`].
    ///
    /// [`Path`]: https://doc.rust-lang.org/stable/std/path/struct.Path.html
    ///
    /// # Examples
    ///
    /// If you really, really need to convert an `Absolute` to a [`PathBuf`]:
    ///
    /// ```rust
    /// let owned_path: std::path::PathBuf = absolute_path.as_path().into();
    /// ```
    ///
    /// [`PathBuf`]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html
    pub fn as_path(&self) -> &path::Path {
        <Self as AsRef<path::Path>>::as_ref(self)
    }

    /// Coerces to an [`OsStr`] slice.
    ///
    /// Since `Absolute` implements `AsRef<OsStr>`, this method is not needed
    /// very often—you can often just pass it directly to the thing that needs
    /// an [`OsStr`].
    ///
    /// [`OsStr`]: https://doc.rust-lang.org/stable/std/ffi/struct.OsStr.html
    ///
    /// # Examples
    ///
    /// ```rust
    /// let install_path = std::env::os_args()
    ///     .first()
    ///     .expect("Specify the installation path on the command line.");
    ///
    /// let windows_path = Absolute::new("C:\\windows").unwrap();
    ///
    /// if windows_path.as_os_str() == install_path {
    ///     panic!("No, you can't install to the Windows path.");
    /// }
    /// ```
    pub fn as_os_str(&self) -> &ffi::OsStr {
        <Self as AsRef<ffi::OsStr>>::as_ref(self)
    }
}

impl AsRef<path::Path> for Absolute {
    fn as_ref(&self) -> &path::Path {
        self.0.as_path()
    }
}

impl AsRef<ffi::OsStr> for Absolute {
    fn as_ref(&self) -> &ffi::OsStr {
        self.0.as_os_str()
    }
}

/// A relative path that may be joined to an absolute path.
///
/// This path obeys the following invariants:
///
/// - It is relative, containing no prefix or root directory components.
/// - It contains only named path components, no `/./` or `/../` ones.
/// - It uses the platform-native path-component separator (`/` on POSIX,
///   `\` on Windows).
///
/// Therefore:
///
/// - It's always reasonably straight-forward for humans to understand.
/// - It can safely be appended to an [`Absolute`] path or another
///   `Relative` path without having to revalidatet the invariants.
/// - Joining a `Relative` to an `Absolute` will always produce a path that
///   refers to a child of the `Absolute`, unless the directory named by the
///   `Absolute` contains a symilnk to a directory outside it.
///
/// Since this type implements `AsRef<Path>`, it can be used with almost any
/// standard library function that expects a path, but you probably only want
/// to join it to an `Absolute` path.
///
/// [`Absolute`]: struct.Absolute.html
///
/// # Examples
///
/// ```rust
/// # fn example() -> Result<(), Error> {
/// let search_path = [
///     Absolute::new(get_user_config_path())?,
///     Absolute::new(get_system_config_path())?,
///     Absolute::new(get_default_config_path())?,
/// ];
///
/// let config_name = Relative::new("myapp/video.cfg")?;
///
/// for prefix in search_path {
///     let guess = prefix.join_relative(config_name);
///
///     if guess.as_path().is_file() {
///         return Ok(guess)
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Relative(path::PathBuf);

impl Relative {
    /// Convert an arbitrary path to follow the rules for a `Relative` path.
    ///
    /// - Any `/./` components in the path are removed.
    /// - If a named component is followed by a `/../` component, they cancel
    ///   each other out and both are removed. We cannot resolve symlinks here
    ///   since we do not know what absolute path this path is relative to.
    ///
    /// # Performance
    ///
    /// This validation is performed entirely in memory, with no reference to
    /// the filesystem.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PathIsAbsolute`] if the given path contains prefix or
    /// root directory components, like `/usr/share` or `C:file.txt`.
    ///
    /// Returns [`Error::RelativePathEscapesPrefix`] if any `/../` components
    /// cannot be normalized away, like `a/b/../../../c`.
    ///
    /// [`Error::PathIsAbsolute`]: enum.Error.html#variant.PathIsAbsolute
    /// [`Error::RelativePathEscapesPrefix`]: enum.Error.html#variant.RelativePathEscapesPrefix
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example(
    /// #   config: std::collections::BTreeMap<String, String>,
    /// #   request: std::collections::BTreeMap<String, String>,
    /// # )
    /// # -> Result<(), Error> {
    /// let web_root = Absolute::new(
    ///     config
    ///         .get("web_root")
    ///         .unwrap_or("/var/www/root")
    /// )?;
    ///
    /// let request_path = Relative::new(
    ///     request
    ///         .get("path")
    ///         unwrap_or("")
    /// )?;
    ///
    /// let mut data_path = web_root.join_relative(request_path);
    ///
    /// if data_path.as_path().is_dir() {
    ///     data_path = data_path.join("index.html");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<path::Path>>(path: P) -> Result<Relative, Error> {
        let components = path.as_ref().components();
        let mut res = path::PathBuf::new();

        for each in components {
            match each {
                path::Component::Prefix(_) | path::Component::RootDir => {
                    return Err(Error::PathIsAbsolute(path.as_ref().into()));
                }
                path::Component::Normal(name) => res.push(name),
                path::Component::ParentDir => {
                    if res.as_os_str() == "" {
                        return Err(Error::RelativePathEscapesPrefix(
                            path.as_ref().into(),
                        ));
                    }

                    res.pop();
                }
                path::Component::CurDir => (),
            }
        }

        Ok(Relative(res))
    }

    /// Clone this path, attempting to add an arbitrary relative path on the
    /// end.
    ///
    /// # Performance
    ///
    /// An expression like:
    ///
    /// ```rust
    /// other_relative_path.join(path)?
    /// ```
    ///
    /// ...is the same as doing:
    ///
    /// ```rust
    /// other_relative_path.join_relative(&Relative::new(path)?)
    /// ```
    ///
    /// ...and therefore involves the same allocation and other costs as
    /// calling [`Relative::new()`] yourself.
    ///
    /// If you plan on joining the same relative path to many other `Relative`
    /// paths, it's more efficient to call `Relative::new()` once yourself and
    /// call [`.join_relative()`] each time.
    ///
    /// [`Relative::new()`]: struct.Relative.html#method.new
    /// [`.join_relative()`]: #method.join_relative
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Relative::new()`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example() -> Result<(), Error> {
    /// let config_base = Relative::new("SuperSoftwareCo/MyCoolApp")?;
    ///
    /// let video_config = config_base.join("video.cfg")?;
    /// let audio_config = config_base.join("audio.cfg")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn join<P: AsRef<path::Path>>(
        &self,
        path: P,
    ) -> Result<Relative, Error> {
        Ok(self.join_relative(&Relative::new(path)?))
    }

    /// Clone this path, adding the given `Relative` path on the end.
    ///
    /// [`.join()`]: #method.join
    ///
    /// # Performance
    ///
    /// Since `Relative` paths all have the same invariants, no additional
    /// checks or processing need to be done, just straight concatenation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example(testdir: Absolute) -> Result<(), Error> {
    /// let browsers = [
    ///     Relative::new("Firefox")?,
    ///     Relative::new("Chrome")?,
    ///     Relative::new("Edge")?,
    /// ];
    ///
    /// let platforms = [
    ///     Relative::new("Windows")?,
    ///     Relative::new("Linux")?,
    ///     Relative::new("macOS")?,
    /// ];
    ///
    /// let result_paths: Vec<Relative> = browsers
    ///     .iter()
    ///     .flat_map(|name| {
    ///         platforms
    ///             .iter()
    ///             .map(name.join_relative)
    ///     })
    ///     .collect();
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn join_relative(&self, tail: &Relative) -> Relative {
        Relative(self.0.join(tail))
    }

    /// Coerces to a [`Path`] slice.
    ///
    /// Since `Relative` implements `AsRef<Path>`, this method is not needed
    /// very often—you can often just pass it directly to the thing that needs
    /// a [`Path`].
    ///
    /// [`Path`]: https://doc.rust-lang.org/stable/std/path/struct.Path.html
    ///
    /// # Examples
    ///
    /// If you really, really need to convert a `Relative` to a [`PathBuf`]:
    ///
    /// ```rust
    /// let owned_path: std::path::PathBuf = relative.as_path().into();
    /// ```
    ///
    /// [`PathBuf`]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html
    pub fn as_path(&self) -> &path::Path {
        <Self as AsRef<path::Path>>::as_ref(self)
    }

    /// Coerces to an [`OsStr`] slice.
    ///
    /// Since `Relative` implements `AsRef<OsStr>`, this method is not needed
    /// very often—you can often just pass it directly to the thing that needs
    /// an [`OsStr`].
    ///
    /// [`OsStr`]: https://doc.rust-lang.org/stable/std/ffi/struct.OsStr.html
    ///
    /// # Examples
    ///
    /// If you really, really need to convert a `Relative` to a [`OsString`]:
    ///
    /// ```rust
    /// let owned_string: std::ffi::OsString = relative.as_os_str().into();
    /// ```
    ///
    /// [`OsString`]: https://doc.rust-lang.org/stable/std/ffi/struct.OsString.html
    pub fn as_os_str(&self) -> &ffi::OsStr {
        <Self as AsRef<ffi::OsStr>>::as_ref(self)
    }
}

impl AsRef<path::Path> for Relative {
    fn as_ref(&self) -> &path::Path {
        self.0.as_path()
    }
}

impl AsRef<ffi::OsStr> for Relative {
    fn as_ref(&self) -> &ffi::OsStr {
        self.0.as_os_str()
    }
}

#[cfg(test)]
mod tests;
