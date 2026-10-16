//! Test seams (feature `testing`): peers over an empty table that a test introduces owners to, so
//! no test reads the real caller table or asks the real unit manager.

use peephole_bus::peephole::fake::introduced_service;
use peephole_bus::peephole_core::{Pin, Proof, RoleName, Table};
use peephole_bus::{BusPeers, ManagerCheck};
use zbus::Connection;

/// Peers on `connection` with an empty table and an unchecked manager. A sender nobody introduced
/// is looked at through the real `/proc` of the test's own process, which is never a service.
pub fn private_peers(connection: &Connection) -> BusPeers {
    BusPeers::new(connection.clone(), Table::empty(), ManagerCheck::Unchecked)
}

/// Makes `owner` (a unique name) stand as the service holding `role`, proved in full, as if the
/// caller table had matched it.
pub fn introduce_service(peers: &BusPeers, owner: &str, role: &RoleName) {
    let standing = introduced_service(
        "org.quire.Test",
        &role.to_string(),
        Proof::Full,
        Pin::AtConnect,
    )
    .expect("a valid app and role");
    peers.introduce(owner, standing);
}
