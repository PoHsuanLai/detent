//! The Accounts page over three fake modules on a private bus (identities, AI keys, sync): the
//! list, one account's switches and actions, the question before a removal, and the page when
//! a module or all of them are missing.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held, Module, Policy};
use crate::support::bus::PrivateBus;
use crate::support::drive::click;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::Scheme;
use ds_harness::{Backdrop, Driver, Harness, Query};
use ds_settings::live::Served;
use ds_settings::schema::KeyPath;
use std::time::Duration;

const HOME: &str = "[aria-label=\"Manage Home cloud\"]";
const KEY: &str = "[aria-label=\"Manage Claude key\"]";
const SYNC: &str = "[aria-label=\"Manage Home sync\"]";
const WORK: &str = "[aria-label=\"Manage Work mail\"]";
const LLM: &str = "[aria-label=\"Llm\"] .ds-toggle";
const KEEP_FILES: &str = "[aria-label=\"Keep this account's files on this computer\"] .ds-toggle";
const UPLOAD_PHOTOS: &str =
    "[aria-label=\"Upload new photos from a folder on this computer to Google Photos\"] .ds-toggle";
const REVOKE: &str = "[aria-label=\"Photos allowed: llm\"] .ds-field-row-control .ds-button";
const SIGN_IN: &str = "[aria-label=\"Sign in again\"] .ds-field-row-control .ds-button";
const REMOVE: &str = "[aria-label=\"Remove account\"] .ds-field-row-control .ds-button";
const BACK: &str = ".accounts-back .ds-button";
const CONTINUE_AS: &str = ".accounts-more [aria-label=\"Continue as\u{2026}\"]";
const ADD_KEY: &str = ".accounts-more [aria-label=\"Add AI Key\u{2026}\"]";

