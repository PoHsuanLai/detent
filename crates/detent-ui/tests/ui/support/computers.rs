//! A fake inferd (`org.quire.Inference1`) for the Your computers group: the places, the Tailscale
//! candidates and the guests, on a private bus. Every call that changes something is written, in
//! order, into one log; a change can be made to fail with one of inferd's named refusals.
#![allow(dead_code)]

use super::bus::PrivateBus;
use porter_dbus::{
    CANDIDATE_KEY_MODELS, CANDIDATE_KEY_NAME, CANDIDATE_KEY_NEEDS_APPROVAL, GUEST_KEY_NAME,
    GUEST_KEY_SINCE, GUEST_KEY_STATE, INFERENCE_BUS, INFERENCE_PATH, PLACE_KEY_KIND,
    PLACE_KEY_MODELS, PLACE_KEY_NAME, PLACE_KEY_PROVIDER, PLACE_KEY_READY,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Value};

type Details = HashMap<String, OwnedValue>;

/// What the fake was asked to change, in order.
pub type Log = Arc<Mutex<Vec<String>>>;

/// A computer the person has, as `Places` lists it.
#[derive(Clone)]
pub struct Own {
    pub name: &'static str,
    pub models: &'static [(&'static str, &'static str)],
    pub ready: bool,
}

/// A Tailscale computer that lends.
#[derive(Clone)]
pub struct Lender {
    pub node: &'static str,
    pub name: &'static str,
    pub models: &'static [(&'static str, &'static str)],
    pub needs_approval: bool,
}

/// A computer that asked to use this one's models.
#[derive(Clone)]
pub struct Asker {
    pub node: &'static str,
    pub name: &'static str,
    pub state: &'static str,
}

#[derive(Default)]
struct State {
    own: Vec<Own>,
    lenders: Vec<Lender>,
    askers: Vec<Asker>,
    /// The name of the refusal the next change answers with.
    refuse: Option<&'static str>,
}

fn owned(value: Value<'_>) -> OwnedValue {
    OwnedValue::try_from(value).expect("a plain value")
}

fn models_of(models: &[(&str, &str)]) -> OwnedValue {
    let list: Vec<(String, String)> = models
        .iter()
        .map(|(id, name)| ((*id).to_owned(), (*name).to_owned()))
        .collect();
    owned(Value::new(list))
}

/// An error with one of inferd's computer names.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.quire.Inference1.Error")]
enum Refusal {
    #[zbus(error)]
    ZBus(zbus::Error),
    #[zbus(name = "Computer.NotOnTailscale")]
    NotOnTailscale(String),
    #[zbus(name = "Computer.NotAnswering")]
    NotAnswering(String),
    #[zbus(name = "Computer.NotAsking")]
    NotAsking(String),
    #[zbus(name = "Computer.TooManyAsking")]
    TooManyAsking(String),
}

struct Inference {
    state: Arc<Mutex<State>>,
    log: Log,
}

impl Inference {
    fn refusal(&self) -> Result<(), Refusal> {
        let name = self.state.lock().expect("state").refuse.take();
        let words = format!("The sentence for {}.", name.unwrap_or_default());
        match name {
            None => Ok(()),
            Some("NotOnTailscale") => Err(Refusal::NotOnTailscale(words)),
            Some("NotAnswering") => Err(Refusal::NotAnswering(words)),
            Some("NotAsking") => Err(Refusal::NotAsking(words)),
            Some(_) => Err(Refusal::TooManyAsking(words)),
        }
    }

    fn note(&self, line: String) {
        self.log.lock().expect("log").push(line);
    }
}

