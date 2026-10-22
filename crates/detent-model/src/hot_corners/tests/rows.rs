use super::*;

#[test]
fn a_keyed_row_is_added_once_and_removed_with_its_binding() {
    use HotModifier::{Alt, Command};
    let bindings = vec![bind(&[], lock()), bind(&[Alt], command("x"))];

    // the rows shown: none held, then each set with a binding or added, once each
    let added = vec![keys(&[Command]), keys(&[Alt])];
    assert_eq!(
        shown_rows(&bindings, &added),
        [keys(&[]), keys(&[Alt]), keys(&[Command])]
    );

    // adding: (case, keys ticked, result)
    let cases: Vec<(&str, HotKeys, Result<usize, &str>)> = vec![
        ("nothing ticked", keys(&[]), Err("Tick at least one key.")),
        (
            "a set that has a binding",
            keys(&[Alt]),
            Err("This corner already has a row for those keys."),
        ),
        (
            "a set that was added and not filled",
            keys(&[Command]),
            Err("This corner already has a row for those keys."),
        ),
        ("a new set", keys(&[Command, Alt]), Ok(2)),
    ];
    for (case, ticked, want) in cases {
        let got = add_row(&bindings, &[keys(&[Command])], ticked)
            .map(|rows| rows.len())
            .map_err(|fault| fault.words());
        assert_eq!(got, want, "{case}");
    }

    // removing the keyed row drops its binding and its added row; the row with no keys stays
    let (left, rows) = remove_row(&bindings, &added, &keys(&[Alt]));
    assert_eq!(left, [bind(&[], lock())]);
    assert_eq!(rows, [keys(&[Command])]);
    let (left, rows) = remove_row(&bindings, &added, &keys(&[]));
    assert_eq!((left, rows), (bindings, added));
}

#[test]
fn setting_a_row_replaces_that_sets_binding_and_nothing_removes_it() {
    use HotModifier::Alt;
    let start = vec![bind(&[], lock()), bind(&[Alt], command("x"))];
    // (case, keys, the pick, the bindings after)
    let cases: Vec<(&str, HotKeys, Option<HotDo>, Vec<HotBinding>)> = vec![
        (
            "replacing the row with no keys",
            keys(&[]),
            Some(HotDo::Desk(HotDesk::Launcher)),
            vec![
                bind(&[], HotDo::Desk(HotDesk::Launcher)),
                bind(&[Alt], command("x")),
            ],
        ),
        (
            "nothing removes the keyed row",
            keys(&[Alt]),
            None,
            vec![bind(&[], lock())],
        ),
        (
            "a new set joins in key order",
            keys(&[HotModifier::Command]),
            Some(command("y")),
            vec![
                bind(&[], lock()),
                bind(&[Alt], command("x")),
                bind(&[HotModifier::Command], command("y")),
            ],
        ),
    ];
    for (case, held, does, want) in cases {
        assert_eq!(with_binding(&start, &held, does), want, "{case}");
    }
    assert_eq!(binding_for(&start, &keys(&[Alt])), Some(&command("x")));
    assert_eq!(binding_for(&start, &keys(&[HotModifier::Shift])), None);
}