fn accounts() -> Opening {
    Opening {
        page: Some(PageSlug("accounts".to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fake daemons answer on real threads; a loaded machine running every test at once can
    // take several seconds, so the budget is generous (15 s) and costs nothing when it settles.
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon")
}

/// accountd's identities: one that two apps use, one whose sign-in expires.
fn identities() -> Vec<Held> {
    vec![
        Held::new("home", "Home cloud", "ok", &[]).used_by(&["Photos", "Mail"]),
        Held::new("work.example", "Work mail", "needs_reauth", &[])
            .expiring("Signed out by Google on 2026-10-14")
            .signing_in("browser"),
    ]
}

/// inferd's AI key, with one permit.
fn keys() -> Vec<Held> {
    vec![
        Held::new("claude-key", "Claude key", "ok", &["llm"])
            .in_group("intelligence")
            .granting("g1", "Photos allowed: llm"),
    ]
}

/// syncd's account, with both switches off.
fn syncing() -> Vec<Held> {
    vec![Held::new("home", "Home sync", "ok", &[]).syncing()]
}

/// The page up over fake modules, showing the list.
struct World {
    accountd: Accountd,
    inferd: Accountd,
    syncd: Accountd,
    harness: Harness,
    rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    daemons: Vec<(Module, zbus::Connection, Served)>,
}

fn world(name: &str) -> World {
    world_in(name, Scheme::Light)
}

fn world_in(name: &str, scheme: Scheme) -> World {
    start(
        name,
        scheme,
        identities(),
        &[Module::Accountd, Module::Inferd, Module::Syncd],
    )
}

/// Only accountd, holding `rows`.
fn world_of(name: &str, rows: Vec<Held>, scheme: Scheme) -> World {
    start(name, scheme, rows, &[Module::Accountd])
}

/// The page with the modules in `served` running; the others are not on the bus.
fn start(name: &str, scheme: Scheme, rows: Vec<Held>, served: &[Module]) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let held = |module: Module, count: usize| if served.contains(&module) { count } else { 0 };
    let expected = held(Module::Accountd, rows.len())
        + held(Module::Inferd, keys().len())
        + held(Module::Syncd, syncing().len());
    let accountd = Accountd::new(Policy::Allow, rows);
    let inferd = Accountd::on(Module::Inferd, Policy::Allow, keys());
    let syncd = Accountd::on(Module::Syncd, Policy::Allow, syncing()).continuing_as();
    let mut daemons = Vec::new();
    for module in served {
        let fake = match module {
            Module::Accountd => &accountd,
            Module::Inferd => &inferd,
            Module::Syncd => &syncd,
        };
        let (connection, announcer) = rt.block_on(fake.serve_announcing(&bus));
        daemons.push((*module, connection, announcer));
    }
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    let mut harness = sandbox.harness_in(accounts(), scheme);
    settle(&mut harness, "the list", |h| {
        h.count(".accounts-list") == 1 || h.count(".accounts-empty") == 1
    });
    // Each module answers on its own; the list is whole once every served one has.
    settle(&mut harness, "every module's rows", |h| {
        expected == 0 || h.count(".account-row") >= expected
    });
    World {
        accountd,
        inferd,
        syncd,
        harness,
        rt,
        _bus: bus,
        daemons,
    }
}

impl World {
    /// Announces, as `module`, that the row at `path` changed to `word`.
    fn announce(&self, module: Module, path: &str, word: &str) {
        let (_, _, served) = self
            .daemons
            .iter()
            .find(|(served, ..)| *served == module)
            .expect("the module runs");
        self.rt
            .block_on(served.changed(
                &KeyPath(path.to_owned()),
                &toml::Value::String(word.to_owned()),
            ))
            .expect("announced");
    }

    /// The module goes off the bus, as when its service stops.
    fn stop(&mut self, module: Module) {
        let at = self
            .daemons
            .iter()
            .position(|(served, ..)| *served == module)
            .expect("the module runs");
        let (_, connection, _) = self.daemons.remove(at);
        self.rt
            .block_on(connection.release_name(module.bus()))
            .expect("the name is released");
    }
}

fn open(w: &mut World, button: &str) {
    click(&mut w.harness, button);
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
}

fn detail(w: &mut World) {
    open(w, HOME);
}

#[test]
fn the_list_has_a_row_per_account_of_every_module_with_its_state() {
    let w = world("accounts-list");
    let html = w.harness.html();
    for text in [
        "Home cloud",
        "Work mail",
        "Claude key",
        "Home sync",
        "Signed in",
        "Needs sign-in",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(w.harness.count(".account-row"), 4);
    assert!(
        html.contains("Sign-in keys"),
        "the client ids have a disclosure"
    );
    assert_eq!(w.harness.count(ADD_KEY), 1);
    assert_eq!(w.harness.count(CONTINUE_AS), 1);
}

#[test]
fn a_pane_with_only_accountd_shows_identities_and_no_ai_or_sync_part() {
    let w = world_of("accounts-only-accountd", identities(), Scheme::Light);
    let html = w.harness.html();
    assert_eq!(w.harness.count(".account-row"), 2);
    for text in ["Home cloud", "Work mail"] {
        assert!(html.contains(text), "{text}");
    }
    for text in ["Claude key", "Home sync"] {
        assert!(!html.contains(text), "{text}");
    }
    assert!(
        !headings(&w).iter().any(|heading| heading == "Intelligence"),
        "no AI part in the list"
    );
    assert_eq!(w.harness.count(ADD_KEY), 0);
    assert_eq!(w.harness.count(CONTINUE_AS), 0);
    assert_eq!(w.harness.count(".accounts[data-accounts=\"ready\"]"), 1);
}

#[test]
fn an_identity_lists_the_apps_using_it_read_only_and_goes_back() {
    let mut w = world("accounts-detail");
    detail(&mut w);
    let html = w.harness.html();
    for text in [
        "Apps using this identity",
        "Photos",
        "Mail",
        "Sign in again",
        "Remove account",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(
        w.harness.count("[aria-label^=\"Revoke\"]"),
        0,
        "an identity has no Revoke rows"
    );
    assert_eq!(
        w.harness.count(
            "[aria-label=\"App\"] .ds-field-row-control .ds-button, [aria-label=\"App\"] .ds-toggle"
        ),
        0,
        "the apps are a plain list"
    );
    click(&mut w.harness, BACK);
    assert_eq!(w.harness.count(".accounts-list"), 1);
    assert_eq!(w.harness.count(".accounts-detail"), 0);
}

#[test]
fn an_ai_key_shows_inferds_permits_with_revoke_and_revoking_goes_to_inferd() {
    let mut w = world("accounts-permit");
    open(&mut w, KEY);
    let html = w.harness.html();
    assert!(html.contains("Photos allowed: llm"), "{html}");
    assert_eq!(
        w.harness.attr(REVOKE, "aria-label").as_deref(),
        Some("Revoke Photos allowed: llm")
    );
    click(&mut w.harness, REVOKE);
    settle(&mut w.harness, "the revoke", |_| {
        !w.inferd.sets().is_empty()
    });
    assert_eq!(
        w.inferd.sets(),
        [(
            "accounts.claude-key.grant.g1".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
    assert!(w.accountd.sets().is_empty() && w.syncd.sets().is_empty());
    settle(&mut w.harness, "the permit to leave the page", |h| {
        !h.html().contains("Photos allowed: llm")
    });
}

#[test]
fn a_switch_sets_the_service_in_inferd_and_the_answer_stays_on_screen() {
    let mut w = world("accounts-switch");
    open(&mut w, KEY);
    assert_eq!(w.harness.attr(LLM, "aria-checked").as_deref(), Some("true"));
    click(&mut w.harness, LLM);
    settle(&mut w.harness, "the set", |_| !w.inferd.sets().is_empty());
    assert_eq!(
        w.inferd.sets(),
        [(
            "accounts.claude-key.service.llm".to_owned(),
            toml::Value::String("off".to_owned())
        )]
    );
    settle(&mut w.harness, "the switch to read off", |h| {
        h.attr(LLM, "aria-checked").as_deref() == Some("false")
    });
}

#[test]
fn the_sync_switches_are_syncds_and_set_on_and_off_in_syncds_module() {
    let mut w = world("accounts-sync");
    open(&mut w, SYNC);
    let html = w.harness.html();
    assert!(html.contains("Upload new photos from a folder on this computer to Google Photos"));
    assert!(
        html.contains("never reads or deletes anything else"),
        "the sync rows keep their help text"
    );
    assert_eq!(
        w.harness.attr(KEEP_FILES, "aria-checked").as_deref(),
        Some("false")
    );
    click(&mut w.harness, KEEP_FILES);
    settle(&mut w.harness, "the switch on", |h| {
        h.attr(KEEP_FILES, "aria-checked").as_deref() == Some("true")
    });
    settle(&mut w.harness, "the pane to be free", |h| {
        h.attr(UPLOAD_PHOTOS, "data-availability").as_deref() != Some("disabled")
    });
    click(&mut w.harness, KEEP_FILES);
    settle(&mut w.harness, "the second set", |_| {
        w.syncd.sets().len() == 2
    });
    let want = |word: &str| {
        (
            "accounts.home.sync.files".to_owned(),
            toml::Value::String(word.to_owned()),
        )
    };
    assert_eq!(w.syncd.sets(), [want("on"), want("off")]);
    assert!(
        w.accountd.sets().is_empty(),
        "accountd, which has an account of the same id, heard nothing"
    );
}

#[test]
fn continue_as_runs_syncds_row_on_one_press() {
    let mut w = world("accounts-continue-as");
    click(&mut w.harness, CONTINUE_AS);
    settle(&mut w.harness, "the action", |_| !w.syncd.sets().is_empty());
    assert_eq!(
        w.syncd.sets(),
        [("sync.add".to_owned(), toml::Value::Boolean(true))]
    );
    assert!(w.accountd.sets().is_empty() && w.inferd.sets().is_empty());
}

#[test]
fn one_module_vanishing_leaves_the_others_shown() {
    let mut w = world("accounts-vanish");
    assert_eq!(w.harness.count(".account-row"), 4);
    w.stop(Module::Syncd);
    settle(&mut w.harness, "sync to leave the list", |h| {
        !h.html().contains("Home sync")
    });
    let html = w.harness.html();
    for text in ["Home cloud", "Work mail", "Claude key"] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(w.harness.count(CONTINUE_AS), 0);
    assert_eq!(w.harness.count(".accounts[data-accounts=\"ready\"]"), 1);
}

#[test]
fn the_expiry_is_a_read_only_line_in_the_modules_words() {
    let mut w = world("accounts-expires");
    open(&mut w, WORK);
    let row = "[aria-label=\"Sign-in\"]";
    assert_eq!(w.harness.count(row), 1, "{}", w.harness.html());
    let html = w.harness.html();
    assert!(html.contains("Signed out by Google on 2026-10-14"));
    assert!(html.contains("starts another 7"));
    for control in [".ds-toggle", ".ds-button"] {
        assert_eq!(
            w.harness.count(&format!("{row} {control}")),
            0,
            "the expiry row has no {control}"
        );
    }
    // An account whose sign-in does not expire has no such row.
    click(&mut w.harness, BACK);
    detail(&mut w);
    assert_eq!(w.harness.count(row), 0);
}

#[test]
fn a_session_permit_row_comes_and_goes_with_inferds_announcement() {
    let mut w = world("accounts-session-grant");
    open(&mut w, KEY);
    let sentence = "Claude Code can use Llm (this session)";
    let (grant, path) = ("s1", "accounts.claude-key.grant.s1");
    w.inferd
        .accounts
        .lock()
        .expect("accounts")
        .iter_mut()
        .for_each(|held| held.grants.push((grant.to_owned(), sentence.to_owned())));
    w.announce(Module::Inferd, path, "added");
    settle(&mut w.harness, "the session row", |h| {
        h.html().contains(sentence)
    });
    w.inferd
        .accounts
        .lock()
        .expect("accounts")
        .iter_mut()
        .for_each(|held| held.grants.retain(|(g, _)| g != grant));
    w.announce(Module::Inferd, path, "removed");
    settle(&mut w.harness, "the session row to go", |h| {
        !h.html().contains(sentence)
    });
}

#[test]
fn an_identity_shows_no_sync_rows() {
    let mut w = world("accounts-no-sync");
    open(&mut w, WORK);
    let html = w.harness.html();
    assert!(!html.contains("Upload new photos from a folder on this computer to Google Photos"));
    assert!(!html.contains("on this computer"));
}

#[test]
fn sign_in_again_runs_the_action_in_accountd_at_once() {
    let mut w = world("accounts-run");
    detail(&mut w);
    click(&mut w.harness, SIGN_IN);
    settle(&mut w.harness, "the action", |_| {
        !w.accountd.sets().is_empty()
    });
    assert_eq!(
        w.accountd.sets(),
        [(
            "accounts.home.reauth".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
}

#[test]
fn removing_an_account_asks_first_and_keeping_it_does_nothing() {
    let mut w = world("accounts-keep");
    detail(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Home cloud?")
    });
    assert!(
        w.accountd.sets().is_empty(),
        "the press alone removes nothing"
    );
    click(
        &mut w.harness,
        ".ds-alert [data-role=\"normal\"], .ds-alert .ds-button",
    );
    settle(&mut w.harness, "the question to go", |h| {
        !h.html().contains("Remove account Home cloud?")
    });
    assert!(w.accountd.sets().is_empty());
    assert_eq!(w.accountd.accounts.lock().expect("accounts").len(), 2);
}

#[test]
fn confirming_the_removal_removes_the_account_and_returns_to_the_list() {
    let mut w = world("accounts-remove");
    detail(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Home cloud?")
    });
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-role=\"destructive\"]",
    );
    settle(&mut w.harness, "the removal", |_| {
        w.accountd.accounts.lock().expect("accounts").len() == 1
    });
    assert_eq!(
        w.accountd.sets(),
        [(
            "accounts.home.remove".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
    settle(&mut w.harness, "the list", |h| {
        h.count(".accounts-list") == 1 && !h.html().contains("Home cloud")
    });
    assert!(w.harness.html().contains("Work mail"));
    assert!(
        w.harness.html().contains("Home sync"),
        "syncd's account of the same id stays"
    );
}

#[test]
fn an_account_with_a_dotted_id_opens_and_removes_by_its_whole_id() {
    let mut w = world("accounts-dotted");
    click(&mut w.harness, WORK);
    settle(&mut w.harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some("work.example")
    });
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Work mail?")
    });
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-role=\"destructive\"]",
    );
    settle(&mut w.harness, "the set", |_| !w.accountd.sets().is_empty());
    assert_eq!(w.accountd.sets()[0].0, "accounts.work.example.remove");
}

#[test]
fn no_bus_says_so_and_blocks_nothing_else_on_the_window() {
    let sandbox = Sandbox::new("accounts-no-bus");
    let harness = sandbox.harness(accounts());
    assert_eq!(
        harness.attr(".accounts", "data-accounts").as_deref(),
        Some("unavailable")
    );
    assert!(harness.html().contains("Unavailable"));
}

#[test]
fn a_bus_with_none_of_the_three_says_the_service_is_not_running() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-none");
    let mut harness = Sandbox::new("accounts-none")
        .on_bus(&bus.address)
        .harness(accounts());
    settle(&mut harness, "the pane giving up", |h| {
        h.html().contains("The accounts service is not running")
    });
    drop(rt);
}

#[test]
fn accountd_refusing_settings_is_said_in_its_own_words() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-refused");
    let fake = Accountd::new(Policy::Refuse, identities());
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = Sandbox::new("accounts-refused")
        .on_bus(&bus.address)
        .harness(accounts());
    settle(&mut harness, "the refusal", |h| {
        h.html().contains("opened from the desktop")
    });
    assert_no_developer_words(&harness);
}

#[test]
fn a_client_id_is_set_when_the_field_is_left() {
    let mut w = world("accounts-client");
    click(&mut w.harness, "#sign-in-clients .ds-disclosure-trigger");
    settle(&mut w.harness, "the field", |h| {
        h.count("[aria-label=\"microsoft client id\"] input") > 0
    });
    click(&mut w.harness, "[aria-label=\"microsoft client id\"] input");
    for letter in "abc".chars() {
        w.harness
            .send(ds_harness::Input::key(ds::prelude::ShortcutKey::Char(
                letter,
            )));
    }
    w.harness
        .send(ds_harness::Input::key(ds::prelude::ShortcutKey::Enter));
    settle(&mut w.harness, "the set", |_| !w.accountd.sets().is_empty());
    assert_eq!(
        w.accountd.sets(),
        [(
            "accounts.clients.microsoft".to_owned(),
            toml::Value::String("abc".to_owned())
        )]
    );
}

/// No service, program or file name of the desktop's insides is in what the pane says.
fn assert_no_developer_words(harness: &Harness) {
    let text = harness.text_of(".accounts").unwrap_or_default();
    assert!(
        !text.is_empty(),
        "the pane says something:\n{}",
        harness.html()
    );
    let words = [
        "accountd",
        "callers.toml",
        "inferd",
        "syncd",
        "ACP",
        "MCP",
        "OAuth",
        "daemon",
    ];
    for word in words {
        assert!(!text.contains(word), "{word} is in: {text}");
    }
}

#[test]
fn the_pane_says_no_developer_words_with_the_modules_up_or_gone() {
    let sandbox = Sandbox::new("accounts-plain-no-bus");
    assert_no_developer_words(&sandbox.harness(accounts()));
    let rt = runtime();
    let bus = PrivateBus::start("accounts-plain-none");
    let mut harness = Sandbox::new("accounts-plain-none")
        .on_bus(&bus.address)
        .harness(accounts());
    settle(&mut harness, "the pane giving up", |h| {
        h.html().contains("The accounts service is not running")
    });
    assert_no_developer_words(&harness);
    drop(rt);
    let w = world("accounts-plain-up");
    assert_no_developer_words(&w.harness);
}

const ADD: &str = ".accounts-add [aria-label=\"Add Account\u{2026}\"]";
const MINUS: &str = ".accounts-add [aria-label=\"Remove Account\"]";

#[test]
fn the_list_has_an_add_account_button_and_a_refused_ask_is_told_calmly() {
    let mut w = world("accounts-add");
    assert_eq!(w.harness.count(ADD), 1);
    // The tip is the short name; the spoken name keeps the ellipsis.
    assert_eq!(
        w.harness.attr(ADD, "aria-description").as_deref(),
        Some("Add Account")
    );
    assert_eq!(
        w.harness.attr(MINUS, "data-tip").as_deref(),
        Some("Remove Account")
    );
    click(&mut w.harness, ADD);
    settle(&mut w.harness, "the line", |h| {
        h.html()
            .contains("Could not add an account. Try again later.")
    });
    // The button is free again after the sheet ended.
    settle(&mut w.harness, "the button", |h| {
        h.attr(ADD, "data-availability").as_deref() != Some("disabled")
    });
}

#[test]
fn a_sign_in_that_ends_is_told_in_a_calm_line_and_the_rows_are_read_again() {
    let mut w = world("accounts-reauth-end");
    detail(&mut w);
    w.accountd.stop("home", "needs_reauth");
    w.announce(Module::Accountd, "accounts.home.reauth", "failed");
    settle(&mut w.harness, "the line", |h| {
        h.html()
            .contains("Could not sign in to Home cloud. Try again later.")
    });
    settle(&mut w.harness, "the rows to be read again", |h| {
        h.html().contains("Needs sign-in")
    });
}

#[test]
fn the_remove_question_adds_a_clause_about_kept_files_only_when_a_sync_row_is_on() {
    let mut w = world("accounts-remove-files");
    open(&mut w, SYNC);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Home sync?")
    });
    assert!(!w.harness.html().contains("Files kept on this computer"));
    click(
        &mut w.harness,
        ".ds-alert .ds-button:not([data-role=\"destructive\"])",
    );
    settle(&mut w.harness, "the question to go", |h| {
        !h.html().contains("Remove account Home sync?")
    });
    click(&mut w.harness, KEEP_FILES);
    settle(&mut w.harness, "the switch on", |h| {
        h.attr(KEEP_FILES, "aria-checked").as_deref() == Some("true")
    });
    settle(&mut w.harness, "the pane to be free", |h| {
        h.attr(REMOVE, "data-availability").as_deref() != Some("disabled")
    });
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html()
            .contains("Files kept on this computer for the account are deleted too.")
    });
}

