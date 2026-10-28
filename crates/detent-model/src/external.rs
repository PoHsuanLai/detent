//! The types of other crates that the model's facts are made of, named once here so the window
//! names only `detent-model` and `detent-client`, never almanac, chordkit, docket, porter or
//! toml directly. Each is an id or a fact the model carries as it is; the window words it or
//! hands it back, and holds no rule of its own about it.

pub use almanac_core::{HistoryFate, MemoryFate};
pub use chordkit::{Action, StandardAction};
pub use docket_settings::{
    AddFault, AgentChoice, AgentRow, Agents, ChoiceFault, CloudAccount, CloudAllowed, ModelChoice,
    NewAgent, OwnComputer, PlaceKind, PlaceName, Toggle, kind_label,
};
pub use porter_capability::ModelId;
pub use porter_core::{AccountId, SecretText, SignInWay};
pub use porter_ids::word::word;
pub use porter_infer::{ComputerName, Declined, DeclinedBecause, InferRefusal, ModelRef, Why};
pub use porter_sheet::ProviderGroup;
pub use porter_tailnet::{Approval, ComputerCandidate, GuestAnswer, GuestRow, RowState};
/// The value of a settings key as the files hold it.
pub use toml;
