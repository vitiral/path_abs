# path_abs: the `PathAbs` type for handling *actual* paths.

The rust `Path` and `PathBuf` are great when you are constructing
paths on the filesystem that may or may not exist, or you care
*immensely* about performance and don't want the overhead of
storing an absolute path.

However, they have several downsides:
- Comparing paths is not reliable. Is `/foo/bar/baz` the same
  path as `bar/baz`? It's impossible to tell without knowing the
  current directory.
- It is impossible to know from the type if a path exists (or indeed, ever
  existed). Most applications are not deleting files, so validating that a
  path exists once is usually "good enough", but no such validation is
  guaranteed with `Path`. This is not wrong -- `Path` is supposed to represent
  just a "string of bits". However, in most cases this is the behavior that
  is wanted.
- There is no way to serialize Paths in an effective manner. Actually getting
  the data has to happen through `std::os::<platform>::ffi::OsStrExt` and
  is different on windows and linux. Even worse, window's UTF-16 can be
  ill-formed which is *invalid* UTF-8, so cannot be encoded into UTF-8
  directly.
- Actually *using* paths requires you to go through `fs` module, namely
  `File`, `OpenOptions`, `create_dir`, etc. It is NOT fun. `PathAbs` provides
  convienient methods -- you already know the path *exists*, now you
  just want to *do things with it*. You can read/write/delete/etc
  using methods defined directly on the path.

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

The STFU-8 protocol/specification(lol) itself (including the name) is licensed
under CC0 Community commons and anyone should be able to reimplement or change
it for any purpose without need of attribution. However, using the same name
for a completely different protocol would probably confuse people so please
don't do it.

