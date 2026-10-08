//! The Accounts page over a fake accountd on a private bus: the list, one account's switches and
//! actions, the question before a removal, and the page without accountd.

mod support;

use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::Scheme;
use ds_harness::{Backdrop, Driver, Harness, Query};
use std::path::PathBuf;
use std::time::Duration;
use support::Sandbox;
use support::accountd::{Accountd, Held, Policy};
use support::bus::PrivateBus;
use support::drive::click;

const HOME: &str = "[aria-label=\"Home cloud\"] .ds-button";
const STORAGE: &str = "[aria-label=\"Storage\"] .ds-toggle";
const KEEP_FILES: &str = "[aria-label=\"Keep this account's files on this computer\"] .ds-toggle";
const UPLOAD_PHOTOS: &str =
    "[aria-label=\"Upload new photos from a folder on this computer to Google Photos\"] .ds-toggle";
const REVOKE: &str = "[aria-label=\"Photos allowed: storage\"] .ds-button";
const SIGN_IN: &str = "[aria-label=\"Sign in again\"] .ds-button";
const REMOVE: &str = "[aria-label=\"Remove account\"] .ds-button";
const BACK: &str = ".accounts-back .ds-button";

fn accounts() -> Opening {
    Opening {
        page: Some(PageSlug("accounts".to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    for _ in 0..200 {
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

fn held() -> Vec<Held> {
    vec![
        Held::new("home", "Home cloud", "ok", &["storage", "calendar"])
            .syncing()
            .granting("g1", "Photos allowed: storage"),
        Held::new("work.example", "Work mail", "needs_reauth", &["mail"])
            .expiring("Signed out by Google on 2026-10-14")
            .signing_in("browser"),
    ]
}

/// The page up over a fake accountd holding `held()`, showing the list.
struct World {
    fake: Accountd,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _daemon: zbus::Connection,
    served: ds_settings::live::Served,
}

fn world(name: &str) -> World {
    world_in(name, Scheme::Light)
}

fn world_in(name: &str, scheme: Scheme) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let fake = Accountd::new(Policy::Allow, held());
    let (daemon, served) = rt.block_on(fake.serve_announcing(&bus));
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    let mut harness = sandbox.harness_in(accounts(), scheme);
    settle(&mut harness, "the list", |h| h.count(".accounts-list") == 1);
    World {
        fake,
        harness,
        _rt: rt,
        _bus: bus,
        _daemon: daemon,
        served,
    }
}

fn detail(w: &mut World) {
    click(&mut w.harness, HOME);
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
}

#[test]
fn the_list_has_a_row_per_account_with_its_state() {
    let w = world("accounts-list");
    let html = w.harness.html();
    for text in ["Home cloud", "Work mail", "Signed in", "Needs sign-in"] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(w.harness.count(".account-row"), 2);
    assert!(
        html.contains("Sign-in clients"),
        "the client ids have a disclosure"
    );
}

#[test]
fn an_account_opens_to_its_services_and_apps_and_goes_back() {
    let mut w = world("accounts-detail");
    detail(&mut w);
    let html = w.harness.html();
    for text in [
        "Storage",
        "Calendar",
        "Photos allowed: storage",
        "Sign in again",
        "Remove account",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(
        w.harness.attr(STORAGE, "aria-checked").as_deref(),
        Some("true")
    );
    click(&mut w.harness, BACK);
    assert_eq!(w.harness.count(".accounts-list"), 1);
    assert_eq!(w.harness.count(".accounts-detail"), 0);
}

#[test]
fn a_switch_sets_the_service_and_the_answer_stays_on_screen() {
    let mut w = world("accounts-switch");
    detail(&mut w);
    click(&mut w.harness, STORAGE);
    settle(&mut w.harness, "the set", |_| !w.fake.sets().is_empty());
    assert_eq!(
        w.fake.sets(),
        [(
            "accounts.home.service.storage".to_owned(),
            toml::Value::String("off".to_owned())
        )]
    );
    settle(&mut w.harness, "the switch to read off", |h| {
        h.attr(STORAGE, "aria-checked").as_deref() == Some("false")
    });
}

#[test]
fn the_sync_switches_sit_under_the_services_and_set_on_and_off() {
    let mut w = world("accounts-sync");
    detail(&mut w);
    let html = w.harness.html();
    let services = html.find("Services").expect("services");
    let sync = html
        .find("Keep this account's files on this computer")
        .expect("sync");
    assert!(services < sync, "the sync switches come after the services");
    assert!(html.contains("Upload new photos from a folder on this computer to Google Photos"));
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
        w.fake.sets().len() == 2
    });
    let sets: Vec<(String, toml::Value)> = w.fake.sets();
    let want = |word: &str| {
        (
            "accounts.home.sync.files".to_owned(),
            toml::Value::String(word.to_owned()),
        )
    };
    assert_eq!(sets, [want("on"), want("off")]);
}

#[test]
fn the_expiry_is_a_read_only_line_in_the_modules_words() {
    let mut w = world("accounts-expires");
    click(&mut w.harness, "[aria-label=\"Work mail\"] .ds-button");
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
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
fn a_session_grant_row_comes_and_goes_with_the_announcement() {
    let mut w = world("accounts-session-grant");
    detail(&mut w);
    let sentence = "Claude Code can use Llm (this session)";
    let (grant, path) = ("s1", "accounts.home.grant.s1");
    w.fake
        .accounts
        .lock()
        .expect("accounts")
        .iter_mut()
        .filter(|held| held.id == "home")
        .for_each(|held| held.grants.push((grant.to_owned(), sentence.to_owned())));
    let announce = |w: &World, word: &str| {
        w._rt
            .block_on(w.served.changed(
                &ds_settings::schema::KeyPath(path.to_owned()),
                &toml::Value::String(word.to_owned()),
            ))
            .expect("announced");
    };
    announce(&w, "added");
    settle(&mut w.harness, "the session row", |h| {
        h.html().contains(sentence)
    });
    w.fake
        .accounts
        .lock()
        .expect("accounts")
        .iter_mut()
        .for_each(|held| held.grants.retain(|(g, _)| g != grant));
    announce(&w, "removed");
    settle(&mut w.harness, "the session row to go", |h| {
        !h.html().contains(sentence)
    });
}

#[test]
fn an_account_without_the_toggles_shows_no_sync_rows() {
    let mut w = world("accounts-no-sync");
    click(&mut w.harness, "[aria-label=\"Work mail\"] .ds-button");
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
    let html = w.harness.html();
    assert!(!html.contains("Upload new photos from a folder on this computer to Google Photos"));
    assert!(!html.contains("on this computer"));
}

#[test]
fn revoke_and_sign_in_again_run_the_action_at_once() {
    let mut w = world("accounts-run");
    detail(&mut w);
    click(&mut w.harness, SIGN_IN);
    settle(&mut w.harness, "the action", |_| !w.fake.sets().is_empty());
    assert_eq!(
        w.fake.sets(),
        [(
            "accounts.home.reauth".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
    settle(&mut w.harness, "the pane to be free", |h| {
        h.attr(REVOKE, "data-availability").as_deref() == Some("enabled")
    });
    click(&mut w.harness, REVOKE);
    settle(&mut w.harness, "the revoke", |_| w.fake.sets().len() == 2);
    settle(&mut w.harness, "the grant to leave the page", |h| {
        !h.html().contains("Photos allowed: storage")
    });
}

#[test]
fn removing_an_account_asks_first_and_keeping_it_does_nothing() {
    let mut w = world("accounts-keep");
    detail(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Home cloud?")
    });
    assert!(w.fake.sets().is_empty(), "the press alone removes nothing");
    click(
        &mut w.harness,
        ".ds-alert [data-role=\"normal\"], .ds-alert .ds-button",
    );
    settle(&mut w.harness, "the question to go", |h| {
        !h.html().contains("Remove account Home cloud?")
    });
    assert!(w.fake.sets().is_empty());
    assert_eq!(w.fake.accounts.lock().expect("accounts").len(), 2);
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
        w.fake.accounts.lock().expect("accounts").len() == 1
    });
    assert_eq!(
        w.fake.sets(),
        [(
            "accounts.home.remove".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
    settle(&mut w.harness, "the list", |h| {
        h.count(".accounts-list") == 1 && !h.html().contains("Home cloud")
    });
    assert!(w.harness.html().contains("Work mail"));
}

#[test]
fn an_account_with_a_dotted_id_opens_and_switches_by_its_whole_id() {
    let mut w = world("accounts-dotted");
    click(&mut w.harness, "[aria-label=\"Work mail\"] .ds-button");
    settle(&mut w.harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some("work.example")
    });
    click(&mut w.harness, "[aria-label=\"Mail\"] .ds-toggle");
    settle(&mut w.harness, "the set", |_| !w.fake.sets().is_empty());
    assert_eq!(w.fake.sets()[0].0, "accounts.work.example.service.mail");
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
fn a_bus_with_no_accountd_says_accountd_is_not_running() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-none");
    let mut harness = Sandbox::new("accounts-none")
        .on_bus(&bus.address)
        .harness(accounts());
    settle(&mut harness, "the pane giving up", |h| {
        h.html().contains("accountd is not running")
    });
    drop(rt);
}

#[test]
fn accountd_refusing_settings_is_said_in_its_own_words() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-refused");
    let fake = Accountd::new(Policy::Refuse, held());
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = Sandbox::new("accounts-refused")
        .on_bus(&bus.address)
        .harness(accounts());
    settle(&mut harness, "the refusal", |h| {
        h.html().contains("does not list Settings")
    });
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
    settle(&mut w.harness, "the set", |_| !w.fake.sets().is_empty());
    assert_eq!(
        w.fake.sets(),
        [(
            "accounts.clients.microsoft".to_owned(),
            toml::Value::String("abc".to_owned())
        )]
    );
}

fn shot(harness: &mut Harness, name: &str) -> usize {
    let dir = std::env::var_os("DETENT_SHOTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(support::TMP).join("shots"));
    std::fs::create_dir_all(&dir).expect("shots dir");
    let image = harness.render_over(Backdrop::Scheme).expect("a picture");
    image.save(dir.join(name)).expect("save png");
    image
        .pixels()
        .map(|p| p.0)
        .collect::<std::collections::HashSet<_>>()
        .len()
}

#[test]
fn the_list_the_account_and_the_question_are_drawn_in_light_and_dark() {
    for (scheme, tag) in [(Scheme::Light, "light"), (Scheme::Dark, "dark")] {
        let mut w = world_in(&format!("accounts-shots-{tag}"), scheme);
        assert!(shot(&mut w.harness, &format!("accounts-list-{tag}.png")) > 50);
        detail(&mut w);
        assert!(shot(&mut w.harness, &format!("accounts-detail-{tag}.png")) > 50);
        click(&mut w.harness, REMOVE);
        settle(&mut w.harness, "the question", |h| {
            h.html().contains("Remove account Home cloud?")
        });
        assert!(shot(&mut w.harness, &format!("accounts-remove-{tag}.png")) > 50);
    }
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
    for (name, row) in [
        ("Manage Home cloud", "Home cloud"),
        ("Manage Work mail", "Work mail"),
    ] {
        let selector = format!("[aria-label=\"{row}\"] .ds-button");
        assert_eq!(
            w.harness.attr(&selector, "aria-label").as_deref(),
            Some(name)
        );
    }
    assert_eq!(
        w.harness.count("[aria-label=\"Manage\"]"),
        0,
        "no bare Manage"
    );
    detail(&mut w);
    for (selector, name) in [
        (REVOKE, "Revoke Photos allowed: storage"),
        (SIGN_IN, "Sign in again Home cloud"),
        (REMOVE, "Remove account Home cloud"),
    ] {
        assert_eq!(
            w.harness.attr(selector, "aria-label").as_deref(),
            Some(name)
        );
    }
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
    click(&mut w.harness, "[aria-label=\"Work mail\"] .ds-button");
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
    assert!(
        w.harness
            .html()
            .contains("Sign in to the account again in your browser.")
    );
}
