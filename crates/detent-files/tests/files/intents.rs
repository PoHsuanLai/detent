//! The intents manifests, read from scratch folders.

use detent_files::read_manifests;
use detent_testkit::Scratch;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

#[test]
fn manifests_are_read_in_folder_order_and_only_the_toml_files_among_them() {
    let scratch = Scratch::under(TMP, "intents-read");
    let (first, second) = (scratch.join("first"), scratch.join("second"));
    for (dir, file, text) in [
        (&first, "org.quire.B.toml", "b"),
        (&first, "org.quire.A.toml", "a"),
        (&first, "default-grants.json", "{}"),
        (&second, "org.quire.C.toml", "c"),
    ] {
        std::fs::create_dir_all(dir).expect("dir");
        std::fs::write(dir.join(file), text).expect("write");
    }
    std::fs::create_dir_all(first.join("nested.toml")).expect("a folder named like a file");
    let missing = scratch.join("missing");
    assert_eq!(
        read_manifests(&[first, missing, second]),
        ["a", "b", "c"],
        "files by name inside a folder, folders as given, nothing else"
    );
}
