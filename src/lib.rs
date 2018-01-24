/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Absolute serializable path types and associated methods.
//!
//! This library provides the following types:
//! - [`PathAbs`](struct.PathAbs.html): a reference counted absolute (canonicalized) path that is
//!   guaranteed (when created) to exist.
//! - [`PathFile`](struct.PathFile.html): a `PathAbs` that is guaranteed to be a file, with
//!   associated methods.
//! - [`PathDir`](struct.PathDir.html): a `PathAbs` that is guaranteed to be a directory, with
//!   associated methods.
//! - [`PathType`](struct.PathType.html): an enum containing either a file or a directory. Returned
//!   by `PathDir::list`.
//!
//! In addition, all types are serializable through serde (even on windows!) by using the crate
//! [`stfu8`](https://crates.io/crates/stfu8) to encode/decode, allowing ill-formed UTF-16.
//! See that crate for more details on how the resulting encoding can be edited (by hand)
//! even in the case of what *would be* ill-formed UTF-16.
//!
//! Also see the [project repo](https://github.com/vitiral/path_abs) and consider leaving a star!
//!
//! > All types are internally `Arc<PathBuf>` so they are extremely cheap to clone. When working
//! > with paths a reference count is NOT an expensive operation for you!
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
