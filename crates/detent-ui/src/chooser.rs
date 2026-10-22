//! A folder or a picture chosen by the person in the desktop's own chooser (the file chooser
//! portal), never typed: the one way a Strata they already have is pointed at, and the one way a
//! wallpaper is added.

use crate::live::{BusSource, connect};
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

/// What the person is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Want {
    /// A folder.
    Folder,
    /// Any file.
    File,
    /// A picture file.
    Picture,
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

/// What asking the person came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Asked {
    /// They chose a file on this computer: its path.
    Path(String),
    /// They closed the chooser, or chose nothing that is a file here.
    Closed,
    /// There is no chooser to ask.
    Unavailable,
}

/// Asks the person for a folder, a file or a picture, and waits for the answer; `None` when the chooser
/// is closed, there is none, or what was chosen is not a file on this computer.
pub(crate) async fn choose(bus: &BusSource, want: Want) -> Option<String> {
    match ask(bus, want).await {
        Asked::Path(path) => Some(path),
        Asked::Closed | Asked::Unavailable => None,
    }
}

/// Asks the person for a folder or a picture, and says how it ended.
pub(crate) async fn ask(bus: &BusSource, want: Want) -> Asked {
    ask_through(bus, want).await.unwrap_or(Asked::Unavailable)
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
            .and_then(|uri| path_of(uri))
            .map_or(Asked::Closed, Asked::Path),
    )
}

/// The path a `file://` address names, with its `%XX` escapes read; `None` for any other address.
fn path_of(uri: &str) -> Option<String> {
    let escaped = uri.strip_prefix("file://")?;
    let bytes = escaped.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let hex = |offset: usize| {
            bytes
                .get(at + offset)
                .and_then(|b| char::from(*b).to_digit(16))
        };
        match (bytes[at], hex(1), hex(2)) {
            (b'%', Some(high), Some(low)) => {
                out.push(u8::try_from(high * 16 + low).ok()?);
                at += 3;
            }
            (byte, _, _) => {
                out.push(byte);
                at += 1;
            }
        }
    }
    String::from_utf8(out)
        .ok()
        .filter(|path| path.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::path_of;

    #[test]
    fn a_file_address_is_read_as_the_path_it_names_and_nothing_else_is() {
        assert_eq!(
            path_of("file:///home/p/Strata"),
            Some("/home/p/Strata".to_owned())
        );
        assert_eq!(
            path_of("file:///mnt/data/my%20models/%C3%A9"),
            Some("/mnt/data/my models/\u{e9}".to_owned())
        );
        assert_eq!(path_of("https://example.org/x"), None);
        assert_eq!(path_of("file://host/x"), None);
        assert_eq!(path_of("file:///100%"), Some("/100%".to_owned()));
    }
}
