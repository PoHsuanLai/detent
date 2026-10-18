//! What lending this computer's models to the person's other computers touches: porter's file for
//! it and the person's own service manager. Behind one handle that the environment carries, so a
//! test hands in scratch folders and a manager that only writes down what it was asked.

use crate::deadline::{Deadline, peer};
use detent_files::ConfigHome;
use detent_model::{Existing, Work};
use ds_peer::within;
use porter_client::{
    LendingConfig, LendingState, SessionUnits, TailnetLending, UnitFailure, UnitManager, UnitName,
};
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

/// What the person reads when there is nowhere to keep the setting.
const NO_HOME: &str = "There is no home folder to keep this setting in, so nothing was changed.";

/// What the person reads when the computer's own services cannot be reached.
const NO_SERVICES: &str = "This computer's services could not be reached, so nothing was changed.";

/// The two things the switch does, and what is there before it does them.
trait Lend: Debug + Send + Sync {
    fn existing(&self) -> Existing;
    fn enable(&self) -> Work<'_, Result<(), String>>;
    fn disable(&self) -> Work<'_, Result<(), String>>;
}

fn existing_of(state: LendingState) -> Existing {
    match state {
        LendingState::NotInstalled => Existing::Absent,
        LendingState::Installed => Existing::Same,
        _ => Existing::Different,
    }
}

/// Lending by `config` over a manager that is given.
#[derive(Debug)]
struct Over<M>(TailnetLending<M>);

impl<M: UnitManager + Debug + 'static> Lend for Over<M> {
    fn existing(&self) -> Existing {
        existing_of(self.0.state())
    }

    fn enable(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async move { self.0.enable().await.map_err(|e| e.to_string()) })
    }

    fn disable(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async move { self.0.disable().await.map_err(|e| e.to_string()) })
    }
}

/// A manager that is never asked: reading what is there needs none.
#[derive(Debug)]
struct NoUnits;

impl UnitManager for NoUnits {
    async fn reload(&self) -> Result<(), UnitFailure> {
        Err(UnitFailure(NO_SERVICES.to_owned()))
    }

    async fn restart(&self, _unit: UnitName) -> Result<(), UnitFailure> {
        Err(UnitFailure(NO_SERVICES.to_owned()))
    }
}

/// The real thing: the person's session, reached when a change is made.
#[derive(Debug)]
struct Session(LendingConfig);

impl Session {
    async fn lending(&self) -> Result<TailnetLending<SessionUnits>, String> {
        let units = SessionUnits::session()
            .await
            .map_err(|_| NO_SERVICES.to_owned())?;
        Ok(TailnetLending::new(self.0.clone(), units))
    }
}

impl Lend for Session {
    fn existing(&self) -> Existing {
        existing_of(TailnetLending::new(self.0.clone(), NoUnits).state())
    }

    fn enable(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async move {
            let lending = self.lending().await?;
            lending.enable().await.map_err(|e| e.to_string())
        })
    }

    fn disable(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async move {
            let lending = self.lending().await?;
            lending.disable().await.map_err(|e| e.to_string())
        })
    }
}

/// This computer has no home folder: nothing can be kept.
#[derive(Debug)]
struct Nowhere;

impl Lend for Nowhere {
    fn existing(&self) -> Existing {
        Existing::Absent
    }

    fn enable(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async { Err(NO_HOME.to_owned()) })
    }

    fn disable(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async { Err(NO_HOME.to_owned()) })
    }
}

/// How the switch "Let my other computers use this computer's models" reaches the files and the
/// service manager.
#[derive(Debug, Clone)]
pub struct Lender(Arc<dyn Lend>);

impl Lender {
    /// The person's own session: the shipped file is looked for where the environment says
    /// (`XDG_DATA_HOME`, `XDG_DATA_DIRS`), the setting kept under `XDG_CONFIG_HOME`.
    pub fn from_env() -> Lender {
        LendingConfig::from_env().map_or_else(|| Lender(Arc::new(Nowhere)), Lender::session)
    }

    /// The person's own session, with porter's file at `<prefix>/share/porter/` and the setting
    /// kept under `config_home`.
    pub fn under_prefix(prefix: &Path, config_home: &ConfigHome) -> Lender {
        Lender::session(LendingConfig::under_prefix(
            prefix,
            config_home.0.join("systemd/user"),
        ))
    }

    /// The person's own session over `config`.
    pub fn session(config: LendingConfig) -> Lender {
        Lender(Arc::new(Session(config)))
    }

    /// Lending by `config` over `manager`: what a test hands in.
    pub fn over<M: UnitManager + Debug + 'static>(config: LendingConfig, manager: M) -> Lender {
        Lender(Arc::new(Over(TailnetLending::new(config, manager))))
    }

    /// What is set up now.
    pub(crate) fn existing(&self) -> Existing {
        self.0.existing()
    }

    /// Sets sharing up; the sentence that says why not when it could not be.
    pub(crate) async fn enable(&self) -> Result<(), String> {
        within(Deadline::ACTION, peer::SERVICES, self.0.enable())
            .await
            .map_err(|silent| silent.to_string())?
    }

    /// Takes sharing away; the sentence that says why not when it could not be.
    pub(crate) async fn disable(&self) -> Result<(), String> {
        within(Deadline::ACTION, peer::SERVICES, self.0.disable())
            .await
            .map_err(|silent| silent.to_string())?
    }
}
