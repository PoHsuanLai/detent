use super::model::{Row, classify};
use super::*;
use crate::Absence;
use crate::picker::Described;
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};
use porter_core::SignInWay;
use std::collections::BTreeMap;

fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

/// The row at `text` of accountd's module.
fn row(text: &str) -> ModuleRow {
    ModuleRow::new(ModuleId::Accountd, path(text))
}

fn spec(key: &str, label: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        path: path(key),
        kind,
        default: toml::Value::Boolean(false),
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Accounts,
        section: Section("Nextcloud".to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

fn fixed(key: &str, label: &str, variant: &str) -> KeySpec {
    spec(
        key,
        label,
        KeyKind::Fixed {
            variant: variant.to_owned(),
        },
    )
}

fn toggle(key: &str, kind: &str) -> KeySpec {
    spec(
        key,
        kind,
        KeyKind::Toggle {
            variants: ["on".to_owned(), "off".to_owned()],
        },
    )
}

/// A grant row: labelled with the sentence, its action named `Revoke`.
fn grant() -> KeySpec {
    KeySpec {
        kind: KeyKind::Live {
            action: LiveAction {
                label: ActionLabel("Revoke".to_owned()),
                weight: ActionWeight::Plain,
            },
        },
        ..spec(
            "accounts.home.grant.g1",
            "Photos allowed: storage",
            KeyKind::Text,
        )
    }
}

fn action(key: &str, label: &str, weight: ActionWeight) -> KeySpec {
    spec(
        key,
        label,
        KeyKind::Live {
            action: LiveAction {
                label: ActionLabel(label.to_owned()),
                weight,
            },
        },
    )
}

/// Two accounts, one with a dotted id, as accountd would describe them.
pub(super) fn described() -> Described {
    let keys = vec![
        fixed("accounts.home.label", "Account", "Home cloud"),
        fixed("accounts.home.state", "Status", "ok"),
        toggle("accounts.home.service.storage", "storage"),
        toggle("accounts.home.service.calendar", "calendar"),
        toggle(
            "accounts.home.sync.files",
            "Keep this account's files on this computer",
        ),
        toggle(
            "accounts.home.sync.photos",
            "Upload new photos from a folder on this computer to Google Photos",
        ),
        toggle("accounts.home.sync.other", "other"),
        grant(),
        action("accounts.home.reauth", "Sign in again", ActionWeight::Plain),
        action(
            "accounts.home.remove",
            "Remove account",
            ActionWeight::Destructive,
        ),
        fixed("accounts.work.example.label", "Account", "Work mail"),
        fixed("accounts.work.example.state", "Status", "needs_reauth"),
        toggle("accounts.work.example.service.mail", "mail"),
        action(
            "accounts.work.example.remove",
            "Remove account",
            ActionWeight::Destructive,
        ),
        spec(
            "accounts.clients.microsoft",
            "microsoft client id",
            KeyKind::Text,
        ),
    ];
    let values: BTreeMap<KeyPath, String> = [
        ("accounts.home.service.storage", "on"),
        ("accounts.home.service.calendar", "off"),
        ("accounts.home.sync.photos", "on"),
        ("accounts.work.example.service.mail", "on"),
        ("accounts.clients.microsoft", "abc"),
    ]
    .into_iter()
    .map(|(key, word)| (path(key), word.to_owned()))
    .collect();
    Described {
        keys,
        values,
        ..Default::default()
    }
}

pub(super) fn ready() -> AccountsState {
    accounts_step(
        AccountsState::default(),
        AccountsInput::Found(ModuleId::Accountd, Ok(described())),
    )
    .0
}

pub(super) fn state_of(state: AccountsState) -> AccountsReady {
    match state {
        AccountsState::Ready(ready) => *ready,
        other => panic!("not ready: {other:?}"),
    }
}

pub(super) fn apply(
    state: AccountsState,
    inputs: impl IntoIterator<Item = AccountsInput>,
) -> (AccountsState, Vec<AccountsEffect>) {
    inputs
        .into_iter()
        .fold((state, Vec::new()), |(state, mut effects), input| {
            let (next, more) = accounts_step(state, input);
            effects.extend(more);
            (next, effects)
        })
}

#[test]
fn rows_are_read_as_accounts_with_dotted_ids_kept_whole() {
    let pane = AccountsPane::of(ModuleId::Accountd, &described());
    let labels: Vec<&str> = pane.accounts.iter().map(|a| a.label.as_str()).collect();
    assert_eq!(labels, ["Home cloud", "Work mail"]);
    let work = &pane.accounts[1];
    assert_eq!(work.key, AccountKey("work.example".to_owned()));
    assert_eq!(work.status, AccountStatus::NeedsReauth);
    assert_eq!(work.status.label(), "Needs sign-in");
    assert_eq!(work.services.len(), 1);
    assert!(work.reauth.is_none());
    assert!(work.remove.is_some());
}

#[test]
fn an_account_lists_its_services_permissions_and_actions() {
    let pane = AccountsPane::of(ModuleId::Accountd, &described());
    let home = &pane.accounts[0];
    let services: Vec<(String, bool)> = home.services.iter().map(|s| (s.label(), s.on)).collect();
    assert_eq!(
        services,
        [("Storage".to_owned(), true), ("Calendar".to_owned(), false)]
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

#[test]
fn the_pane_opens_on_the_list_and_an_account_opens_and_closes() {
    let (state, effects) = apply(
        ready(),
        [AccountsInput::Open(AccountKey("home".to_owned()))],
    );
    assert!(effects.is_empty());
    let up = state_of(state.clone());
    assert_eq!(up.open().map(|a| a.label.as_str()), Some("Home cloud"));
    let back = state_of(apply(state, [AccountsInput::Back]).0);
    assert_eq!(back.screen, AccountsScreen::List);
}

#[test]
fn an_account_nobody_listed_does_not_open() {
    let (state, _) = apply(
        ready(),
        [AccountsInput::Open(AccountKey("nobody".to_owned()))],
    );
    assert_eq!(state_of(state).screen, AccountsScreen::List);
}

#[test]
fn a_switch_sets_the_word_at_once_and_waits_for_the_answer() {
    let storage = row("accounts.home.service.storage");
    let (state, effects) = apply(
        ready(),
        [AccountsInput::Switched {
            row: storage.clone(),
            on: false,
        }],
    );
    assert_eq!(
        effects,
        [AccountsEffect::Set {
            row: storage.clone(),
            value: toml::Value::String("off".to_owned()),
        }]
    );
    let up = state_of(state.clone());
    assert!(!up.pane.accounts[0].services[0].on);
    assert_eq!(up.pending, Some(storage.clone()));
    // a second change waits for the first
    let (_, again) = apply(
        state.clone(),
        [AccountsInput::Switched {
            row: row("accounts.home.service.calendar"),
            on: true,
        }],
    );
    assert!(again.is_empty());
    // the answer frees the pane and the rows are read again
    let (state, effects) = apply(state, [AccountsInput::Done(storage)]);
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Accountd)]);
    assert_eq!(state_of(state).pending, None);
}

#[test]
fn a_refused_set_says_why_and_reads_the_module_again() {
    let storage = row("accounts.home.service.storage");
    let (state, effects) = apply(
        ready(),
        [
            AccountsInput::Switched {
                row: storage.clone(),
                on: false,
            },
            AccountsInput::Failed {
                row: storage,
                why: "refused".to_owned(),
            },
        ],
    );
    assert_eq!(
        effects.last(),
        Some(&AccountsEffect::Reload(ModuleId::Accountd)),
        "{effects:?}"
    );
    assert!(effects.contains(&AccountsEffect::Tell("refused".to_owned())));
    assert_eq!(state_of(state).pending, None);
}

#[test]
fn revoke_and_sign_in_again_run_on_one_press() {
    for key in ["accounts.home.grant.g1", "accounts.home.reauth"] {
        let (state, effects) = apply(ready(), [AccountsInput::Ran(row(key))]);
        assert_eq!(effects, [AccountsEffect::Invoke(row(key))], "{key}");
        assert_eq!(state_of(state).pending, Some(row(key)));
    }
}

#[test]
fn removing_an_account_is_never_run_without_the_question_answered() {
    let remove = row("accounts.home.remove");
    let (state, effects) = apply(ready(), [AccountsInput::Ran(remove.clone())]);
    assert!(effects.is_empty(), "a press of Remove alone does nothing");
    assert_eq!(state_of(state).pending, None);

    let (asked, effects) = apply(ready(), [AccountsInput::AskRemove(remove.clone())]);
    assert!(effects.is_empty());
    assert_eq!(
        state_of(asked.clone()).confirming,
        Some(RemovalQuestion {
            ask: Ask::Remove,
            row: remove.clone(),
            account: "Home cloud".to_owned(),
            files: LocalFiles::Kept,
        })
    );

    let (kept, effects) = apply(asked.clone(), [AccountsInput::Keep]);
    assert!(effects.is_empty());
    assert_eq!(state_of(kept).confirming, None);

    let (removing, effects) = apply(asked, [AccountsInput::Remove]);
    assert_eq!(effects, [AccountsEffect::Invoke(remove.clone())]);
    assert_eq!(state_of(removing).pending, Some(remove));
}

#[test]
fn an_answer_with_no_question_removes_nothing() {
    let (_, effects) = apply(ready(), [AccountsInput::Remove]);
    assert!(effects.is_empty());
}

#[test]
fn only_a_removal_row_is_asked_about() {
    let (state, _) = apply(
        ready(),
        [AccountsInput::AskRemove(row("accounts.home.reauth"))],
    );
    assert_eq!(state_of(state).confirming, None);
}

#[test]
fn a_removed_account_takes_the_pane_back_to_the_list() {
    let remove = row("accounts.home.remove");
    let (state, _) = apply(
        ready(),
        [
            AccountsInput::Open(AccountKey("home".to_owned())),
            AccountsInput::AskRemove(remove.clone()),
            AccountsInput::Remove,
            AccountsInput::Done(remove),
        ],
    );
    let mut rest = described();
    rest.keys
        .retain(|spec| !spec.path.0.starts_with("accounts.home."));
    let (state, _) = apply(state, [AccountsInput::Found(ModuleId::Accountd, Ok(rest))]);
    let up = state_of(state);
    assert_eq!(up.screen, AccountsScreen::List);
    assert_eq!(up.pane.accounts.len(), 1);
}

#[test]
fn a_reading_keeps_the_screen_when_its_account_is_still_there() {
    let (state, _) = apply(
        ready(),
        [
            AccountsInput::Open(AccountKey("work.example".to_owned())),
            AccountsInput::Found(ModuleId::Accountd, Ok(described())),
        ],
    );
    assert_eq!(
        state_of(state).screen,
        AccountsScreen::Detail(AccountKey("work.example".to_owned()))
    );
}

#[test]
fn a_client_id_is_set_trimmed_and_only_when_it_changed() {
    let microsoft = row("accounts.clients.microsoft");
    let (_, same) = apply(
        ready(),
        [AccountsInput::ClientEdited {
            row: microsoft.clone(),
            text: ClientText::Plain(" abc ".to_owned()),
        }],
    );
    assert!(same.is_empty());
    let (_, changed) = apply(
        ready(),
        [AccountsInput::ClientEdited {
            row: microsoft.clone(),
            text: ClientText::Plain(" xyz ".to_owned()),
        }],
    );
    assert_eq!(
        changed,
        [AccountsEffect::Set {
            row: microsoft,
            value: toml::Value::String("xyz".to_owned())
        }]
    );
}

#[test]
fn a_change_the_module_announces_reads_it_again() {
    let (_, effects) = apply(ready(), [AccountsInput::Changed(ModuleId::Accountd)]);
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Accountd)]);
}

