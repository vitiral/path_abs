# path_abs: ergonomic paths and files in rust.
[![Build Status](https://travis-ci.org/vitiral/path_abs.svg?branch=windows)](https://travis-ci.org/vitiral/path_abs)
[![Build status](https://ci.appveyor.com/api/projects/status/vgis54solhygre0n?svg=true)](https://ci.appveyor.com/project/vitiral/path-abs)
[![Docs](https://docs.rs/path_abs/badge.svg)](https://docs.rs/path_abs)

The stdlib `Path`, `PathBuf` and `File` objects have non-helpful error messages
(they don't mention the path where an error is from!) and don't tell you
anything about the _type_ the path was, whether the path _even exists_, and
comparing paths is next to impossible (does one have a symlink? What is the
current workind directory? Etc).

The path_abs crate aims to make working with paths and files ergonomic, so that
you can focus on the _types_. It allows you to know that:
- The path you have existed (at least at _one time_).
- The path you have is a certain type (`PathFile` or `PathDir`)
- Any errors that happen when querying the filesystem will _include information
  about the path_.
- Methods related to your type are within easy reach. For example, you can
  `PathFile.append_str("something")` or `PathDir.list()`.

**See the [library docs](https://docs.rs/path_abs) for more information**

# LICENSE
The source code in this repository is Licensed under either of
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
