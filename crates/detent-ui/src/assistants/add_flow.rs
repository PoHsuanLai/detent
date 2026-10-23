//! The Add sheet's work: fetching the list of coding assistants, downloading the one picked, and
//! writing its entry. The program that reaches the network runs on a blocking thread, so the
//! window keeps drawing; the sheet is changed as each part finishes.

use super::outside_view::{Shown, shown};
use crate::env::Env;
use detent_files::{AgentsTool, add_agent};
use detent_model::{AddButton, AddForm, Request};
use dioxus::prelude::*;
use docket_settings::NewAgent;

type Adding = Signal<Option<AddForm>>;

fn change(adding: &mut Adding, edit: impl FnOnce(AddForm) -> AddForm) {
    let next = adding.peek().clone().map(edit);
    adding.set(next);
}

/// Opens the sheet over the assistants installed and not listed yet, and asks for the list on
/// offer when there is a program to ask with.
pub(super) fn open(env: &Env, mut adding: Adding, now: &Shown) {
    let rows = &now.agents.rows;
    let Some(program) = env.agents_program.clone() else {
        adding.set(Some(AddForm::offline(&now.new, rows)));
        return;
    };
    adding.set(Some(AddForm::loading(&now.new, rows)));
    let tool = AgentsTool::new(program, &env.data_home);
    spawn(async move {
        let listed = tokio::task::spawn_blocking(move || tool.list()).await;
        change(&mut adding, |form| match listed {
            Ok(Ok(offers)) => form.catalogue(offers),
            Ok(Err(said)) => form.unreachable(&said),
            Err(_) => form.unreachable(""),
        });
    });
}

/// Closes the sheet, unless a download is under way.
pub(super) fn close(mut adding: Adding) {
    let busy = adding
        .peek()
        .clone()
        .is_some_and(|form| form.button() == AddButton::Busy);
    if !busy {
        adding.set(None);
    }
}

/// Adds the assistant picked: lists it when it is installed, downloads it first when it is not.
pub(super) fn submit(env: &Env, mut adding: Adding, now: Signal<Shown>) {
    let Some(form) = adding.peek().clone() else {
        return;
    };
    if form.button() != AddButton::Ready {
        return;
    }
    match form.request() {
        Err(words) => {
            let mut refused = form;
            refused.refusal = Some(words.to_owned());
            adding.set(Some(refused));
        }
        Ok(Request::Add(new)) => write(env, adding, now, &new),
        Ok(Request::Install { id, version, then }) => {
            adding.set(Some(form.installing()));
            install(env, adding, now, (id, version), then);
        }
    }
}

/// Lists `new` in `agents.toml` and closes the sheet, or says why it could not.
fn write(env: &Env, mut adding: Adding, mut now: Signal<Shown>, new: &NewAgent) {
    match add_agent(&env.config_home, new) {
        Ok(()) => {
            adding.set(None);
            now.set(shown(env));
        }
        Err(fault) => change(&mut adding, |form| form.refused(fault)),
    }
}

/// Downloads `want` (an id and a version), then lists `then`. The sheet says so once the program
/// has begun, and shows its sentence when it fails.
fn install(
    env: &Env,
    mut adding: Adding,
    now: Signal<Shown>,
    want: (String, String),
    then: NewAgent,
) {
    let Some(program) = env.agents_program.clone() else {
        change(&mut adding, |form| form.install_failed(""));
        return;
    };
    let tool = AgentsTool::new(program, &env.data_home);
    let (begun, mut told) = tokio::sync::mpsc::unbounded_channel::<()>();
    spawn(async move {
        if told.recv().await.is_some() {
            change(&mut adding, AddForm::downloading);
        }
    });
    let env = env.clone();
    spawn(async move {
        let (id, version) = want;
        let done = tokio::task::spawn_blocking(move || {
            tool.install(&id, &version, || {
                let _ = begun.send(());
            })
        })
        .await;
        match done {
            Ok(Ok(())) => write(&env, adding, now, &then),
            Ok(Err(said)) => change(&mut adding, |form| form.install_failed(&said)),
            Err(_) => change(&mut adding, |form| form.install_failed("")),
        }
    });
}