#[test]
fn the_account_type_is_drawn_with_the_providers_mark() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-mark");
    let fake = Accountd::new(
        Policy::Allow,
        vec![
            Held::new("local-ai", "Ollama", "ok", &["llm"]).of_provider("ollama", "Ollama"),
            Held::new("mail", "Fastmail", "ok", &["mail"]).of_provider("fastmail", "Fastmail"),
        ],
    );
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = Sandbox::new("accounts-mark")
        .on_bus(&bus.address)
        .harness(accounts());
    settle(&mut harness, "the list", |h| h.count(".accounts-list") == 1);
    click(&mut harness, "[aria-label=\"Manage Ollama\"]");
    settle(&mut harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
    assert_eq!(
        harness.count(".ds-provider[data-kind=\"face\"]"),
        1,
        "a provider whose file gives a face is drawn from it:\n{}",
        harness.html()
    );
    click(&mut harness, BACK);
    click(&mut harness, "[aria-label=\"Manage Fastmail\"]");
    settle(&mut harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some("mail")
    });
    assert_eq!(
        harness.count(".ds-provider[data-kind=\"letter\"]"),
        1,
        "one without a face keeps its named mark"
    );
    drop(rt);
}

/// WCAG relative luminance of an 8-bit sRGB pixel.
fn luminance(pixel: [u8; 3]) -> f64 {
    let linear = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(pixel[0]) + 0.7152 * linear(pixel[1]) + 0.0722 * linear(pixel[2])
}

