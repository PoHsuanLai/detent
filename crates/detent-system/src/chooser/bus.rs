//! The desktop's file chooser portal on the bus: one request, answered by a signal that is
//! listened for before the question is asked.

use super::{Asked, ChooserLink, Want};
use crate::bus::{BusSource, connect};
use detent_model::{Work, path_of_file_uri};
use std::collections::HashMap;
use zbus::export::futures_core::Stream;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const CHOOSER: &str = "org.freedesktop.portal.FileChooser";
const REQUEST: &str = "org.freedesktop.portal.Request";

/// The name of the one filter a picture request offers.
const PICTURES_FILTER: &str = "Pictures";

/// The kinds of file the filter lets through (the portal's type 1 is a MIME type).
const PICTURE_KINDS: &[&str] = &["image/png", "image/jpeg", "image/webp"];

/// The portal's code for a filter entry that names a MIME type.
const KIND_IS_MIME: u32 = 1;

/// The chooser portal of the person's session.
#[derive(Debug)]
pub struct OnPortal(pub BusSource);

impl ChooserLink for OnPortal {
    fn ask(&self, want: Want) -> Work<'_, Asked> {
        Box::pin(async move {
            ask_through(&self.0, want)
                .await
                .unwrap_or(Asked::Unavailable)
        })
    }
}

impl Want {
    /// Names the request, so its answer can be listened for before it is asked.
    fn token(self) -> &'static str {
        match self {
            Want::Folder => "detent_engine_folder",
            Want::File => "detent_corner_file",
            Want::Picture => "detent_wallpaper_picture",
        }
    }

    /// What the chooser's window is called.
    fn title(self) -> &'static str {
        match self {
            Want::Folder => "Choose a Folder",
            Want::File => "Choose a File",
            Want::Picture => "Add Picture",
        }
    }

    /// The chooser's options for it.
    fn options(self) -> HashMap<&'static str, Value<'static>> {
        let mut options = HashMap::from([("handle_token", Value::from(self.token()))]);
        match self {
            Want::Folder => {
                options.insert("directory", Value::from(true));
            }
            Want::File => {}
            Want::Picture => {
                // Without the filter, every file is offered and a refused one is a silent no-op.
                let (filters, current) = picture_filters();
                options.insert("filters", filters);
                options.insert("current_filter", current);
            }
        }
        options
    }
}

/// The portal's `filters` (`a(sa(us))`: one "Pictures" filter of MIME types) and the same filter
/// as its `current_filter` (`(sa(us))`).
fn picture_filters() -> (Value<'static>, Value<'static>) {
    let filter = || {
        (
            PICTURES_FILTER,
            PICTURE_KINDS
                .iter()
                .map(|kind| (KIND_IS_MIME, *kind))
                .collect::<Vec<(u32, &'static str)>>(),
        )
    };
    (Value::from(vec![filter()]), Value::from(filter()))
}

/// `ask`, with `None` for any step of reaching the chooser that failed.
async fn ask_through(bus: &BusSource, want: Want) -> Option<Asked> {
    let connection = connect(bus).await.ok()?;
    // The path the portal will give the request, from the name of this connection and the token.
    let sender = connection
        .unique_name()?
        .as_str()
        .trim_start_matches(':')
        .replace('.', "_");
    let request = zbus::Proxy::new(
        &connection,
        PORTAL,
        format!("{PORTAL_PATH}/request/{sender}/{}", want.token()),
        REQUEST,
    )
    .await
    .ok()?;
    let mut answers = request.receive_signal("Response").await.ok()?;
    let chooser = zbus::Proxy::new(&connection, PORTAL, PORTAL_PATH, CHOOSER)
        .await
        .ok()?;
    let _handle: OwnedObjectPath = chooser
        .call("OpenFile", &("", want.title(), want.options()))
        .await
        .ok()?;
    let answer = std::future::poll_fn(|cx| std::pin::Pin::new(&mut answers).poll_next(cx)).await?;
    let (code, results): (u32, HashMap<String, OwnedValue>) = answer.body().deserialize().ok()?;
    if code != 0 {
        return Some(Asked::Closed);
    }
    let uris = Vec::<String>::try_from(results.get("uris")?.try_clone().ok()?).ok()?;
    Some(
        uris.first()
            .and_then(|uri| path_of_file_uri(uri))
            .map_or(Asked::Closed, Asked::Path),
    )
}
