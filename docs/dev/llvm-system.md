# System LLVM selection

`scripts/llvm_system.py` prepares the existing LLVM 23.1.2 installation for
`/opt/llvm-23.1.2-z3-5.1`. The default is a dry run. The home installation is retained.

From the repository root:

```bash
just llvm-system --plan
sudo python3 scripts/llvm_system.py --apply --source "$(llvm-config --prefix)"
systemctl --user daemon-reload
python3 scripts/llvm_system.py --verify
```

Open a new login session for the PAM environment changes to take effect. Existing
processes keep their inherited environment. Repository commands also discover
`/opt/llvm-current`, so existing agent sessions can use the selected compiler without
depending on shell startup files. `PSE_LLVM_PREFIX` explicitly selects another prefix.
The selection aligns `PATH`, `CLANG_PATH`, `LIBCLANG_PATH`, and `LLVM_CONFIG_PATH`.
It does not globally set `CC`, `CXX`, `LD_LIBRARY_PATH`, or bindgen argument overrides.
Native recipes derive resource headers from the selected Clang.

The installer copies to a private staging directory, bundles the already resolved Z3
shared library, and rewrites dynamic ELF RUNPATHs to origin-relative paths. It checks
`llvm-config` prefix/libdir, Clang resource headers, shared-library resolution,
libclang loading, and C/C++ compilation and execution. The C link uses the existing
mold selection as a relocation smoke check; no linker comparison runs. A failed
relocation prevents switching the system selection. The script does not substitute a
distribution LLVM or launch an unqualified LLVM rebuild.

After staging succeeds, the installation becomes root-owned and is published under
its versioned name. The script atomically updates `/opt/llvm-current`, the three
`/usr/local/bin` tool links, `/etc/environment`, and
`/etc/environment.d/60-pse-llvm.conf`. Other environment settings and PATH entries are
preserved. The `environment.d` file covers systemd user-service environments.
`--verify` checks noninteractive, login, retained-standard-PATH, and user-service
execution. Run verification as the logged-in user, rather than through sudo.

System application prints a rollback command and retains the old files and links in
`/var/backups/pse-llvm/<timestamp>`:

```bash
sudo python3 scripts/llvm_system.py --rollback /var/backups/pse-llvm/<timestamp>
systemctl --user daemon-reload
```

Rollback restores original file bytes, modes and owners, previous symlink targets,
and originally absent paths. It refuses to overwrite later administrator changes.
It retains the versioned installation and the home installation. Open a new login
session after rollback as well.

**Tested:** a disposable relocation of this host's existing installation passed
version/prefix/resource checks, LLVM/libclang/Z3 resolution, libclang loading, and
C/C++ compilation/execution. `python3 -m unittest scripts.tests.test_llvm_system -v`
passed six isolated tests, zero failures against baseline zero. After maintainer
application, `python3 scripts/llvm_system.py --verify` passed against the installed
`/opt` prefix, including login, noninteractive, standard-PATH and user-service
execution. The script requires Python 3.11+, `patchelf`, `readelf`, and the existing
native linker/compiler prerequisites.
