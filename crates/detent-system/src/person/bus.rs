//! The person from AccountsService (`org.freedesktop.Accounts`) on the system bus, which a page
//! only looks at; when it is not there, the passwd file and `~/.face`.

use super::PersonLink;
use crate::bus::{BusSource, system_connection};
use crate::deadline::{Deadline, peer};
use detent_model::{Person, Work, passwd_name};
use ds_peer::within;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use zbus::Proxy;
use zbus::zvariant::OwnedObjectPath;

const SERVICE: &str = "org.freedesktop.Accounts";
const MANAGER_PATH: &str = "/org/freedesktop/Accounts";
const MANAGER: &str = "org.freedesktop.Accounts";
const USER: &str = "org.freedesktop.Accounts.User";

/// The picture a person keeps in their home folder when AccountsService does not name one.
const FACE: &str = ".face";

/// The person behind the session, from the system's accounts service, else from files.
#[derive(Debug, Clone)]
pub struct OnAccounts {
    bus: BusSource,
    passwd: PathBuf,
    home: PathBuf,
}

impl OnAccounts {
    /// The person of this session, looked for on `bus`, else in the passwd file at `passwd` and
    /// the home folder `home`.
    pub fn new(bus: BusSource, passwd: PathBuf, home: PathBuf) -> OnAccounts {
        OnAccounts { bus, passwd, home }
    }

    async fn read(&self) -> Option<Person> {
        let uid = std::fs::metadata("/proc/self").ok()?.uid();
        let asked = within(Deadline::READ, peer::DESKTOP, from_accounts(&self.bus, uid)).await;
        if let Ok(Some(person)) = asked {
            return Some(person);
        }
        let text = std::fs::read_to_string(&self.passwd).ok()?;
        Some(Person {
            name: passwd_name(&text, uid)?,
            picture: face_in(&self.home),
        })
    }
}

impl PersonLink for OnAccounts {
    fn who(&self) -> Work<'_, Option<Person>> {
        Box::pin(self.read())
    }
}

/// The `.face` file of `home`, when there is one.
fn face_in(home: &Path) -> Option<PathBuf> {
    let face = home.join(FACE);
    face.is_file().then_some(face)
}

/// `uid`'s real name and picture as AccountsService keeps them; none when it is not there, does
/// not know the user, or keeps no name.
async fn from_accounts(bus: &BusSource, uid: u32) -> Option<Person> {
    let connection = system_connection(bus).await.ok()?;
    let manager = Proxy::new(&connection, SERVICE, MANAGER_PATH, MANAGER)
        .await
        .ok()?;
    let found: OwnedObjectPath = manager
        .call("FindUserById", &(i64::from(uid),))
        .await
        .ok()?;
    let user = Proxy::new(&connection, SERVICE, found.as_str().to_owned(), USER)
        .await
        .ok()?;
    let name: String = user.get_property("RealName").await.ok()?;
    let icon: String = user.get_property("IconFile").await.unwrap_or_default();
    let picture = Some(PathBuf::from(icon)).filter(|path| path.is_file());
    (!name.trim().is_empty()).then_some(Person { name, picture })
}
