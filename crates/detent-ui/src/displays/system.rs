//! The compositor's output management, as the Displays page's link. A change goes through the
//! compositor the way any program's does: it applies it at once and asks the person, through the
//! desktop's own question, whether to keep it, and goes back by itself when they do not.
//!
//! The wire client is blocking, so each call runs on a thread of its own with a connection of its
//! own, and only the real session is ever reached: a test's private bus is no compositor.

use super::brightness;
use super::link::DisplaysLink;
use super::map::{mode_of, monitors_of, transform_of};
use crate::deadline::{Deadline, peer};
use crate::live::BusSource;
use crate::ticks::{Ticker, Ticks};
use desktop_client::client::{DesktopClient, WaylandRoute};
use desktop_client::output::{
    ConfigOutcome, HeadChangeRequest, HeadConfig, HeadSettings, ModeChoice, OutputConfig,
    OutputEvent, ScaleMilli as WireScale,
};
use desktop_client::{
    Desktop, DesktopCommand, DesktopConfig, DesktopEvent, OutputName, Point, Protocol, Ticket,
};
use detent_model::{
    Brightness, BrightnessList, DisplayChange, DisplayEdit, DisplayMode, DisplaysStatus, Missing,
    OutputId, Work, mode_at,
};
use ds_peer::within;

const CANNOT: &str = "Displays can\u{2019}t be changed right now.";

fn connect() -> Result<DesktopClient, Missing> {
    DesktopClient::connect(WaylandRoute::Environment, DesktopConfig::default())
        .map_err(|_| Missing::NoService)
}

fn read_blocking() -> Result<DisplaysStatus, Missing> {
    let client = connect()?;
    let state = client.state();
    if !state.protocols.contains(Protocol::WlrOutputManagement) {
        return Err(Missing::NoService);
    }
    let displays = monitors_of(&state.outputs);
    if displays.is_empty() {
        return Err(Missing::NoDevice);
    }
    Ok(DisplaysStatus { displays })
}

/// Tells `ticker` each time the compositor finishes describing the displays again, until nobody
/// listens. It ends when the next change arrives after the page has gone.
fn watch_blocking(ticker: &Ticker) {
    let Ok(mut client) = connect() else { return };
    while client.blocking_dispatch().is_ok() {
        let described = client
            .drain_events()
            .iter()
            .any(|event| matches!(event, DesktopEvent::Output(OutputEvent::Done { .. })));
        if described && ticker.tick().is_err() {
            return;
        }
    }
}

/// The head called `name` and the settings that make `edit` of it; `None` when it is gone or has
/// no such mode.
fn settings_for(
    client: &DesktopClient,
    name: &OutputName,
    edit: &DisplayEdit,
) -> Option<(desktop_client::HeadKey, HeadSettings)> {
    let head = client.state().outputs.by_name(name)?;
    let settings = match edit {
        DisplayEdit::Scale(scale) => HeadSettings::keep().scaled(WireScale::new(scale.0)),
        DisplayEdit::Rotate(rotation) => HeadSettings::keep().transformed(transform_of(*rotation)),
        DisplayEdit::Move(at) => HeadSettings::keep().at(Point::new(at.x, at.y)),
        DisplayEdit::Refresh(rate) => {
            let size = head.current()?.size;
            let mode = head
                .modes
                .iter()
                .find(|mode| mode.size == size && mode.refresh.get() == rate.0)?;
            HeadSettings::keep().with_mode(ModeChoice::Existing(mode.key))
        }
        DisplayEdit::Size(size) => {
            let listed: Vec<DisplayMode> = head.modes.iter().map(mode_of).collect();
            let wanted = mode_at(&listed, head.current().map(mode_of), *size)?;
            let mode = head.modes.iter().find(|mode| mode_of(mode) == wanted)?;
            HeadSettings::keep().with_mode(ModeChoice::Existing(mode.key))
        }
    };
    Some((head.key, settings))
}