/// The contrast between a button's fill (the commonest colour inside it) and its label (the
/// colour furthest from the fill).
fn label_contrast(harness: &mut Harness, selector: &str) -> f64 {
    let rect = harness.rect(selector).expect("the button is laid out");
    let image = harness.render_over(Backdrop::Scheme).expect("a picture");
    let (x0, y0) = (rect.origin.x.0 as u32, rect.origin.y.0 as u32);
    let (w, h) = (rect.size.width.0 as u32, rect.size.height.0 as u32);
    let mut counts = std::collections::HashMap::new();
    let mut inside = Vec::new();
    for y in y0 + 2..y0 + h.saturating_sub(2) {
        for x in x0 + 6..x0 + w.saturating_sub(6) {
            let p = image.get_pixel(x, y).0;
            let rgb = [p[0], p[1], p[2]];
            *counts.entry(rgb).or_insert(0usize) += 1;
            inside.push(rgb);
        }
    }
    let fill = counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(rgb, _)| rgb)
        .expect("pixels");
    let far = |p: &[u8; 3]| {
        p.iter()
            .zip(fill)
            .map(|(a, b)| i32::from(*a).abs_diff(i32::from(b)))
            .sum::<u32>()
    };
    let label = inside
        .iter()
        .max_by_key(|p| far(p))
        .copied()
        .expect("pixels");
    let (hi, lo) = (
        luminance(fill).max(luminance(label)),
        luminance(fill).min(luminance(label)),
    );
    (hi + 0.05) / (lo + 0.05)
}

