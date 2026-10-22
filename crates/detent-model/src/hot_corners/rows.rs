//! A corner's rows in the sheet: the pop-up that fires with nothing held, then one row for each
//! other set of keys, and the changes the person makes to them. A set of keys has at most one row.

use super::binding::{HotBinding, HotDo, one_per_keys};
use super::keys::HotKeys;

/// Why a set of keys cannot be added as a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotAddFault {
    /// No key is ticked: the row without keys is always there.
    NoKeys,
    /// The corner already has a row for these keys.
    Already,
}

impl HotAddFault {
    /// The sentence that says why nothing was added.
    pub fn words(self) -> &'static str {
        match self {
            HotAddFault::NoKeys => "Tick at least one key.",
            HotAddFault::Already => "This corner already has a row for those keys.",
        }
    }
}

/// The sets of keys the corner shows a pop-up for: none held, then the sets that have a binding
/// or that the person added and has not filled yet (`added`), each once, in key order.
pub fn shown_rows(bindings: &[HotBinding], added: &[HotKeys]) -> Vec<HotKeys> {
    let mut rows: Vec<HotKeys> = std::iter::once(HotKeys::default())
        .chain(bindings.iter().map(|binding| binding.keys.clone()))
        .chain(added.iter().cloned())
        .collect();
    rows.sort();
    rows.dedup();
    rows
}

/// `added` with a row for `keys`, or why there cannot be one.
pub fn add_row(
    bindings: &[HotBinding],
    added: &[HotKeys],
    keys: HotKeys,
) -> Result<Vec<HotKeys>, HotAddFault> {
    if keys.is_empty() {
        return Err(HotAddFault::NoKeys);
    }
    let rows = shown_rows(bindings, added);
    if rows.contains(&keys) {
        return Err(HotAddFault::Already);
    }
    Ok(added.iter().cloned().chain(std::iter::once(keys)).collect())
}

/// The bindings and the added rows after the row for `keys` is removed. The row without keys
/// cannot be removed; its pop-up is set to nothing instead.
pub fn remove_row(
    bindings: &[HotBinding],
    added: &[HotKeys],
    keys: &HotKeys,
) -> (Vec<HotBinding>, Vec<HotKeys>) {
    let kept = |binding: &&HotBinding| keys.is_empty() || &binding.keys != keys;
    let bindings = bindings.iter().filter(kept).cloned().collect();
    let added = added
        .iter()
        .filter(|row| keys.is_empty() || *row != keys)
        .cloned()
        .collect();
    (bindings, added)
}

/// The corner's bindings with the one for `keys` set to `does`, or gone when `does` is `None`.
pub fn with_binding(
    bindings: &[HotBinding],
    keys: &HotKeys,
    does: Option<HotDo>,
) -> Vec<HotBinding> {
    let others = bindings
        .iter()
        .filter(|binding| &binding.keys != keys)
        .cloned();
    let set = does.map(|does| HotBinding {
        keys: keys.clone(),
        does,
    });
    one_per_keys(others.chain(set).collect())
}

/// What the corner does with exactly `keys` held.
pub fn binding_for<'a>(bindings: &'a [HotBinding], keys: &HotKeys) -> Option<&'a HotDo> {
    bindings
        .iter()
        .find(|binding| &binding.keys == keys)
        .map(|binding| &binding.does)
}
