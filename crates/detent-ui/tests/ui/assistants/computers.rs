//! Your computers: the lenders, adding a server, removing, and the guests asking to borrow.

use super::*;

#[test]
fn only_the_persons_own_computers_are_listed_and_there_is_no_address_to_type() {
    let c = computers("computers-list", vec![LAB_COMPUTER], vec![], vec![]);
    let text = group_text(&c.harness);
    for want in ["Your computers", "lab", "Qwen", "Ready", "Add\u{2026}"] {
        assert!(text.contains(want), "{want} in {text}");
    }
    for not in ["Acme One", "Local", "me@acme.example"] {
        assert!(
            !text.contains(not),
            "{not} is another kind of place: {text}"
        );
    }
    assert_eq!(c.harness.count(&format!("{GROUP} input")), 0);
    assert_eq!(
        c.fake.changes(),
        Vec::<String>::new(),
        "reading changes nothing"
    );
}

#[test]
fn add_lists_the_lenders_and_adding_one_moves_it_into_the_list() {
    let mut c = computers("computers-add", vec![LAB_COMPUTER], vec![STUDIO], vec![]);
    press(&mut c.harness, "Add a computer");
    settle(&mut c.harness, "the lender", |h| {
        group_text(h).contains("Llama. Say yes on studio when it asks.")
    });
    press(&mut c.harness, "Add studio");
    settle(&mut c.harness, "studio in the list", |_| {
        c.fake.changes() == ["add n1"]
    });
    settle(&mut c.harness, "the empty Add list", |h| {
        group_text(h).contains("None of your other computers is lending its models")
    });
    let text = group_text(&c.harness);
    assert_eq!(
        text.matches("studio").count(),
        1,
        "one row, not two: {text}"
    );
}

const SERVER_SHEET: &str = "[data-system=\"add-server\"]";

#[test]
fn a_server_is_added_from_its_sheet_and_shows_in_the_list_with_remove() {
    let mut c = computers_with_models("computers-server", vec![LAB_COMPUTER]);
    press(&mut c.harness, "Add a server");
    settle(&mut c.harness, "the sheet", |h| h.count(SERVER_SHEET) == 1);
    let sheet = c.harness.text_of(SERVER_SHEET).unwrap_or_default();
    assert!(
        sheet.contains("The server must answer on this computer."),
        "{sheet}"
    );
    settle(&mut c.harness, "the models", |h| {
        h.count("#server-model") == 1
    });
    type_into(&mut c.harness, "#server-name", "studio");
    type_into(&mut c.harness, "#server-port", "8080");
    click(&mut c.harness, "#server-add");
    settle(&mut c.harness, "the server in the list", |h| {
        h.count(SERVER_SHEET) == 0 && group_text(h).contains("studio")
    });
    assert_eq!(
        c.fake.changes(),
        ["add_server studio qwen port=8080 key=no"]
    );
    assert_eq!(
        c.harness
            .count(&format!("{GROUP} .ds-button[aria-label=\"Remove studio\"]")),
        1
    );
}

#[test]
fn a_sheet_that_asks_for_nothing_sendable_tells_why_and_calls_nothing() {
    let mut c = computers_with_models("computers-server-bad", vec![LAB_COMPUTER]);
    press(&mut c.harness, "Add a server");
    settle(&mut c.harness, "the sheet", |h| h.count(SERVER_SHEET) == 1);
    // The models arrive from the fake on a thread and grow the sheet; on a loaded machine they
    // came after the click had been aimed, so the click waits for them, then for the sheet's
    // slide to hold still.
    settle(&mut c.harness, "the models", |h| {
        h.count("#server-model") == 1
    });
    click_when_still(&mut c.harness, "#server-add");
    settle(&mut c.harness, "the sentence", |h| {
        h.text_of(SERVER_SHEET)
            .unwrap_or_default()
            .contains("Give the server a name.")
    });
    assert_eq!(c.fake.changes(), Vec::<String>::new());
}

