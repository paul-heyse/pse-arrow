// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Materialize sealed provider evidence in Cargo's reusable, writable output directory.

use std::{
    fs, io,
    io::Write,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn materialize(source: &Path, destination: &Path) -> io::Result<()> {
    let bytes = fs::read(source)?;
    match fs::read(destination) {
        Ok(current) if current == bytes => {
            // Older builds copied the source's sealed mode. Repair the output alone,
            // preserving its content and modification time.
            let mut permissions = fs::metadata(destination)?.permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                permissions.set_mode(permissions.mode() | 0o200);
            }
            #[cfg(not(unix))]
            permissions.set_readonly(false);
            fs::set_permissions(destination, permissions)?;
            return Ok(());
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::other("receipt output has no parent"))?;
    let (temporary, mut file) = loop {
        let path = parent.join(format!(
            ".pse-receipt-{}-{}",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => break (path, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, destination)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(all(test, unix))]
mod tests {
    use super::materialize;
    use std::{fs, os::unix::fs::PermissionsExt, sync::atomic::Ordering};

    #[test]
    fn sealed_receipt_repeated_and_changed_outputs() {
        let directory = std::env::temp_dir().join(format!(
            "pse-receipt-test-{}-{}",
            std::process::id(),
            super::NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).expect("disposable directory");
        let source = directory.join("sealed.json");
        let destination = directory.join("cargo.json");
        fs::write(&source, b"verified first receipt").expect("source");
        fs::set_permissions(&source, fs::Permissions::from_mode(0o444)).expect("seal");
        materialize(&source, &destination).expect("first materialization");
        let first_time = fs::metadata(&destination)
            .expect("output metadata")
            .modified()
            .expect("mtime");
        materialize(&source, &destination).expect("repeated materialization");
        assert_eq!(
            fs::metadata(&destination)
                .expect("output")
                .modified()
                .expect("mtime"),
            first_time
        );
        fs::set_permissions(&destination, fs::Permissions::from_mode(0o444))
            .expect("legacy output");
        materialize(&source, &destination).expect("repair unchanged sealed output");
        assert_eq!(
            fs::metadata(&destination)
                .expect("output")
                .modified()
                .expect("mtime"),
            first_time
        );
        assert_ne!(
            fs::metadata(&destination)
                .expect("output")
                .permissions()
                .mode()
                & 0o200,
            0
        );
        fs::set_permissions(&destination, fs::Permissions::from_mode(0o444))
            .expect("old sealed output");
        fs::remove_file(&source).expect("replace disposable source");
        fs::write(&source, b"verified changed receipt").expect("changed source");
        fs::set_permissions(&source, fs::Permissions::from_mode(0o444)).expect("seal replacement");
        materialize(&source, &destination).expect("replace changed sealed output");
        assert_eq!(
            fs::read(&destination).expect("output bytes"),
            fs::read(&source).expect("source bytes")
        );
        assert_eq!(
            fs::metadata(&source)
                .expect("source metadata")
                .permissions()
                .mode()
                & 0o777,
            0o444
        );
        assert_ne!(
            fs::metadata(&destination)
                .expect("output")
                .permissions()
                .mode()
                & 0o200,
            0
        );
        fs::remove_dir_all(directory).expect("remove disposable directory");
    }
}
