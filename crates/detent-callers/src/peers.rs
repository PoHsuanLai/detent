//! The peers the desktop judges a bus message's sender with.

use peephole_bus::peephole::{RealProc, TableFiles};
use peephole_bus::peephole_core::{Proof, Requirement, Table, roles};
use peephole_bus::{BusPeers, ManagerCheck};
use zbus::Connection;

/// Peers on `connection`, judging against the desktop's caller table (the root-owned files peephole
/// loads) and believing the unit manager only when its own owner is the system's.
///
/// A table that cannot be read leaves an empty one, so no program is a service and every question
/// is refused: Settings then shows its services as unavailable instead of trusting a name.
///
/// A build with `dev-callers` adds the rows of the person's own file (`detent --dev-callers`) after
/// the table; a release build has no such rows.
pub fn desktop_peers(connection: &Connection) -> BusPeers {
    let table = TableFiles::desktop()
        .load(&RealProc::system())
        .unwrap_or_else(|why| {
            eprintln!("detent: the list of the desktop's own programs could not be read ({why}); no program is trusted");
            Table::empty()
        });
    #[cfg(feature = "dev-callers")]
    let table = crate::dev::with_dev_rows(table);
    let manager = ManagerCheck::Verified(Requirement::new(roles::MANAGER, Proof::Exe));
    BusPeers::new(connection.clone(), table, manager)
}