#[test]
fn the_remove_answer_is_legible_in_light_and_dark() {
    for (scheme, tag) in [(Scheme::Light, "light"), (Scheme::Dark, "dark")] {
        let mut w = world_in(&format!("accounts-legible-{tag}"), scheme);
        detail(&mut w);
        click(&mut w.harness, REMOVE);
        settle(&mut w.harness, "the question", |h| {
            h.html().contains("Remove account Home cloud?")
        });
        let ratio = label_contrast(
            &mut w.harness,
            ".ds-alert .ds-button[data-role=\"destructive\"]",
        );
        assert!(
            ratio >= 4.5,
            "{tag}: Remove account is {ratio:.2}:1 on its fill"
        );
    }
}

#[test]
fn every_button_says_which_account_or_app_it_is_for() {
    let mut w = world("accounts-names");
    for name in [
        "Manage Home cloud",
        "Manage Work mail",
        "Manage Claude key",
        "Manage Home sync",
    ] {
        let selector = format!(".account-row [aria-label=\"{name}\"]");
        assert_eq!(w.harness.count(&selector), 1, "{name}");
    }
    assert_eq!(
        w.harness.count("[aria-label=\"Manage\"]"),
        0,
        "no bare Manage"
    );
    detail(&mut w);
    for (selector, name) in [
        (SIGN_IN, "Sign in again Home cloud"),
        (REMOVE, "Remove account Home cloud"),
    ] {
        assert_eq!(
            w.harness.attr(selector, "aria-label").as_deref(),
            Some(name)
        );
    }
    click(&mut w.harness, BACK);
    open(&mut w, KEY);
    assert_eq!(
        w.harness.attr(REVOKE, "aria-label").as_deref(),
        Some("Revoke Photos allowed: llm")
    );
    assert_eq!(
        w.harness.count("[aria-label=\"Revoke\"]"),
        0,
        "no bare Revoke"
    );
}

