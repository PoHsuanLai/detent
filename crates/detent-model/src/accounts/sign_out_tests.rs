//! An assistant's sign-out: the question first, then how it ended.

use super::tests::{action, apply, fixed, path, row, state_of};
use super::*;
use crate::picker::Described;
use ds_settings::schema::ActionWeight;
use std::collections::BTreeMap;

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
fn a_sign_out_end_is_told_and_settled_ends_read_again() {
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
    let tell = |end| {
        AccountsEffect::Tell(AccountsNotice::SignedOut {
            end,
            account: Some("Claude Code".to_owned()),
        })
    };
    let cases = [
        ("asked", SignOutEnd::Asked, false),
        ("login_untouched", SignOutEnd::LoginUntouched, true),
        ("done", SignOutEnd::Done, true),
        ("failed", SignOutEnd::Failed, true),
        ("cancelled", SignOutEnd::Cancelled, true),
        ("expired", SignOutEnd::Expired, true),
        ("launcher_gone", SignOutEnd::LauncherGone, true),
    ];
    for (word, end, reloads) in cases {
        let mut want = vec![tell(end)];
        if reloads {
            want.push(AccountsEffect::Reload(ModuleId::Accountd));
        }
        assert_eq!(told(word), want, "{word}");
    }
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
