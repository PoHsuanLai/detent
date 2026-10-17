//! The install path puts the intents manifest where intentd reads it, beside the D-Bus service
//! file that starts the app. Staged under a scratch `DESTDIR`; nothing reaches the real system.

use detent_testkit::Scratch;
use std::path::{Path, PathBuf};
use std::process::Command;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn install_into(scratch: &Scratch) -> PathBuf {
    install_under(scratch, "/usr")
}

/// The installer run with `PREFIX=prefix` into the scratch stage; the tree under the prefix.
fn install_under(scratch: &Scratch, prefix: &str) -> PathBuf {
    let stage = scratch.join("stage");
    let status = Command::new("bash")
        .arg(repo().join("scripts/install.sh"))
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("DESTDIR", &stage)
        .env("PREFIX", prefix)
        .env("DETENT_BIN", env!("CARGO_BIN_EXE_detent"))
        .status()
        .expect("bash runs the installer");
    assert!(status.success(), "the installer failed");
    stage.join(prefix.trim_start_matches('/'))
}

#[test]
fn the_manifest_is_installed_where_intentd_reads_it() {
    let scratch = Scratch::under(TMP, "install-manifest");
    let usr = install_into(&scratch);
    let installed = usr.join("share/quire/intents/org.quire.Settings.toml");
    let shipped = repo().join("dist/intents/org.quire.Settings.toml");
    assert_eq!(
        std::fs::read_to_string(&installed).expect("installed"),
        std::fs::read_to_string(shipped).expect("shipped")
    );
    let table: toml::Table = std::fs::read_to_string(&installed)
        .expect("installed")
        .parse()
        .expect("a TOML manifest");
    assert_eq!(
        table["app"].as_str(),
        Some("org.quire.Settings"),
        "the file is named for the app it declares"
    );
}

#[test]
fn the_service_file_names_the_app_and_the_installed_binary() {
    let scratch = Scratch::under(TMP, "install-service");
    let usr = install_into(&scratch);
    let service =
        std::fs::read_to_string(usr.join("share/dbus-1/services/org.quire.Settings.service"))
            .expect("the service file is installed beside the manifest");
    assert!(service.contains("Name=org.quire.Settings\n"), "{service}");
    assert!(service.contains("Exec=/usr/bin/detent\n"), "{service}");
    assert!(
        service.contains("SystemdService=org.quire.Settings.service\n"),
        "{service}"
    );
    assert!(usr.join("bin/detent").is_file());
    assert!(
        usr.join("share/applications/org.quire.Settings.desktop")
            .is_file()
    );
}

#[test]
fn the_user_unit_is_the_one_the_service_file_names() {
    let scratch = Scratch::under(TMP, "install-unit");
    let usr = install_into(&scratch);
    let unit = std::fs::read_to_string(usr.join("lib/systemd/user/org.quire.Settings.service"))
        .expect("the user unit is installed");
    assert!(unit.contains("Type=dbus\n"), "{unit}");
    assert!(unit.contains("BusName=org.quire.Settings\n"), "{unit}");
    assert!(unit.contains("ExecStart=/usr/bin/detent\n"), "{unit}");
}

/// The variables peephole-core's `LOADER_VARIABLES` names: a unit that leaves any of them set is
/// not given the Settings role in full proof.
const LOADER_VARIABLES: [&str; 22] = [
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "LD_AUDIT",
    "GCONV_PATH",
    "GIO_EXTRA_MODULES",
    "GTK_MODULES",
    "GTK_PATH",
    "GST_PLUGIN_PATH",
    "GST_PLUGIN_SYSTEM_PATH",
    "LIBGL_DRIVERS_PATH",
    "MESA_LOADER_DRIVER_OVERRIDE",
    "__EGL_VENDOR_LIBRARY_DIRS",
    "__EGL_VENDOR_LIBRARY_FILENAMES",
    "GBM_BACKENDS_PATH",
    "LIBVA_DRIVERS_PATH",
    "VK_ICD_FILENAMES",
    "VK_DRIVER_FILES",
    "VK_ADD_DRIVER_FILES",
    "VK_LAYER_PATH",
    "VK_ADD_LAYER_PATH",
    "VK_INSTANCE_LAYERS",
    "VK_LOADER_LAYERS_ENABLE",
];

#[test]
fn the_user_unit_clears_every_loader_variable() {
    let scratch = Scratch::under(TMP, "install-unit-loader");
    let usr = install_into(&scratch);
    let unit = std::fs::read_to_string(usr.join("lib/systemd/user/org.quire.Settings.service"))
        .expect("the user unit is installed");
    let unset: Vec<&str> = unit
        .lines()
        .filter_map(|line| line.strip_prefix("UnsetEnvironment="))
        .flat_map(str::split_whitespace)
        .collect();
    let kept: Vec<&str> = LOADER_VARIABLES
        .into_iter()
        .filter(|name| !unset.contains(name))
        .collect();
    assert!(kept.is_empty(), "not unset: {kept:?}");
}

#[test]
fn a_prefix_with_characters_sed_reads_as_syntax_is_written_as_it_is() {
    let scratch = Scratch::under(TMP, "install-odd-prefix");
    let prefix = "/opt/a|b&c\\d";
    let root = install_under(&scratch, prefix);
    let service =
        std::fs::read_to_string(root.join("share/dbus-1/services/org.quire.Settings.service"))
            .expect("the service file");
    assert!(
        service.contains(&format!("Exec={prefix}/bin/detent\n")),
        "{service}"
    );
    let unit = std::fs::read_to_string(root.join("lib/systemd/user/org.quire.Settings.service"))
        .expect("the unit");
    assert!(
        unit.contains(&format!("ExecStart={prefix}/bin/detent\n")),
        "{unit}"
    );
}