#[test]
fn sign_in_again_says_a_browser_only_for_an_account_whose_row_says_browser() {
    let mut w = world("accounts-sign-in-words");
    detail(&mut w);
    let plain = w.harness.html();
    assert!(plain.contains("Sign in to the account again."), "{plain}");
    assert!(!plain.contains("in your browser"));
    click(&mut w.harness, BACK);
    settle(&mut w.harness, "the list", |h| {
        h.count(".accounts-list") == 1
    });
    open(&mut w, WORK);
    assert!(
        w.harness
            .html()
            .contains("Sign in to the account again in your browser.")
    );
}

fn grouped() -> Vec<Held> {
    vec![
        Held::new("claude", "Claude Code", "ok", &["agent"])
            .in_group("agent")
            .signing_in("agent"),
        Held::new("work", "Work mail", "ok", &["mail"]).in_group("internet"),
        Held::runtime("ollama", "Ollama", "ok").in_group("intelligence"),
    ]
}

/// The headings of the list, top to bottom.
fn headings(w: &World) -> Vec<String> {
    let page = w.harness.html();
    let html = &page[page.find("class=\"accounts-list\"").expect("the list")..];
    ["Internet Accounts", "Intelligence", "Assistants"]
        .into_iter()
        .map(|name| (html.find(&format!(">{name}<")), name))
        .filter_map(|(at, name)| at.map(|at| (at, name.to_owned())))
        .collect::<std::collections::BTreeMap<_, _>>()
        .into_values()
        .collect()
}

