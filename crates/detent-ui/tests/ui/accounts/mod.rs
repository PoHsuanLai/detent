//! The Accounts page over three fake modules in memory (identities and AI keys, an AI key's spend,
//! sync): the
//! list, one account's switches and actions, the question before a removal, and the page when
//! a module or all of them are missing.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held, ModuleAt, SECRET};
use crate::support::bus::PrivateBus;
use crate::support::drive::click;
use crate::support::syncd::{Standing, Syncd};
use detent_callers::{RoleCheck, RoleName, introduce_service, private_peers, roles};
use detent_client::Vouch;
use detent_model::{Absence, ModuleId, PageSlug};
use detent_system::modules::FakeModules;
use detent_ui::Opening;
use ds::prelude::Scheme;
use ds_harness::{Backdrop, Harness, Input};
use ds_settings::live::{LiveClient, LiveError, Verdict};
use ds_settings::schema::KeyPath;
use porter_wire::names::{ACCOUNTS_BUS, ACCOUNTS_SETTINGS_PATH};
use std::time::Duration;

mod add;
mod keys;
mod list;
mod removal;
mod secrets;
mod sync;
mod trust;
mod words;

/// An account: its row in the list and the id its detail names.
#[derive(Clone, Copy)]
struct Acct {
    row: &'static str,
    id: &'static str,
}

const HOME: Acct = Acct {
    row: "#account-home",
    id: "home",
};
const KEY: Acct = Acct {
    row: "#account-claude-key",
    id: "claude-key",
};
const SYNC: Acct = Acct {
    row: "#account-sync-home",
    id: "sync/home",
};
const WORK: Acct = Acct {
    row: "#account-work-example",
    id: "work.example",
};
const LLM: &str = "[aria-label=\"Llm\"] .ds-toggle";
const KEEP_FILES: &str = "[aria-label=\"Keep this account's files on this computer\"] .ds-toggle";
const UPLOAD_PHOTOS: &str =
    "[aria-label=\"Upload new photos from a folder on this computer to Google Photos\"] .ds-toggle";
const REVOKE: &str = "[aria-label=\"Revoke Photos allowed: llm\"]";
const SIGN_IN: &str = "[aria-label=\"Sign in again\"] .ds-field-row-control .ds-button";
const REMOVE: &str = "[aria-label=\"Remove account\"] .ds-field-row-control .ds-button";
const CONTINUE_AS: &str = ".accounts-more [aria-label=\"Continue as\u{2026}\"]";

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

/// accountd's AI key, with one permit; inferd adds the month's spend of it.
fn keys() -> Vec<Held> {
    vec![
        Held::new("claude-key", "Claude key", "ok", &["llm"])
            .in_group("intelligence")
            .granting("g1", "Photos allowed: llm")
            .spending(2500, 5000),
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
    /// What the sync service's `Status` answers; set a dataset's standing to change it.
    sync1: Syncd,
    harness: Harness,
    rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    /// The modules, hosted in memory.
    modules: FakeModules,
    /// The sync service's own object on the bus, while it is there: it is no live module.
    sync_bus: Option<zbus::Connection>,
}

fn world(name: &str) -> World {
    world_in(name, Scheme::Light)
}

fn world_in(name: &str, scheme: Scheme) -> World {
    let listed = [identities(), keys()].concat();
    start(
        name,
        scheme,
        listed,
        &[ModuleId::Accountd, ModuleId::Inferd, ModuleId::Syncd],
    )
}

/// Only accountd, holding `rows`.
fn world_of(name: &str, rows: Vec<Held>, scheme: Scheme) -> World {
    start(name, scheme, rows, &[ModuleId::Accountd])
}

/// The page with the modules in `served` running; the others are not on the bus.
fn start(name: &str, scheme: Scheme, rows: Vec<Held>, served: &[ModuleId]) -> World {
    start_syncing(name, scheme, rows, served, syncing(), Syncd::default())
}

/// The same, with syncd's accounts `mirrored` and the sync service answering `sync1`.
fn start_syncing(
    name: &str,
    scheme: Scheme,
    rows: Vec<Held>,
    served: &[ModuleId],
    mirrored: Vec<Held>,
    sync1: Syncd,
) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let held = |module: ModuleId, count: usize| if served.contains(&module) { count } else { 0 };
    let expected = held(ModuleId::Accountd, rows.len()) + held(ModuleId::Syncd, mirrored.len());
    // inferd only sends the spend rows of the accounts accountd lists.
    let spent: Vec<Held> = rows
        .iter()
        .filter(|held| held.spend.is_some())
        .cloned()
        .collect();
    let accountd = Accountd::new(Verdict::Allow, rows);
    let inferd = Accountd::on(ModuleId::Inferd, Verdict::Allow, spent);
    let syncd = Accountd::on(ModuleId::Syncd, Verdict::Allow, mirrored).continuing_as();
    let modules = FakeModules::new();
    for module in served {
        let fake = match module {
            ModuleId::Accountd => &accountd,
            ModuleId::Inferd => &inferd,
            ModuleId::Syncd => &syncd,
        };
        fake.host_on(&modules);
    }
    let sync_bus = served
        .contains(&ModuleId::Syncd)
        .then(|| rt.block_on(sync1.serve_on(&bus)));
    let sandbox = Sandbox::new(name)
        .on_bus(&bus.address)
        .with_modules(modules.clone());
    let mut harness = sandbox.harness_in(accounts(), scheme);
    settle(&mut harness, "the list", |h| {
        h.count(".accounts-list") == 1 || h.count(".accounts-empty") == 1
    });
    // Each module answers on its own; the list is whole once every served one has.
    settle(&mut harness, "every module's rows", |h| {
        expected == 0 || h.count(".account-row-body") >= expected
    });
    World {
        accountd,
        inferd,
        syncd,
        sync1,
        harness,
        rt,
        _bus: bus,
        modules,
        sync_bus,
    }
}

impl World {
    /// Announces, as `module`, that the row at `path` changed to `word`.
    fn announce(&self, module: ModuleId, path: &str, word: &str) {
        self.modules.announce(
            &module.daemon(),
            &KeyPath(path.to_owned()),
            toml::Value::String(word.to_owned()),
        );
    }

    /// The module goes away, as when its service stops (and syncd's own object leaves the bus).
    fn stop(&mut self, module: ModuleId) {
        self.modules.away(&module.daemon(), Absence::NotRunning);
        if let (ModuleId::Syncd, Some(connection)) = (module, self.sync_bus.take()) {
            self.rt
                .block_on(connection.release_name(module.bus()))
                .expect("the name is released");
        }
    }
}

/// Rests the pointer on the permit's row, so the strip shows under it, and presses Revoke.
fn press_revoke(w: &mut World) {
    let at = w.harness.centre(REVOKE).expect("the strip is laid out");
    w.harness.send(Input::pointer_move(at));
    w.harness.advance(Duration::from_millis(200));
    click(&mut w.harness, REVOKE);
}

/// Chooses `account` in the list and waits for its detail beside it.
fn open(w: &mut World, account: Acct) {
    click(&mut w.harness, account.row);
    settle(&mut w.harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some(account.id)
    });
}

fn detail(w: &mut World) {
    open(w, HOME);
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

const ADD: &str = ".accounts-add [aria-label=\"Add Account\u{2026}\"]";
const MINUS: &str = ".accounts-add [aria-label=\"Remove Account\"]";

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