#[zbus::interface(name = "org.quire.Inference1")]
impl Inference {
    async fn places(&self) -> Vec<(String, Details)> {
        let state = self.state.lock().expect("state");
        let this: Details = [
            (PLACE_KEY_KIND, owned(Value::from("this_computer"))),
            (PLACE_KEY_NAME, owned(Value::from("This computer"))),
            (PLACE_KEY_MODELS, models_of(&[("local", "Local")])),
            (PLACE_KEY_READY, owned(Value::from(true))),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
        let cloud: Details = [
            (PLACE_KEY_KIND, owned(Value::from("cloud_account"))),
            (PLACE_KEY_NAME, owned(Value::from("me@acme.example"))),
            (PLACE_KEY_PROVIDER, owned(Value::from("Acme"))),
            (PLACE_KEY_MODELS, models_of(&[("acme-1", "Acme One")])),
            (PLACE_KEY_READY, owned(Value::from(true))),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
        let mut rows = vec![("this-computer".to_owned(), this)];
        rows.extend(state.own.iter().map(|own| {
            let details: Details = [
                (PLACE_KEY_KIND, owned(Value::from("own_computer"))),
                (PLACE_KEY_NAME, owned(Value::from(own.name))),
                (PLACE_KEY_MODELS, models_of(own.models)),
                (PLACE_KEY_READY, owned(Value::from(own.ready))),
            ]
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect();
            (format!("computer:{}", own.name), details)
        }));
        rows.push(("account:acme".to_owned(), cloud));
        rows
    }

    async fn candidates(&self) -> Vec<(String, Details)> {
        let state = self.state.lock().expect("state");
        state
            .lenders
            .iter()
            .map(|lender| {
                let details: Details = [
                    (CANDIDATE_KEY_NAME, owned(Value::from(lender.name))),
                    (CANDIDATE_KEY_MODELS, models_of(lender.models)),
                    (
                        CANDIDATE_KEY_NEEDS_APPROVAL,
                        owned(Value::from(lender.needs_approval)),
                    ),
                ]
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect();
                (lender.node.to_owned(), details)
            })
            .collect()
    }

    async fn add_tailnet_computer(
        &self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        node: String,
    ) -> Result<String, Refusal> {
        self.refusal()?;
        self.note(format!("add {node}"));
        let name = {
            let mut state = self.state.lock().expect("state");
            let at = state.lenders.iter().position(|l| l.node == node);
            let Some(at) = at else {
                return Err(Refusal::NotOnTailscale(String::new()));
            };
            let lender = state.lenders.remove(at);
            state.own.push(Own {
                name: lender.name,
                models: lender.models,
                ready: true,
            });
            lender.name
        };
        let _ = Self::engines_changed(&emitter).await;
        Ok(format!("computer:{name}"))
    }

    async fn remove_computer(
        &self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        name: String,
    ) -> Result<(), Refusal> {
        self.refusal()?;
        self.note(format!("remove {name}"));
        self.state
            .lock()
            .expect("state")
            .own
            .retain(|own| own.name != name);
        let _ = Self::engines_changed(&emitter).await;
        Ok(())
    }

    async fn guests(&self) -> Vec<(String, Details)> {
        let state = self.state.lock().expect("state");
        state
            .askers
            .iter()
            .map(|asker| {
                let details: Details = [
                    (GUEST_KEY_NAME, owned(Value::from(asker.name))),
                    (GUEST_KEY_STATE, owned(Value::from(asker.state))),
                    (GUEST_KEY_SINCE, owned(Value::from(1_700_000_000_i64))),
                ]
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect();
                (asker.node.to_owned(), details)
            })
            .collect()
    }

    async fn answer_guest(
        &self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        node: String,
        answer: String,
    ) -> Result<(), Refusal> {
        self.refusal()?;
        self.note(format!("answer {node} {answer}"));
        let state = if answer == "allow" {
            "approved"
        } else {
            "denied"
        };
        for asker in &mut self.state.lock().expect("state").askers {
            if asker.node == node {
                asker.state = state;
            }
        }
        let _ = Self::guests_changed(&emitter).await;
        Ok(())
    }

    async fn forget_guest(
        &self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        node: String,
    ) -> Result<(), Refusal> {
        self.refusal()?;
        self.note(format!("forget {node}"));
        self.state
            .lock()
            .expect("state")
            .askers
            .retain(|asker| asker.node != node);
        let _ = Self::guests_changed(&emitter).await;
        Ok(())
    }

    #[zbus(signal)]
    async fn guests_changed(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn engines_changed(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

/// The fake, kept alive while the test runs.
pub struct Fake {
    pub log: Log,
    state: Arc<Mutex<State>>,
    _connection: zbus::Connection,
}

impl Fake {
    /// The next change is refused with the refusal named `name` (`NotAnswering`...).
    pub fn refuse_next(&self, name: &'static str) {
        self.state.lock().expect("state").refuse = Some(name);
    }

    /// A computer begins asking; inferd says the guests changed.
    pub async fn asks(&self, asker: Asker) {
        self.state.lock().expect("state").askers.push(asker);
        let served = self
            ._connection
            .object_server()
            .interface::<_, Inference>(INFERENCE_PATH)
            .await
            .expect("inferd is served");
        Inference::guests_changed(served.signal_emitter())
            .await
            .expect("the signal goes out");
    }

    /// What was changed, in order.
    pub fn changes(&self) -> Vec<String> {
        self.log.lock().expect("log").clone()
    }
}

/// Serves inferd holding `own` computers, `lenders` on Tailscale and `askers`.
pub async fn serve(
    bus: &PrivateBus,
    own: Vec<Own>,
    lenders: Vec<Lender>,
    askers: Vec<Asker>,
) -> Fake {
    let log: Log = Arc::default();
    let state = Arc::new(Mutex::new(State {
        own,
        lenders,
        askers,
        refuse: None,
    }));
    let connection = bus.connect().await;
    connection
        .object_server()
        .at(
            INFERENCE_PATH,
            Inference {
                state: state.clone(),
                log: log.clone(),
            },
        )
        .await
        .expect("inferd served");
    connection
        .request_name(INFERENCE_BUS)
        .await
        .expect("inferd's name");
    Fake {
        log,
        state,
        _connection: connection,
    }
}
