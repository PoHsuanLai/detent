//! An invocation read as what Settings can do. Pure: it decides nothing, it only names the
//! action and its arguments, or says which argument is wrong.

use crate::answer::{not_found, refusal};
use detent_model::PageSlug;
use docket_core::{AppRefusal, Invocation, ParamName, Value};
use ds_settings::schema::KeyPath;
use porter_core::AppName;

/// What a call asks of Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ask {
    /// Show a page.
    OpenPage(PageSlug),
    /// Tell the value of a key.
    ReadKey(KeyPath),
    /// Set a key to the stored form `text`.
    SetKey { key: KeyPath, text: String },
    /// Tell the accounts, one line each.
    ListAccounts,
    /// Show the Accounts page.
    OpenAccounts,
    /// Show the Intelligence page.
    OpenIntelligence,
}

fn param(name: &str) -> Result<ParamName, AppRefusal> {
    ParamName::parse(name).map_err(|_| AppRefusal::Unsupported)
}

/// The key of the entity argument `name`, which must be of `kind` and belong to `app`.
fn entity_arg(
    inv: &Invocation,
    app: &AppName,
    name: &str,
    kind: &str,
) -> Result<String, AppRefusal> {
    let wanted = param(name)?;
    let Some(arg) = inv.args.get(&wanted) else {
        return Err(AppRefusal::NeedsParam {
            param: wanted,
            options: Vec::new(),
        });
    };
    match &arg.value {
        Value::Entity(id) if &id.app == app && id.kind.as_str() == kind => {
            Ok(id.key.as_str().to_owned())
        }
        Value::Entity(id) => Err(not_found(app, id.kind.as_str(), id.key.as_str())),
        _ => Err(refusal(format!("{name} must name a {kind}"))),
    }
}

/// The text of the argument `name`; a whole number is taken as its digits.
fn text_arg(inv: &Invocation, name: &str) -> Result<String, AppRefusal> {
    let wanted = param(name)?;
    match inv.args.get(&wanted).map(|arg| &arg.value) {
        None => Err(AppRefusal::NeedsParam {
            param: wanted,
            options: Vec::new(),
        }),
        Some(Value::Text(text)) => Ok(text.clone()),
        Some(Value::Integer(n)) => Ok(n.to_string()),
        Some(_) => Err(refusal(format!("{name} must be text"))),
    }
}

/// What `inv` asks, or why it cannot be asked.
pub(crate) fn ask(inv: &Invocation, app: &AppName) -> Result<Ask, AppRefusal> {
    match inv.action.as_str() {
        "settings.page.show" => {
            entity_arg(inv, app, "page", "settings.page").map(|slug| Ask::OpenPage(PageSlug(slug)))
        }
        "settings.key.read" => {
            entity_arg(inv, app, "key", "settings.key").map(|key| Ask::ReadKey(KeyPath(key)))
        }
        "settings.key.set" => Ok(Ask::SetKey {
            key: KeyPath(entity_arg(inv, app, "key", "settings.key")?),
            text: text_arg(inv, "value")?,
        }),
        "settings.accounts.list" => Ok(Ask::ListAccounts),
        "settings.accounts.open" => Ok(Ask::OpenAccounts),
        "settings.intelligence.open" => Ok(Ask::OpenIntelligence),
        _ => Err(AppRefusal::Unsupported),
    }
}
