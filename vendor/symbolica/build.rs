use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn git_command(directory: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(directory)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR");
    command
}

// A vendored crate or archive must not inherit its consumer's repository identity
// or rebuild dependencies. A local .git file also admits genuine linked worktrees.
fn source_repository(directory: &Path) -> Option<PathBuf> {
    let directory = directory.canonicalize().ok()?;
    if !directory.join(".git").exists() {
        return None;
    }
    let output = git_command(&directory)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8(output.stdout).ok()?;
    let root = Path::new(root.trim_end_matches(['\r', '\n']))
        .canonicalize()
        .ok()?;
    (root == directory).then_some(directory)
}

pub(crate) fn git_metadata_paths(directory: &Path) -> Vec<PathBuf> {
    let Some(directory) = source_repository(directory) else {
        return Vec::new();
    };
    ["HEAD", "refs", "packed-refs"]
        .into_iter()
        .filter_map(|entry| {
            let output = git_command(&directory)
                .args(["rev-parse", "--path-format=absolute", "--git-path", entry])
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let path = PathBuf::from(
                String::from_utf8(output.stdout)
                    .ok()?
                    .trim_end_matches(['\r', '\n']),
            );
            path.exists().then_some(path)
        })
        .collect()
}

fn get_git_version(directory: &Path) -> Option<String> {
    let directory = source_repository(directory)?;
    let output = git_command(&directory)
        .args(["describe", "--tags", "--always"])
        .output()
        .ok()?;

    if output.status.success() {
        let git_desc = String::from_utf8(output.stdout).ok()?;
        Some(git_desc.trim().to_string())
    } else {
        None
    }
}

fn build_directives(directory: &Path, cargo_version: &str) -> Vec<String> {
    let version = get_git_version(directory).unwrap_or_else(|| cargo_version.to_owned());
    let mut directives = vec![format!(
        "cargo:rustc-env=SYMBOLICA_VERSION=symbolica-{version}"
    )];
    for path in git_metadata_paths(directory) {
        directives.push(format!("cargo:rerun-if-changed={}", path.display()));
    }
    directives.push("cargo:rerun-if-changed=build.rs".into());
    directives
}