#[test]
fn the_pane_is_unavailable_only_when_every_module_is_not_there_and_ignores_the_person() {
    let gone = |module| AccountsInput::Found(module, Err(Absence::NotRunning));
    let (state, effects) = apply(
        AccountsState::default(),
        [
            gone(ModuleId::Accountd),
            gone(ModuleId::Inferd),
            gone(ModuleId::Syncd),
            AccountsInput::Open(AccountKey("home".to_owned())),
        ],
    );
    assert!(matches!(
        state,
        AccountsState::Unavailable(Absence::NotRunning, _)
    ));
    assert!(effects.is_empty());
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
        _ => Some("other"),
    }
}

#[test]
fn classify_tells_a_service_from_a_sync_switch_and_drops_an_unknown_leaf() {
    let table = [
        ("accounts.home.service.storage", Some("service")),
        ("accounts.home.sync.files", Some("sync")),
        ("accounts.work.example.sync.photos", Some("sync")),
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
    let sync: Vec<(&str, bool)> = pane.accounts[0]
        .sync
        .iter()
        .map(|s| (s.label.as_str(), s.on))
        .collect();
    assert_eq!(
        sync,
        [
            ("Keep this account's files on this computer", false),
            (
                "Upload new photos from a folder on this computer to Google Photos",
                true
            )
        ]
    );
    assert!(pane.accounts[1].sync.is_empty());
}

/// A coding-agent account as accountd describes it: signed in inside the agent, so its only
/// action is Sign out.
fn agent_described() -> Described {
    Described {
        keys: vec![
            fixed("accounts.agent.label", "Account", "Claude Code"),
            fixed("accounts.agent.state", "Status", "ok"),
            action("accounts.agent.sign_out", "Sign out", ActionWeight::Plain),
            action(
                "accounts.agent.remove",
                "Remove account",
                ActionWeight::Destructive,
            ),
        ],
        values: BTreeMap::new(),
        ..Default::default()
    }
}

#[test]
fn an_agent_account_shows_its_sign_out_row_and_signs_out_only_when_confirmed() {
    let pane = AccountsPane::of(ModuleId::Accountd, &agent_described());
    let agent = &pane.accounts[0];
    assert_eq!(agent.status, AccountStatus::Ok);
    assert_eq!(agent.status.label(), "Signed in");
    assert_eq!(
        agent.sign_out,
        Some((path("accounts.agent.sign_out"), "Sign out".to_owned()))
    );
    assert!(agent.reauth.is_none());

    let (state, _) = accounts_step(
        AccountsState::default(),
        AccountsInput::Found(ModuleId::Accountd, Ok(agent_described())),
    );
    let sign_out = row("accounts.agent.sign_out");
    // A bare press of the row runs nothing.
    let (state, effects) = apply(state, [AccountsInput::Ran(sign_out.clone())]);
    assert!(effects.is_empty());
    // Asking shows the question and runs nothing.
    let (asked, effects) = apply(state, [AccountsInput::AskSignOut(sign_out.clone())]);
    assert!(effects.is_empty());
    assert_eq!(
        state_of(asked.clone()).confirming,
        Some(RemovalQuestion {
            ask: Ask::SignOut,
            row: sign_out.clone(),
            account: "Claude Code".to_owned(),
            files: LocalFiles::None,
        })
    );
    // Cancel: still nothing.
    let (kept, effects) = apply(asked.clone(), [AccountsInput::Keep]);
    assert!(effects.is_empty());
    assert_eq!(state_of(kept.clone()).confirming, None);
    // Ask again, then confirm: it runs once.
    let (asked, _) = apply(kept, [AccountsInput::AskSignOut(sign_out.clone())]);
    let (done, effects) = apply(asked, [AccountsInput::Remove]);
    assert_eq!(effects, [AccountsEffect::Invoke(sign_out.clone())]);
    let done = state_of(done);
    assert_eq!(done.confirming, None);
    assert_eq!(done.pending, Some(sign_out));
}

#[test]
fn a_sign_out_end_is_told_in_words_and_settled_ends_read_again() {
    let (state, _) = accounts_step(
        AccountsState::default(),
        AccountsInput::Found(ModuleId::Accountd, Ok(agent_described())),
    );
    let sign_out = row("accounts.agent.sign_out");
    let told = |word: &str| {
        let heard = AccountsInput::heard(
            ModuleId::Accountd,
            sign_out.path.clone(),
            &toml::Value::String(word.to_owned()),
        );
        accounts_step(state.clone(), heard).1
    };
    let tell = |text: &str| AccountsEffect::Tell(text.to_owned());
    let cases = [
        ("asked", "Asking Claude Code to sign out\u{2026}", false),
        (
            "login_untouched",
            "Claude Code is still signed in on its own; open it to sign out there.",
            true,
        ),
        ("done", "Signed out of Claude Code.", true),
        ("failed", "Couldn't sign out of Claude Code.", true),
        (
            "cancelled",
            "Signing out of Claude Code was cancelled.",
            true,
        ),
        ("expired", "Claude Code didn't answer in time.", true),
        (
            "launcher_gone",
            "The app that runs Claude Code quit before signing out.",
            true,
        ),
    ];
    for (word, line, reloads) in cases {
        let mut want = vec![tell(line)];
        if reloads {
            want.push(AccountsEffect::Reload(ModuleId::Accountd));
        }
        assert_eq!(told(word), want, "{word}");
    }
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

#[test]
fn a_session_grant_announced_or_gone_reads_the_module_again() {
    let row = path("accounts.home.grant.s1");
    for word in ["added", "removed"] {
        let said = toml::Value::String(word.to_owned());
        assert_eq!(
            AccountsInput::heard(ModuleId::Accountd, row.clone(), &said),
            AccountsInput::Changed(ModuleId::Accountd)
        );
    }
}

#[test]
fn a_change_that_is_no_sign_out_word_just_reads_again() {
    let on = toml::Value::String("on".to_owned());
    let other = path("accounts.agent.files");
    assert_eq!(
        AccountsInput::heard(ModuleId::Accountd, other, &on),
        AccountsInput::Changed(ModuleId::Accountd)
    );
    let unknown = toml::Value::String("shrugged".to_owned());
    let row = path("accounts.agent.sign_out");
    assert_eq!(
        AccountsInput::heard(ModuleId::Accountd, row, &unknown),
        AccountsInput::Changed(ModuleId::Accountd)
    );
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
fn the_sign_in_row_is_read_and_each_slug_words_the_reauth_line() {
    let lines = [
        (
            "browser",
            SignInWay::Browser,
            "Sign in to the account again in your browser.",
        ),
        ("password", SignInWay::Password, "Enter the password again."),
        ("key", SignInWay::Key, "Enter the key again."),
        (
            "agent",
            SignInWay::Agent,
            "Asks the assistant to sign itself in, which needs its app to be running.",
        ),
        (
            "outside",
            SignInWay::Outside,
            "Sign in to the account again.",
        ),
        (
            "nothing",
            SignInWay::Nothing,
            "Sign in to the account again.",
        ),
    ];
    for (slug, way, line) in lines {
        assert_eq!(way.slug(), slug);
        let account = pane_with_sign_in(Some(slug));
        assert_eq!(account.sign_in, Some(way), "{slug}");
        assert_eq!(reauth_help(account.sign_in), line, "{slug}");
    }
}

#[test]
fn a_missing_or_unknown_sign_in_row_gets_the_plain_line_and_the_expiry_row_decides_nothing() {
    for word in [None, Some("shrugged")] {
        let account = pane_with_sign_in(word);
        assert_eq!(account.sign_in, None);
        assert_eq!(
            reauth_help(account.sign_in),
            "Sign in to the account again."
        );
    }
}
