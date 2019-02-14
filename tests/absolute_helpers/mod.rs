use std::env;
use std::fs;
use std::io;
use std::path;

use path_abs::PathAbs;
use path_abs::PathInfo;

use tempdir::TempDir;

fn symlink_dir<P, Q>(src: P, dst: Q)
where
    P: AsRef<path::Path>,
    Q: AsRef<path::Path>,
{
    #[cfg(windows)]
    {
        use std::os::windows::fs as winfs;

        let dst = dst.as_ref();

        winfs::symlink_dir(src, &dst).expect(
            "Could not create symbolic link. \
             Run as Administrator, or on Windows 10 in Developer Mode",
        );
        dst.symlink_metadata().expect(
            "Link creation succeeded, but can't read link? \
             If you're using Wine, see bug 44948",
        );
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs as unixfs;

        unixfs::symlink(src, dst).expect("Could not create symbolic link");
    }

    #[cfg(all(not(windows), not(unix)))]
    unreachable!();
}

#[test]
fn absolute_path_is_idempotent() {
    crate::setup();
    // The current_dir() result is always absolute,
    // so absolutizing it should not change it.

    let actual = PathAbs::new(env::current_dir().unwrap()).unwrap();
    let expected = env::current_dir().unwrap().canonicalize().unwrap();

    assert_eq!(actual.as_path(), expected.as_path());
}

#[test]
fn absolute_path_removes_currentdir_component() {
    crate::setup();
    let actual = PathAbs::new("foo/./bar").unwrap();
    let expected = PathAbs::new("foo/bar").unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn absolute_path_removes_empty_component() {
    crate::setup();
    let actual = PathAbs::new("foo//bar").unwrap();
    let expected = PathAbs::new("foo/bar").unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn absolute_path_lexically_resolves_parentdir_component() {
    crate::setup();
    let tmp_dir = TempDir::new("normalize_parentdir").unwrap();
    let a_dir = tmp_dir.path().join("a");
    fs::create_dir_all(&a_dir).unwrap();

    let b_dir = tmp_dir.path().join("b");
    fs::create_dir_all(&b_dir).unwrap();

    fs::create_dir_all(&b_dir.join("target")).unwrap();

    let link_path = a_dir.join("link");
    symlink_dir("../b/target", link_path);

    // Because of the symlink, a/link/../foo is actually b/foo, but
    // lexically resolving the path produces a/foo.
    let actual = PathAbs::new(a_dir.join("link/../foo")).unwrap();
    let expected = PathAbs::new(a_dir.join("foo")).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn absolute_path_interprets_relative_to_current_directory() {
    crate::setup();
    let actual = PathAbs::new("foo").unwrap();
    let expected = PathAbs::new(env::current_dir().unwrap().join("foo")).unwrap();

    assert_eq!(actual, expected);
}

#[cfg(unix)]
mod unix {
    use super::*;
    use path_abs::PathInfo;

    #[test]
    fn absolute_path_need_not_exist() {
        crate::setup();

        // It's not likely this path would exist, but let's be sure.
        let raw_path = path::Path::new("/does/not/exist");
        assert_eq!(
            raw_path.metadata().unwrap_err().kind(),
            io::ErrorKind::NotFound,
        );

        let path = PathAbs::new(raw_path).unwrap();

        assert_eq!(path.as_os_str(), "/does/not/exist");
    }

    #[test]
    fn absolute_path_cannot_go_above_root() {
        crate::setup();
        let err = PathAbs::new("/foo/../..").unwrap_err();

        assert_eq!(err.io_error().kind(), io::ErrorKind::NotFound);
        assert_eq!(err.io_error().to_string(), ".. consumed root");
        assert_eq!(err.action(), "resolving absolute");
        assert_eq!(err.path(), path::Path::new("/foo/../.."));
    }
}

#[cfg(windows)]
mod windows {
    use super::*;

    #[test]
    fn absolute_path_need_not_exist() {
        crate::setup();

        // It's not likely this path would exist, but let's be sure.
        let raw_path = path::Path::new(r"C:\does\not\exist");
        assert_eq!(
            raw_path.metadata().unwrap_err().kind(),
            io::ErrorKind::NotFound,
        );

        let path = PathAbs::new(raw_path).unwrap();
        assert_eq!(path.as_os_str(), r"\\?\C:\does\not\exist");
    }

    #[test]
    fn absolute_path_cannot_go_above_root() {
        crate::setup();
        let err = PathAbs::new(r"C:\foo\..\..").unwrap_err();

        assert_eq!(err.io_error().kind(), io::ErrorKind::NotFound);
        assert_eq!(err.io_error().to_string(), ".. consumed root");
        assert_eq!(err.action(), "resolving absolute");
        assert_eq!(err.path(), path::Path::new(r"C:\foo\..\.."));
    }

    #[test]
    fn absolute_supports_root_only_relative_path() {
        crate::setup();
        let actual = PathAbs::new(r"\foo").unwrap();

        let mut current_drive_root = path::PathBuf::new();
        current_drive_root.extend(
            env::current_dir().unwrap().components().take(2), // the prefix (C:) and root (\) components
        );

        let expected = PathAbs::new(current_drive_root.join("foo")).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn absolute_supports_prefix_only_relative_path() {
        crate::setup();
        let actual = PathAbs::new(r"C:foo").unwrap();

        let expected =
            PathAbs::new(path::Path::new(r"C:").canonicalize().unwrap().join("foo")).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn absolute_accepts_bogus_prefix() {
        crate::setup();
        let path = PathAbs::new(r"\\?\bogus\path\").unwrap();

        assert_eq!(path.as_os_str(), r"\\?\bogus\path");
    }
}
