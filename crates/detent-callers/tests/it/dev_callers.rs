//! `detent --dev-callers FILE` (feature `dev-callers`): a stand-in a checkout built is let in as the
//! service its row names, at `Proof::Dev`, only in a build with the feature, and only as that role.
//! Scratch folders and made-up process facts only; nothing reads the desktop's table or `/proc`.

use detent_callers::{Proof, SERVICE_PROOF, allowing, roles};
use peephole_bus::peephole_core::{
    CgroupPath, Exe, ExePath, Facts, FileTrust, Pid, Pin, SandboxFacts, Standing, Table, Uid,
    Writer, judge,
};

/// Where a checkout builds accountd's stand-in: an image the person can write.
const STAND_IN: &str = "/home/p/proj/target/debug/accountd_standin";
const SCOPE: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/run-u7.scope";
const ROWS: &str = "[[peer]]\napp = \"org.quire.Accounts\"\nrole = \"accounts\"\nexe = [\"/home/p/proj/target/debug/accountd_standin\"]\n";

/// A process running `image`, which its owner can write, in no unit.
fn facts(image: &str) -> Facts {
    Facts {
        pid: Pid::new(4242).expect("a pid"),
        uid: Uid::new(1000),
        cgroup: CgroupPath::new(SCOPE).expect("a cgroup path"),
        exe: Exe::Read {
            path: ExePath::new(image).expect("an image path"),
            trust: FileTrust::Writable(Writer::Owner(Uid::new(1000))),
        },
        sandbox: SandboxFacts::Native,
        unit: None,
        pin: Pin::AtConnect,
    }
}

/// Whether `standing` is let in as accountd, the way the owner check asks.
fn is_accountd(standing: &Standing) -> bool {
    standing
        .holds(&allowing(roles::ACCOUNTS, SERVICE_PROOF))
        .is_ok()
}

/// A stand-in with no row is no service, in every build: an image the person can write is never one.
#[test]
fn a_stand_in_no_row_names_is_no_service() {
    let standing = judge(&facts(STAND_IN), &Table::empty());
    assert!(!is_accountd(&standing), "{standing:?}");
}

/// A release build lists one way to be allowed, the real proof; nothing below `Full` is let in.
#[cfg(not(feature = "dev-callers"))]
#[test]
fn a_build_without_the_feature_accepts_only_the_real_proof() {
    use peephole_bus::peephole_core::AppKey;
    assert_eq!(allowing(roles::ACCOUNTS, Proof::Full).len(), 1);
    let below = Standing::Service {
        app: AppKey::new("org.quire.Accounts").expect("an app"),
        role: roles::ACCOUNTS,
        proof: Proof::Exe,
        pin: Pin::AtConnect,
    };
    assert!(!is_accountd(&below));
}

#[cfg(feature = "dev-callers")]
mod with_feature {
    use super::*;
    use detent_callers::{load_dev_rows, use_dev_rows};
    use detent_testkit::Scratch;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::{Path, PathBuf};

    fn write(dir: &Path, name: &str, text: &str, mode: u32) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, text).expect("scratch file written");
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("mode set");
        path
    }

    fn standing_with(rows: &Table, image: &str) -> Standing {
        judge(&facts(image), &Table::empty().merge(rows.clone()))
    }

    /// The row names the stand-in's image: it is accountd at `Proof::Dev`, and only accountd.
    #[test]
    fn a_rows_file_naming_a_daemons_image_is_accepted_at_dev_proof_only() {
        let dir = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "dev-callers-accepts");
        let file = write(dir.path(), "rows.toml", ROWS, 0o600);
        let rows = load_dev_rows(&file).expect("the person's own rows load");

        let named = standing_with(&rows, STAND_IN);
        assert!(
            matches!(
                &named,
                Standing::Service {
                    proof: Proof::Dev,
                    ..
                }
            ),
            "{named:?}"
        );
        assert!(is_accountd(&named));
        // The real requirement alone (the release build's) does not take it.
        let real_only = [detent_callers::Requirement::new(
            roles::ACCOUNTS,
            SERVICE_PROOF,
        )];
        assert!(named.holds(&real_only).is_err());
        // Another service's name, and an image no row names, are not let in.
        assert!(named.holds(&allowing(roles::SYNC, SERVICE_PROOF)).is_err());
        let other = standing_with(&rows, "/home/p/proj/target/debug/other");
        assert!(!is_accountd(&other), "{other:?}");
    }

    /// The file is taken only when it is the person's own plain file that parses.
    #[test]
    fn only_the_persons_own_plain_unshared_file_that_parses_adds_rows() {
        let dir = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "dev-callers-files");
        let own = write(dir.path(), "own.toml", ROWS, 0o600);
        let link = dir.path().join("link.toml");
        symlink(&own, &link).expect("link made");
        let cases = [
            ("a file of the person's, mode 600", own, true),
            (
                "mode 644 (others read, nobody else writes)",
                write(dir.path(), "readable.toml", ROWS, 0o644),
                true,
            ),
            (
                "writable by the group",
                write(dir.path(), "group.toml", ROWS, 0o660),
                false,
            ),
            (
                "writable by everyone",
                write(dir.path(), "world.toml", ROWS, 0o666),
                false,
            ),
            ("a link to a good file", link, false),
            (
                "text that is not a table",
                write(dir.path(), "junk.toml", "not = [a table", 0o600),
                false,
            ),
            (
                "a row naming no image",
                write(
                    dir.path(),
                    "noexe.toml",
                    "[[peer]]\napp = \"org.quire.Accounts\"\nrole = \"accounts\"\n",
                    0o600,
                ),
                false,
            ),
            (
                "a file that is not there",
                dir.path().join("none.toml"),
                false,
            ),
            ("a folder", dir.path().to_path_buf(), false),
        ];
        for (name, path, taken) in cases {
            assert_eq!(load_dev_rows(&path).is_ok(), taken, "{name}");
        }
    }

    /// Rows that cannot be taken up are an error the start can stop on.
    #[test]
    fn rows_that_cannot_be_read_are_an_error() {
        let dir = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "dev-callers-fault");
        assert!(use_dev_rows(&dir.join("none.toml")).is_err());
    }
}
