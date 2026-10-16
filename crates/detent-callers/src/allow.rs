//! What lets a caller in: a role, proved at the level a question needs.

use peephole_bus::peephole_core::{Proof, Requirement, RoleName};

/// The ways to be allowed as `role`: proved at least `proof`; and, in a build with `dev-callers`,
/// also at `Proof::Dev`, which only a stand-in named in the person's own rows file
/// (`detent --dev-callers`) has at that standing. A release build lists the first way only.
pub fn allowing(role: RoleName, proof: Proof) -> Vec<Requirement> {
    #[cfg(feature = "dev-callers")]
    {
        vec![
            Requirement::new(role.clone(), proof),
            Requirement::dev(role),
        ]
    }
    #[cfg(not(feature = "dev-callers"))]
    {
        vec![Requirement::new(role, proof)]
    }
}