fn apply_blocking(change: &DisplayChange) -> Result<(), String> {
    let mut client = connect().map_err(|_| CANNOT.to_owned())?;
    let name = OutputName::new(change.output.0.clone());
    let (head, settings) = settings_for(&client, &name, &change.edit)
        .ok_or_else(|| "That display isn\u{2019}t set up that way.".to_owned())?;
    let config = OutputConfig::new(vec![HeadConfig::new(
        head,
        HeadChangeRequest::Enable(settings),
    )]);
    let ticket = client
        .send(DesktopCommand::ApplyOutputs(config))
        .map_err(|_| CANNOT.to_owned())?;
    let Ticket::Config(id) = ticket else {
        return Err(CANNOT.to_owned());
    };
    loop {
        for event in client.drain_events() {
            if let DesktopEvent::Output(OutputEvent::ConfigResolved {
                id: resolved,
                outcome,
                ..
            }) = event
                && resolved == id
            {
                return outcome_words(outcome);
            }
        }
        client.blocking_dispatch().map_err(|_| CANNOT.to_owned())?;
    }
}

fn outcome_words(outcome: ConfigOutcome) -> Result<(), String> {
    match outcome {
        ConfigOutcome::Succeeded => Ok(()),
        ConfigOutcome::Cancelled => {
            Err("The displays changed while this was being set. Try again.".to_owned())
        }
        _ => Err("This display can\u{2019}t be set that way.".to_owned()),
    }
}

/// Runs `work` on a thread of its own, under the read deadline.
async fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, Missing> + Send + 'static,
) -> Result<T, Missing> {
    let ran = within(Deadline::READ, peer::DISPLAYS, async {
        tokio::task::spawn_blocking(work).await
    })
    .await;
    match ran {
        Ok(Ok(done)) => done,
        Ok(Err(_)) | Err(_) => Err(Missing::Failed),
    }
}

/// The compositor of the person's session, when `bus` is the session.
#[derive(Debug)]
pub(crate) struct OnCompositor(pub(crate) BusSource);

impl OnCompositor {
    fn reachable(&self) -> Result<(), Missing> {
        match self.0 {
            BusSource::Session => Ok(()),
            BusSource::Address(..) | BusSource::Absent => Err(Missing::NoService),
        }
    }
}

impl DisplaysLink for OnCompositor {
    fn read(&self) -> Work<'_, Result<DisplaysStatus, Missing>> {
        match self.reachable() {
            Ok(()) => Box::pin(off_thread(read_blocking)),
            Err(missing) => Box::pin(std::future::ready(Err(missing))),
        }
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Missing>> {
        let reachable = self.reachable();
        Box::pin(async move {
            reachable?;
            let (ticker, ticks) = Ticks::channel();
            Ok(ticks.with_relay(tokio::task::spawn_blocking(move || watch_blocking(&ticker))))
        })
    }

    fn apply(&self, change: DisplayChange) -> Work<'_, Result<(), String>> {
        if self.reachable().is_err() {
            return Box::pin(std::future::ready(Err(CANNOT.to_owned())));
        }
        Box::pin(async move {
            match within(
                Deadline::ACTION,
                peer::DISPLAYS,
                tokio::task::spawn_blocking(move || apply_blocking(&change)),
            )
            .await
            {
                Ok(Ok(done)) => done,
                Ok(Err(_)) => Err(CANNOT.to_owned()),
                Err(silent) => Err(format!("{silent}.")),
            }
        })
    }

    fn brightness(&self) -> Work<'_, Result<BrightnessList, Missing>> {
        Box::pin(brightness::read(&self.0))
    }

    fn brightness_changes(&self) -> Work<'_, Result<Ticks, Missing>> {
        Box::pin(brightness::changes(&self.0))
    }

    fn set_brightness(&self, output: OutputId, level: Brightness) -> Work<'_, Result<(), String>> {
        Box::pin(async move { brightness::set(&self.0, &output, level).await })
    }
}
