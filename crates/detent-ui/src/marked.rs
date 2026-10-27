//! A `data-*` mark on a quire component, which the window's tests address a row by.

use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};

/// The `common` of a component that carries `data-<name>="<value>"`; none when `name` is not a
/// valid attribute name.
pub(crate) fn marked(name: &'static str, value: &str) -> Common {
    Common {
        data: DataName::parse(name)
            .map(|name| vec![DataAttr::new(name, value.to_owned())])
            .unwrap_or_default(),
        ..Common::default()
    }
}
