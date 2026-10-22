use super::*;

#[test]
fn the_open_form_makes_a_binding_only_when_it_holds_something_to_open() {
    // (case, the form, what it stores)
    let form = |kind, app: &str, path: &str, web: &str| HotOpenForm {
        kind,
        app: app.to_owned(),
        path: path.to_owned(),
        web: web.to_owned(),
    };
    let cases: Vec<(&str, HotOpenForm, Option<HotItem>)> = vec![
        ("no app picked", form(HotOpenKind::App, "", "", ""), None),
        (
            "an app",
            form(HotOpenKind::App, "org.quire.Files", "", ""),
            Some(HotItem::App("org.quire.Files".to_owned())),
        ),
        (
            "a file uses the path, not the app",
            form(HotOpenKind::File, "org.quire.Files", "/a/b.txt", ""),
            Some(HotItem::Path("/a/b.txt".to_owned())),
        ),
        (
            "a folder not chosen yet",
            form(HotOpenKind::Folder, "", "", ""),
            None,
        ),
        (
            "an address without a scheme",
            form(HotOpenKind::Web, "", "", " example.org/a "),
            Some(HotItem::Web("https://example.org/a".to_owned())),
        ),
        (
            "words are no address",
            form(HotOpenKind::Web, "", "", "not an address"),
            None,
        ),
    ];
    for (case, form, want) in cases {
        assert_eq!(form.item(), want, "{case}");
        assert_eq!(form.binding(), want.map(HotDo::Open), "{case}");
    }
}

#[test]
fn the_form_shows_what_a_corner_already_opens_and_the_app_list_narrows_by_name() {
    let item = HotItem::Path("/a/b".to_owned());
    let form = HotOpenForm::of_item(&item);
    assert_eq!((form.kind, form.item()), (HotOpenKind::File, Some(item)));
    let web = HotOpenForm::of_item(&HotItem::Web("https://example.org".to_owned()));
    assert_eq!(web.kind, HotOpenKind::Web);
    let apps: Vec<HotApp> = [
        ("b.Files", "Files"),
        ("a.Mail", "Mail"),
        ("c.Blank", ""),
        ("d.Notes", "Quick Notes"),
    ]
    .into_iter()
    .map(|(id, name)| HotApp {
        id: id.to_owned(),
        name: name.to_owned(),
    })
    .collect();
    let ids = |query: &str| -> Vec<String> {
        apps_matching(&apps, query)
            .into_iter()
            .map(|app| app.id)
            .collect()
    };
    assert_eq!(
        ids(""),
        ["b.Files", "a.Mail", "d.Notes"],
        "by name, nameless left out"
    );
    assert_eq!(ids("NOTES quick"), ["d.Notes"]);
    assert_eq!(ids("zzz"), Vec::<String>::new());
}

#[test]
fn only_web_addresses_with_a_host_are_taken() {
    // (typed, taken)
    let cases: &[(&str, Option<&str>)] = &[
        ("https://example.org", Some("https://example.org")),
        ("http://example.org/a?b=c", Some("http://example.org/a?b=c")),
        ("example.org", Some("https://example.org")),
        ("localhost", None),
        ("", None),
        ("ftp://example.org", None),
        ("file:///etc/passwd", None),
        ("https://", None),
        ("https:///path", None),
        ("a b.org", None),
    ];
    for (typed, want) in cases {
        assert_eq!(web_address(typed).as_deref(), *want, "{typed:?}");
    }
    assert_eq!(command_binding("  ls  "), Some(command("ls")));
    assert_eq!(command_binding("   "), None);
}
