// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn fixture(root: &Path) -> GeneratedTree {
    fs::create_dir(root.join("generated")).unwrap();
    fs::write(root.join("generated/replaced"), b"original").unwrap();
    fs::write(root.join("generated/unchanged"), b"unchanged").unwrap();
    fs::write(root.join("generated/obsolete"), b"obsolete").unwrap();
    let mut tree = GeneratedTree::empty(vec!["generated".into()]);
    tree.files
        .insert("generated/added".into(), b"added".to_vec());
    tree.files
        .insert("generated/replaced".into(), b"replacement".to_vec());
    tree.files
        .insert("generated/unchanged".into(), b"unchanged".to_vec());
    tree
}

#[test]
fn stale_publication_refuses_before_touching_any_owned_output() {
    let checkout = tempfile::tempdir().unwrap();
    let root = checkout.path();
    let tree = fixture(root);
    let delta = Delta::stage(root, &tree).unwrap();
    assert!(!root.join("generated/added").exists());
    fs::write(root.join("generated/unchanged"), b"edited after stage").unwrap();
    assert!(
        delta
            .apply(root)
            .unwrap_err()
            .to_string()
            .contains("generated/unchanged")
    );
    assert!(!root.join("generated/added").exists());
    assert_eq!(
        fs::read(root.join("generated/replaced")).unwrap(),
        b"original"
    );
    assert_eq!(
        fs::read(root.join("generated/obsolete")).unwrap(),
        b"obsolete"
    );
}

#[test]
fn an_unstaged_owned_file_refuses_without_pruning_it() {
    let checkout = tempfile::tempdir().unwrap();
    let root = checkout.path();
    let tree = fixture(root);
    let delta = Delta::stage(root, &tree).unwrap();
    fs::write(
        root.join("generated/unexpected"),
        b"another producer or edit",
    )
    .unwrap();
    assert!(
        delta
            .apply(root)
            .unwrap_err()
            .to_string()
            .contains("generated/unexpected")
    );
    assert!(!root.join("generated/added").exists());
    assert_eq!(
        fs::read(root.join("generated/unexpected")).unwrap(),
        b"another producer or edit"
    );
}

#[test]
fn publication_replays_from_observed_postimages_without_replacing_them() {
    let checkout = tempfile::tempdir().unwrap();
    let root = checkout.path();
    let tree = fixture(root);
    let replaced = root.join("generated/replaced");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&replaced, Permissions::from_mode(0o640)).unwrap();
    }
    #[cfg(unix)]
    let mut original = File::open(&replaced).unwrap();
    let delta = Delta::stage(root, &tree).unwrap();
    delta
        .entries
        .iter()
        .find(|entry| entry.relative == Path::new("generated/replaced"))
        .unwrap()
        .publish(root)
        .unwrap();
    let stamp = std::time::UNIX_EPOCH + std::time::Duration::from_secs(17);
    File::options()
        .write(true)
        .open(&replaced)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(stamp))
        .unwrap();

    delta.apply(root).unwrap();
    delta.apply(root).unwrap();
    assert_eq!(fs::metadata(&replaced).unwrap().modified().unwrap(), stamp);
    assert_eq!(fs::read(&replaced).unwrap(), b"replacement");
    assert_eq!(fs::read(root.join("generated/added")).unwrap(), b"added");
    assert!(!root.join("generated/obsolete").exists());
    assert_eq!(
        fs::read(root.join("generated/unchanged")).unwrap(),
        b"unchanged"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            fs::metadata(&replaced).unwrap().permissions().mode() & 0o777,
            0o640
        );
        let mut bytes = Vec::new();
        io::Read::read_to_end(&mut original, &mut bytes).unwrap();
        assert_eq!(bytes, b"original");
    }
}