#[test]
fn a_refused_server_keeps_the_sheet_open_with_the_services_sentence() {
    let mut c = computers_with_models("computers-server-refused", vec![LAB_COMPUTER]);
    press(&mut c.harness, "Add a server");
    settle(&mut c.harness, "the sheet", |h| h.count(SERVER_SHEET) == 1);
    settle(&mut c.harness, "the models", |h| {
        h.count("#server-model") == 1
    });
    type_into(&mut c.harness, "#server-name", "studio");
    type_into(&mut c.harness, "#server-port", "8080");
    c.fake.refuse_next("NotAnswering");
    click(&mut c.harness, "#server-add");
    settle(&mut c.harness, "the sentence", |h| {
        h.text_of(SERVER_SHEET)
            .unwrap_or_default()
            .contains("The sentence for NotAnswering.")
    });
    assert_eq!(c.harness.count(SERVER_SHEET), 1);
}

#[test]
fn remove_asks_for_that_computer_and_it_leaves_the_list() {
    let mut c = computers("computers-remove", vec![LAB_COMPUTER], vec![], vec![]);
    press(&mut c.harness, "Remove lab");
    settle(&mut c.harness, "the row to go", |h| {
        !group_text(h).contains("Qwen")
    });
    assert_eq!(c.fake.changes(), ["remove lab"]);
    assert!(group_text(&c.harness).contains("Computers you add appear here"));
}

#[test]
fn a_guest_is_allowed_denied_and_forgotten() {
    let mut c = computers("computers-guests", vec![], vec![], vec![LAPTOP]);
    let asking = "Asking to use this computer's models";
    assert!(group_text(&c.harness).contains(asking));
    press(&mut c.harness, "Allow laptop");
    settle(&mut c.harness, "approved", |h| {
        group_text(h).contains("Can use this computer's models")
    });
    press(&mut c.harness, "Don\u{2019}t Allow laptop");
    settle(&mut c.harness, "denied", |h| {
        group_text(h).contains("Not allowed to use this computer's models")
    });
    press(&mut c.harness, "Forget laptop");
    settle(&mut c.harness, "forgotten", |h| {
        !group_text(h).contains("laptop")
    });
    assert_eq!(
        c.fake.changes(),
        ["answer n3 allow", "answer n3 deny", "forget n3"]
    );
}

#[test]
fn each_refusal_reads_as_a_sentence_and_changes_nothing() {
    // The service answers any change with the refusal it is told to, so one button provokes all four.
    let mut c = computers("computers-refused", vec![], vec![STUDIO], vec![]);
    press(&mut c.harness, "Add a computer");
    settle(&mut c.harness, "the lender", |h| {
        group_text(h).contains("Say yes on studio")
    });
    // The service's own sentence is what the person reads.
    const REFUSALS: &[&str] = &[
        "NotOnTailscale",
        "NotAnswering",
        "NotAsking",
        "TooManyAsking",
    ];
    for refusal in REFUSALS {
        let sentence = format!("The sentence for {refusal}.");
        c.fake.refuse_next(refusal);
        press(&mut c.harness, "Add studio");
        settle(&mut c.harness, refusal, |h| {
            h.html()
                .replace("&#39;", "'")
                .replace('\u{2019}', "'")
                .contains(&sentence)
        });
    }
    assert_eq!(
        c.fake.changes(),
        Vec::<String>::new(),
        "a refusal changes nothing"
    );
}

#[test]
fn without_inferd_the_group_says_so_and_nothing_else_is_blocked() {
    let bus = PrivateBus::start("computers-absent");
    let sandbox = Sandbox::new("computers-absent").on_bus(&bus.address);
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the notice", |h| {
        h.count("[data-computers=\"unavailable\"]") == 1
    });
    assert!(
        group_text_of(&harness, "[data-computers=\"unavailable\"]")
            .contains("The assistant is not running")
    );
}

fn group_text_of(harness: &Harness, selector: &str) -> String {
    harness.text_of(selector).unwrap_or_default()
}

#[test]
fn a_computer_that_starts_asking_appears_when_inferd_says_the_guests_changed() {
    let mut c = computers("computers-signal", vec![], vec![], vec![]);
    assert!(!group_text(&c.harness).contains("laptop"));
    c._rt.block_on(c.fake.asks(LAPTOP));
    settle(&mut c.harness, "the guest", |h| {
        group_text(h).contains("Asking to use this computer's models")
    });
    assert_eq!(
        c.fake.changes(),
        Vec::<String>::new(),
        "nobody was asked to change anything"
    );
}
