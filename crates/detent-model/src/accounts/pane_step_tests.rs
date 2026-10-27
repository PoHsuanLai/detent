//! What a person's choices do to the pane: switches, actions, the removal question and sign-in
//! keys.

use super::tests::{apply, described, ready, row, state_of};
use super::*;

#[test]
fn a_switch_sets_the_word_at_once_and_waits_for_the_answer() {
    let storage = row("accounts.home.service.storage");
    let (state, effects) = apply(
        ready(),
        [AccountsInput::Switched {
            row: storage.clone(),
            on: SwitchState::Off,
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
    assert_eq!(up.pane.accounts[0].services[0].on, SwitchState::Off);
    assert_eq!(up.pending, Some(storage.clone()));
    // a second change waits for the first
    let (_, again) = apply(
        state.clone(),
        [AccountsInput::Switched {
            row: row("accounts.home.service.calendar"),
            on: SwitchState::On,
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
                on: SwitchState::Off,
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
    assert!(
        effects.contains(&AccountsEffect::Tell(AccountsNotice::Refused(
            "refused".to_owned()
        )))
    );
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
