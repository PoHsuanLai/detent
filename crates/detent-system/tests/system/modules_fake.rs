//! The in-memory modules link: a module hosted in the process is opened, refused as the skeleton
//! refuses, heard when it changes, and found again after it goes away and comes back.

use crate::modules_bus::{Policy, ROW, Words, daemon, is_text};
use detent_model::Absence;
use detent_system::modules::{FakeModules, Followed, ModulesLink};
use ds_settings::schema::KeyPath;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

fn word(text: &str) -> toml::Value {
    toml::Value::String(text.to_owned())
}

/// The next thing a follower is told, or a failure when nothing comes.
async fn next(heard: &mut UnboundedReceiver<Followed>) -> Followed {
    heard.recv().await.expect("the follower is still there")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_hosted_module_is_opened_and_a_set_is_told_to_the_follower() {
    let modules = FakeModules::new();
    let daemon = daemon();
    modules.host(&daemon, Words::holding("first", Policy::Allow));
    let (told, mut heard) = unbounded_channel();
    let following = modules.follow(&daemon, is_text, told);
    let checking = async {
        let Followed::Opened(module, described) = next(&mut heard).await else {
            panic!("expected the module to open");
        };
        let row = KeyPath(ROW.to_owned());
        assert_eq!(described.values.get(&row), Some(&"first".to_owned()));
        module.set(&row, &word("second")).await.expect("taken");
        let Followed::Changed(path, value) = next(&mut heard).await else {
            panic!("expected the change");
        };
        assert_eq!((path, value), (row, word("second")));
    };
    tokio::select! {
        () = following => panic!("the follower ended"),
        () = checking => {}
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_module_that_refuses_is_gone_as_not_permitted() {
    let modules = FakeModules::new();
    let daemon = daemon();
    modules.host(&daemon, Words::holding("first", Policy::Refuse));
    let (told, mut heard) = unbounded_channel();
    let following = modules.follow(&daemon, is_text, told);
    let checking = async {
        let Followed::Gone(why) = next(&mut heard).await else {
            panic!("expected the module to be refused");
        };
        assert_eq!(why, Absence::NotPermitted);
    };
    tokio::select! {
        () = following => panic!("the follower ended"),
        () = checking => {}
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_module_that_goes_away_and_comes_back_is_read_again() {
    let modules = FakeModules::new();
    let daemon = daemon();
    let (told, mut heard) = unbounded_channel();
    let following = modules.follow(&daemon, is_text, told);
    let checking = async {
        assert!(matches!(
            next(&mut heard).await,
            Followed::Gone(Absence::NotRunning)
        ));
        modules.host(&daemon, Words::holding("first", Policy::Allow));
        assert!(matches!(next(&mut heard).await, Followed::Opened(..)));
        modules.away(&daemon, Absence::Unavailable);
        assert!(matches!(
            next(&mut heard).await,
            Followed::Gone(Absence::Unavailable)
        ));
        modules.host(&daemon, Words::holding("second", Policy::Allow));
        let Followed::Opened(_, described) = next(&mut heard).await else {
            panic!("expected the module to come back");
        };
        let row = KeyPath(ROW.to_owned());
        assert_eq!(described.values.get(&row), Some(&"second".to_owned()));
    };
    tokio::select! {
        () = following => panic!("the follower ended"),
        () = checking => {}
    }
}
