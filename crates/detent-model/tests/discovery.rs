//! Finding schemas: the real ones, and the stale ones that are skipped.

use detent_model::{StaleReason, discover};
use detent_testkit::{Scratch, seed_schemas};
use ds_settings::schema::{AppId, FilePath, Schema};

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

fn schema(app: &str, file: &str, version: u16) -> Schema {
    Schema {
        app: AppId(app.to_owned()),
        file: FilePath(file.to_owned()),
        version,
        key: Vec::new(),
    }
}

fn write(dir: &std::path::Path, name: &str, schema: &Schema) {
    std::fs::create_dir_all(dir).expect("scratch dir");
    std::fs::write(dir.join(format!("{name}.settings.toml")), schema.to_toml()).expect("write");
}

#[test]
fn the_real_schemas_are_found_in_file_name_order() {
    let scratch = Scratch::under(TMP, "discovery-real");
    let dir = seed_schemas(scratch.path());
    let found = discover(&[dir]);
    let apps: Vec<&str> = found.programs.iter().map(|p| p.app.0.as_str()).collect();
    assert_eq!(apps, ["docket", "inferd", "palmrest", "quire", "sill"]);
    assert_eq!(found.warnings, Vec::new());
}

#[test]
fn a_stale_schema_is_skipped_with_a_warning() {
    let scratch = Scratch::under(TMP, "discovery-stale");
    let dir = scratch.join("quire/settings");
    write(&dir, "good", &schema("good", "good/settings.toml", 1));
    write(&dir, "future", &schema("future", "future/settings.toml", 2));
    write(
        &dir,
        "escape",
        &schema("escape", "../../etc/passwd.toml", 1),
    );
    std::fs::write(dir.join("broken.settings.toml"), "not = [valid").expect("write");

    let found = discover(&[dir]);
    let apps: Vec<&str> = found.programs.iter().map(|p| p.app.0.as_str()).collect();
    assert_eq!(apps, ["good"], "only the good schema is kept");

    let reasons: Vec<(String, StaleReason)> = found
        .warnings
        .iter()
        .map(|w| (w.app.0.clone(), w.reason.clone()))
        .collect();
    assert_eq!(
        reasons,
        [
            (
                "escape".to_owned(),
                StaleReason::UnsafeFile("../../etc/passwd.toml".to_owned())
            ),
            ("future".to_owned(), StaleReason::UnsupportedVersion(2)),
        ],
        "the malformed file is reported by ds-settings on stderr, the rest here"
    );
}

#[test]
fn an_earlier_directory_shadows_a_later_one() {
    let scratch = Scratch::under(TMP, "discovery-shadow");
    let (home, system) = (scratch.join("home"), scratch.join("system"));
    write(&home, "acme", &schema("acme", "acme/mine.toml", 1));
    write(&system, "acme", &schema("acme", "acme/theirs.toml", 1));

    let found = discover(&[home, system]);
    assert_eq!(found.programs.len(), 1);
    assert_eq!(
        found.programs[0].file.relative().to_str(),
        Some("acme/mine.toml")
    );
    assert_eq!(found.warnings.len(), 1);
    assert_eq!(found.warnings[0].reason, StaleReason::Shadowed);
}
