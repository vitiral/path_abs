# path_abs: Absolute serializable path types and associated methods.
[![Build Status](https://travis-ci.org/vitiral/path_abs.svg?branch=windows)](https://travis-ci.org/vitiral/path_abs)
[![Build status](https://ci.appveyor.com/api/projects/status/vgis54solhygre0n?svg=true)](https://ci.appveyor.com/project/vitiral/path-abs)
[![Docs](https://docs.rs/path_abs/badge.svg)](https://docs.rs/path_abs)

**See the [library docs](https://docs.rs/path_abs) for information on the
types**

The rust `Path` and `PathBuf` are great when you are constructing paths on the
filesystem that may or may not exist, or you care *immensely* about performance
and don't want the overhead of creating absolute (canonicalized) paths.

However, they have several downsides:
- They are not ergonomic. Actually *using* paths requires you to go through
  `fs` module, namely `File`, `OpenOptions`, `create_dir`, etc. It is NOT fun.
  `PathAbs` provides convienient methods -- you already know the path *exists*,
  now you just want to *do things with it*. You can read/write/etc using
  methods defined directly on `PathFile` and `PathDir`.
- Comparing paths is not reliable. Is `/foo/bar/baz` the same path as
  `bar/baz`? It's impossible to tell without knowing the current directory
  and the state of symlinks.
- It is impossible to know from the type whether a path exists (or indeed, ever
  existed) or what its  filetype is. Most applications are not deleting files,
  so validating that a path exists once is usually "good enough", but no such
  validation is guaranteed with `Path`. This is not wrong -- `Path` is supposed
  to represent just a "string of cross-platform bits". However, ensuring
  that you are only "referencing paths that once existed" is very helpful to
  reduce unexpected errors in your application.
- There is no way to serialize Paths in an effective manner. Actually getting
  the data has to happen through `std::os::<platform>::ffi::OsStrExt` and
  is different on windows and linux. Even worse, window's UTF-16 can be
  ill-formed which is *invalid* UTF-8, so cannot be encoded into UTF-8
  directly. `PathAbs` solves this by using the
  [`stfu8`](https://github.com/vitiral/stfu8) crate under the hood.

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
