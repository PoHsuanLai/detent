//! Rows read as accounts: what each row of an account says of it.

use super::rows::{Row, classify};
use super::tests::{described, fixed, toggle};
use super::*;
use crate::picker::Described;
use ds_settings::schema::Help;
use porter_core::SignInWay;

#[test]
fn rows_are_read_as_accounts_with_dotted_ids_kept_whole() {
    let pane = AccountsPane::of(ModuleId::Accountd, &described());
    let labels: Vec<&str> = pane.accounts.iter().map(|a| a.label.as_str()).collect();
    assert_eq!(labels, ["Home cloud", "Work mail"]);
    let work = &pane.accounts[1];
    assert_eq!(work.key, AccountKey("work.example".to_owned()));
    assert_eq!(work.status, AccountStatus::NeedsReauth);
    assert_eq!(work.services.len(), 1);
    assert!(work.reauth.is_none());
    assert!(work.remove.is_some());
}

#[test]
fn an_account_lists_its_services_permissions_and_actions() {
    let pane = AccountsPane::of(ModuleId::Accountd, &described());
    let home = &pane.accounts[0];
    let services: Vec<(&str, SwitchState)> = home
        .services
        .iter()
        .map(|s| (s.kind.as_str(), s.on))
        .collect();
    assert_eq!(
        services,
        [("storage", SwitchState::On), ("calendar", SwitchState::Off)]
    );
    assert_eq!(home.permissions.len(), 1);
    assert_eq!(home.permissions[0].label, "Photos allowed: storage");
    assert_eq!(home.permissions[0].action, "Revoke");
}

#[test]
fn the_sign_in_client_ids_are_text_rows_held_apart_from_the_accounts() {
    let pane = AccountsPane::of(ModuleId::Accountd, &described());
    assert_eq!(pane.accounts.len(), 2);
    assert_eq!(pane.clients.len(), 1);
    assert_eq!(pane.clients[0].value, ClientValue::Text("abc".to_owned()));
}

/// Two runtimes on this computer (one stopped, one sending a service row per model), a cloud
/// account that also serves a language model, and a local account that serves none.
fn with_runtimes() -> Described {
    let mut described = described();
    described.keys.extend([
        fixed("accounts.ollama.label", "Account", "Ollama"),
        fixed("accounts.ollama.state", "Status", "ok"),
        fixed("accounts.ollama.place", "Where", "this_computer"),
        toggle("accounts.ollama.service.llm", "llm"),
        toggle("accounts.ollama.service.llm", "llm"),
        fixed("accounts.lm-studio.label", "Account", "LM Studio"),
        fixed("accounts.lm-studio.state", "Status", "offline"),
        fixed("accounts.lm-studio.place", "Where", "this_computer"),
        toggle("accounts.lm-studio.service.llm", "llm"),
        fixed("accounts.anthropic.label", "Account", "Anthropic"),
        fixed("accounts.anthropic.place", "Where", "elsewhere"),
        toggle("accounts.anthropic.service.llm", "llm"),
        fixed("accounts.photos.label", "Account", "Photos"),
        fixed("accounts.photos.place", "Where", "this_computer"),
        toggle("accounts.photos.service.storage", "storage"),
    ]);
    described
}

#[test]
fn local_runtimes_are_the_llm_accounts_whose_place_is_this_computer_running_or_not() {
    let pane = AccountsPane::of(ModuleId::Accountd, &with_runtimes());
    let runtimes: Vec<(&str, &AccountStatus)> = pane
        .local_runtimes()
        .map(|a| (a.key.0.as_str(), &a.status))
        .collect();
    assert_eq!(
        runtimes,
        [
            ("ollama", &AccountStatus::Ok),
            ("lm-studio", &AccountStatus::Offline)
        ]
    );
    let none = AccountsPane::of(ModuleId::Accountd, &described());
    assert_eq!(none.local_runtimes().count(), 0, "no place row, no runtime");
}

#[test]
fn a_service_sent_once_per_model_is_one_switch() {
    let pane = AccountsPane::of(ModuleId::Accountd, &with_runtimes());
    let ollama = pane
        .account(&AccountKey("ollama".to_owned()))
        .expect("ollama");
    assert_eq!(ollama.services.len(), 1);
}