fn main() {
    let cargo_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let directory = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("Cargo supplies CARGO_MANIFEST_DIR");
    for directive in build_directives(&directory, &cargo_version) {
        println!("{directive}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "symbolica-build-test-{}-{}-{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn git(directory: &Path, args: &[&str]) -> String {
        let output = git_command(directory)
            .args(["-c", "tag.gpgSign=false"])
            .args(args).output().unwrap();
        assert!(output.status.success(), "git {args:?}: {output:?}");
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    fn commit(directory: &Path) {
        git(directory, &[
            "-c", "user.name=Build Test", "-c", "user.email=build@example.invalid",
            "-c", "commit.gpgsign=false", "commit", "--quiet", "--allow-empty", "-m", "fixture",
        ]);
    }
    fn repository(directory: &Path) {
        git(directory, &["init", "--quiet"]);
        commit(directory);
    }
    fn release_directives() -> Vec<String> {
        vec![
            "cargo:rustc-env=SYMBOLICA_VERSION=symbolica-3.0.1".into(),
            "cargo:rerun-if-changed=build.rs".into(),
        ]
    }

    #[test]
    fn vendored_source_does_not_follow_consumer_commits_or_metadata() {
        let consumer = Directory::new();
        repository(&consumer.0);
        git(&consumer.0, &["tag", "consumer-v1"]);
        let supplier = consumer.0.join("vendor").join("symbolica");
        fs::create_dir_all(&supplier).unwrap();
        fs::write(supplier.join("build.rs"), "// supplier fixture").unwrap();
        let before = build_directives(&supplier, "3.0.1");
        let consumer_before = git(&consumer.0, &["rev-parse", "HEAD"]);
        commit(&consumer.0);
        assert_ne!(consumer_before, git(&consumer.0, &["rev-parse", "HEAD"]));
        assert_eq!(before, release_directives());
        assert_eq!(build_directives(&supplier, "3.0.1"), before);
        assert!(git_metadata_paths(&supplier).is_empty());
        assert!(get_git_version(&supplier).is_none());
    }

    #[test]
    fn standalone_supplier_keeps_git_version_and_metadata_dependencies() {
        let supplier = Directory::new();
        repository(&supplier.0);
        git(&supplier.0, &["tag", "supplier-v1"]);
        assert_eq!(get_git_version(&supplier.0).as_deref(), Some("supplier-v1"));
        let paths = git_metadata_paths(&supplier.0);
        assert!(paths.contains(&supplier.0.join(".git").join("HEAD")));
        assert!(paths.contains(&supplier.0.join(".git").join("refs")));
        assert_eq!(build_directives(&supplier.0, "3.0.1")[0], "cargo:rustc-env=SYMBOLICA_VERSION=symbolica-supplier-v1");
        commit(&supplier.0);
        assert_ne!(get_git_version(&supplier.0).as_deref(), Some("supplier-v1"));
        git(&supplier.0, &["pack-refs", "--all"]);
        assert!(git_metadata_paths(&supplier.0).contains(&supplier.0.join(".git").join("packed-refs")));
    }

    #[test]
    fn linked_supplier_worktree_keeps_private_head_and_shared_refs() {
        let supplier = Directory::new();
        repository(&supplier.0);
        git(&supplier.0, &["tag", "supplier-v1"]);
        let directory = Directory::new();
        let linked = directory.0.join("linked");
        git(&supplier.0, &["worktree", "add", "--quiet", "--detach", linked.to_str().unwrap()]);
        assert!(linked.join(".git").is_file());
        assert_eq!(get_git_version(&linked).as_deref(), Some("supplier-v1"));
        let head = PathBuf::from(git(&linked, &["rev-parse", "--path-format=absolute", "--git-path", "HEAD"]));
        let paths = git_metadata_paths(&linked);
        assert!(paths.contains(&head));
        assert!(paths.contains(&supplier.0.join(".git").join("refs")));
        assert_ne!(head, supplier.0.join(".git").join("HEAD"));
    }

    #[test]
    fn archived_source_uses_declared_release_and_only_build_script_dependency() {
        let archive = Directory::new();
        assert_eq!(build_directives(&archive.0, "3.0.1"), release_directives());
        assert!(get_git_version(&archive.0).is_none());
        assert!(git_metadata_paths(&archive.0).is_empty());
    }

    #[test]
    fn foreign_git_location_overrides_cannot_supply_identity_or_watch_paths() {
        const SUPPLIER: &str = "SYMBOLICA_BUILD_TEST_SUPPLIER_DIR";
        if let Some(path) = std::env::var_os(SUPPLIER) {
            let supplier = PathBuf::from(path);
            assert_eq!(get_git_version(&supplier).as_deref(), Some("supplier-v1"));
            let metadata = git_metadata_paths(&supplier);
            assert!(!metadata.is_empty());
            assert!(metadata.iter().all(|path| path.starts_with(&supplier)));
            return;
        }
        let supplier = Directory::new();
        repository(&supplier.0);
        git(&supplier.0, &["tag", "supplier-v1"]);
        let consumer = Directory::new();
        repository(&consumer.0);
        git(&consumer.0, &["tag", "consumer-v1"]);
        // Isolate environment changes in a child test process so concurrent
        // tests never see repository overrides belonging to another fixture.
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tests::foreign_git_location_overrides_cannot_supply_identity_or_watch_paths"])
            .env(SUPPLIER, &supplier.0)
            .env("GIT_DIR", consumer.0.join(".git"))
            .env("GIT_WORK_TREE", &consumer.0)
            .env("GIT_COMMON_DIR", consumer.0.join(".git"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }

    #[cfg(unix)]
    #[test]
    fn unsupported_non_unicode_source_path_falls_back_without_consumer_binding() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let directory = Directory::new();
        let supplier = directory.0.join(OsString::from_vec(b"supplier-\xff".to_vec()));
        fs::create_dir(&supplier).unwrap();
        repository(&supplier);
        assert_eq!(build_directives(&supplier, "3.0.1"), release_directives());
    }
}
