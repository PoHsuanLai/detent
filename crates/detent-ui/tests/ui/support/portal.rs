//! A fake screen-sharing portal: `org.quire.Portal1` at the portal path on the portal's name on a
//! private bus, holding the apps a test sets. It writes down every call, and can be made to
//! refuse the next removal.
#![allow(dead_code)]

use super::bus::PrivateBus;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use zbus::fdo;

/// The portal's name on the bus, which the owner check knows the role of.
pub const NAME: &str = "org.freedesktop.impl.portal.desktop.quire";
const PATH: &str = "/org/freedesktop/portal/desktop";

/// An app the portal remembers.
#[derive(Clone)]
pub struct Remembered {
    pub app: &'static str,
    /// The portal's own name for the app, from its desktop entry. The window must not use it.
    pub label: &'static str,
    /// 1 while the app runs, 2 until taken back.
    pub mode: u32,
    pub granted: u64,
}

type Row = (String, String, u32, u64);

struct Portal {
    rows: Arc<Mutex<Vec<Row>>>,
    listed: Arc<AtomicUsize>,
    revoked: Arc<Mutex<Vec<String>>>,
    refuse: Arc<AtomicBool>,
}

#[zbus::interface(name = "org.quire.Portal1")]
impl Portal {
    async fn list_grants(&self) -> Vec<Row> {
        self.listed.fetch_add(1, Ordering::SeqCst);
        self.rows.lock().expect("rows").clone()
    }

    async fn revoke(&self, app_id: String) -> fdo::Result<u32> {
        if self.refuse.swap(false, Ordering::SeqCst) {
            return Err(fdo::Error::Failed("not changed".to_owned()));
        }
        self.revoked.lock().expect("revoked").push(app_id.clone());
        let mut rows = self.rows.lock().expect("rows");
        let before = rows.len();
        rows.retain(|(app, ..)| *app != app_id);
        Ok(u32::try_from(before - rows.len()).unwrap_or(u32::MAX))
    }
}

/// The fake, kept alive while the test runs.
pub struct Fake {
    listed: Arc<AtomicUsize>,
    revoked: Arc<Mutex<Vec<String>>>,
    refuse: Arc<AtomicBool>,
    pub connection: zbus::Connection,
}

impl Fake {
    /// How many times the list was asked for.
    pub fn listed(&self) -> usize {
        self.listed.load(Ordering::SeqCst)
    }

    /// The apps taken back, in order.
    pub fn revoked(&self) -> Vec<String> {
        self.revoked.lock().expect("revoked").clone()
    }

    /// The next removal is refused.
    pub fn refuse_next(&self) {
        self.refuse.store(true, Ordering::SeqCst);
    }
}

/// The portal holding `apps` on its own connection, without a name.
pub async fn serve_unnamed(bus: &PrivateBus, apps: &[Remembered]) -> Fake {
    let rows = apps
        .iter()
        .map(|a| (a.app.to_owned(), a.label.to_owned(), a.mode, a.granted))
        .collect();
    let fake = Fake {
        listed: Arc::default(),
        revoked: Arc::default(),
        refuse: Arc::default(),
        connection: bus.connect().await,
    };
    fake.connection
        .object_server()
        .at(
            PATH,
            Portal {
                rows: Arc::new(Mutex::new(rows)),
                listed: Arc::clone(&fake.listed),
                revoked: Arc::clone(&fake.revoked),
                refuse: Arc::clone(&fake.refuse),
            },
        )
        .await
        .expect("portal served");
    fake
}

/// The portal holding `apps`, owning the portal's name.
pub async fn serve(bus: &PrivateBus, apps: &[Remembered]) -> Fake {
    let fake = serve_unnamed(bus, apps).await;
    fake.connection
        .request_name(NAME)
        .await
        .expect("the portal's name");
    fake
}