#[test]
fn the_list_is_grouped_by_the_groups_the_module_names_whatever_order_it_sends() {
    let w = world_of("accounts-groups", grouped(), Scheme::Light);
    assert_eq!(
        headings(&w),
        ["Internet Accounts", "Intelligence", "Assistants"]
    );
    let page = w.harness.html();
    let html = &page[page.find("class=\"accounts-list\"").expect("the list")..];
    let at = |text: &str| html.find(text).expect(text);
    assert!(
        at("Work mail") < at("Ollama"),
        "Internet before Intelligence"
    );
    assert!(
        at("Ollama") < at("Claude Code"),
        "Intelligence before Assistants"
    );
}

#[test]
fn an_account_without_a_group_row_is_listed_where_its_own_rows_put_it() {
    let rows = vec![
        Held::new("claude", "Claude Code", "ok", &["agent"]).signing_in("agent"),
        Held::new("work", "Work mail", "ok", &["mail"]),
        Held::runtime("ollama", "Ollama", "ok"),
    ];
    let w = world_of("accounts-fallback-groups", rows, Scheme::Light);
    assert_eq!(
        headings(&w),
        ["Internet Accounts", "Intelligence", "Assistants"]
    );
}

#[test]
fn the_minus_waits_for_a_selection_then_asks_before_removing_the_selected_account() {
    let mut w = world("accounts-minus");
    assert_eq!(
        w.harness.attr(MINUS, "data-availability").as_deref(),
        Some("disabled"),
        "nothing is selected"
    );
    click(&mut w.harness, "[aria-posinset=\"3\"] .account-row");
    settle(&mut w.harness, "the minus", |h| {
        h.attr(MINUS, "data-availability").as_deref() != Some("disabled")
    });
    click(&mut w.harness, MINUS);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Work mail?")
    });
    assert!(
        w.accountd.sets().is_empty(),
        "the press alone removes nothing"
    );
}

#[test]
fn a_permit_says_since_when_and_for_how_long_from_the_rows_keyed_by_its_grant() {
    let mut w = world("accounts-grant-facts");
    {
        let mut held = w.inferd.accounts.lock().expect("accounts");
        let key = held.first_mut().expect("the key");
        *key = key
            .clone()
            .granted_on("g1", "2026-10-08", "always")
            .granting("g2", "Mail allowed: llm")
            .granted_on("g2", "2026-10-09", "session");
    }
    w.announce(Module::Inferd, "accounts.claude-key.grant.g2", "added");
    open(&mut w, KEY);
    settle(&mut w.harness, "the second permit", |h| {
        h.html().contains("Since 2026-10-09. This session only")
    });
    assert!(w.harness.html().contains("Since 2026-10-08. Always"));
}