/// Which row a toggle at `key` is, as a word: the table's left side is the path's shape.
fn row_word(key: &str) -> Option<&'static str> {
    let spec = toggle(key, "x");
    match classify(ModuleId::Accountd, &spec, &described())? {
        (_, Row::AccountService(..)) => Some("service"),
        (_, Row::Sync(_)) => Some("sync"),
        (_, Row::ShowInFiles(_)) => Some("files"),
        _ => Some("other"),
    }
}

#[test]
fn classify_tells_a_service_from_a_sync_switch_and_drops_an_unknown_leaf() {
    let table = [
        ("accounts.home.service.storage", Some("service")),
        ("accounts.home.sync.files", Some("sync")),
        ("accounts.work.example.sync.photos", Some("sync")),
        ("accounts.work.example.storage.files", Some("files")),
        ("accounts.home.sync.other", None),
        ("accounts.home.mystery", None),
    ];
    for (key, want) in table {
        assert_eq!(row_word(key), want, "{key}");
    }
}

#[test]
fn an_account_carries_the_modules_label_for_each_sync_switch() {
    let pane = AccountsPane::of(ModuleId::Accountd, &described());
    let sync: Vec<(&str, SwitchState)> = pane.accounts[0]
        .sync
        .iter()
        .map(|s| (s.label.as_str(), s.on))
        .collect();
    assert_eq!(
        sync,
        [
            (
                "Keep this account's files on this computer",
                SwitchState::Off
            ),
            (
                "Upload new photos from a folder on this computer to Google Photos",
                SwitchState::On
            )
        ]
    );
    assert!(pane.accounts[1].sync.is_empty());
}

#[test]
fn the_expiry_row_is_read_with_the_modules_label_help_and_words() {
    let mut expires = fixed(
        "accounts.home.expires",
        "Sign-in",
        "Google signs this account out on 2026-10-14",
    );
    expires.help = Help("Google ends the sign-ins of an app that is not verified yet.".to_owned());
    let mut keys = described().keys;
    keys.push(expires);
    let pane = AccountsPane::of(
        ModuleId::Accountd,
        &Described {
            keys,
            ..described()
        },
    );
    let home = pane.account(&AccountKey("home".to_owned())).expect("home");
    assert_eq!(
        home.expires,
        Some(AccountExpiry {
            label: "Sign-in".to_owned(),
            help: "Google ends the sign-ins of an app that is not verified yet.".to_owned(),
            text: "Google signs this account out on 2026-10-14".to_owned(),
        })
    );
    assert_eq!(pane.accounts[1].expires, None);
}

fn pane_with_sign_in(word: Option<&str>) -> Account {
    let mut keys = described().keys;
    keys.extend(word.map(|w| fixed("accounts.home.sign_in", "How", w)));
    let described = Described {
        keys,
        ..described()
    };
    AccountsPane::of(ModuleId::Accountd, &described)
        .accounts
        .remove(0)
}

#[test]
fn the_sign_in_row_is_read_by_its_slug() {
    for way in [
        SignInWay::Browser,
        SignInWay::Password,
        SignInWay::Key,
        SignInWay::Agent,
        SignInWay::Outside,
        SignInWay::Nothing,
    ] {
        let account = pane_with_sign_in(Some(way.slug()));
        assert_eq!(account.sign_in, Some(way), "{}", way.slug());
    }
}

#[test]
fn a_missing_or_unknown_sign_in_row_is_no_way_to_sign_in() {
    for word in [None, Some("shrugged")] {
        assert_eq!(pane_with_sign_in(word).sign_in, None);
    }
}

#[test]
fn an_account_reads_its_spend_rows_and_names_the_datasets_the_sync_service_runs_for_it() {
    let mut rows = described();
    rows.keys
        .push(fixed("accounts.home.spend.month", "This month", "2500"));
    rows.keys
        .push(fixed("accounts.home.spend.cap", "Cap", "5000"));
    let pane = AccountsPane::of(ModuleId::Accountd, &rows);
    let home = &pane.accounts[0];
    assert_eq!(home.spend.bar().map(|bar| bar.permille), Some(500));
    assert_eq!(pane.accounts[1].spend.bar(), None);
    // Photos sync is on; the calendar service is off and storage is not a mirrored service.
    assert_eq!(pane.datasets(), [DatasetName("home/photos".to_owned())]);
}
