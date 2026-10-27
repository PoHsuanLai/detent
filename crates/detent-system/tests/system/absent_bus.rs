//! A real link built over no bus answers with why there is nothing, and reaches nothing: the
//! window started without a bus shows each pane as unavailable.

use detent_model::{Absence, SpaceKey};
use detent_system::bluetooth::{BluetoothLink, OnBluetooth};
use detent_system::bus::BusSource;
use detent_system::computers::{ComputersLink, OnBus as OnComputers};
use detent_system::running::{OnBus as OnRunning, RunningLink};
use detent_system::screen_sharing::{OnBus as OnPortal, ScreenSharingLink};
use detent_system::spaces::{OnBus as OnSpaces, SpacesLink};
use detent_system::updates::{OnBus as OnUpdates, UpdatesLink};
use detent_system::wifi::{OnWifi, WifiLink};

#[tokio::test]
async fn the_system_bus_pages_find_no_service_without_a_bus() {
    assert_eq!(
        OnWifi(BusSource::Absent).read().await.err(),
        Some(Absence::NotRunning)
    );
    assert_eq!(
        OnBluetooth::new(BusSource::Absent).read().await.err(),
        Some(Absence::NotRunning)
    );
}

#[tokio::test]
async fn the_daemon_panes_find_no_bus() {
    let bus = || BusSource::Absent;
    assert_eq!(OnUpdates(bus()).read().await.err(), Some(Absence::NoBus));
    assert_eq!(OnComputers(bus()).read().await.err(), Some(Absence::NoBus));
    assert_eq!(OnRunning(bus()).read().await.err(), Some(Absence::NoBus));
    assert_eq!(OnPortal(bus()).list().await.err(), Some(Absence::NoBus));
    assert_eq!(OnSpaces(bus()).list().await.err(), Some(Absence::NoBus));
}

#[tokio::test]
async fn a_removal_without_a_bus_says_it_was_not_made() {
    let removal = almanac_core::Removal::KEEP_ALL;
    let ended = OnSpaces(BusSource::Absent)
        .remove(SpaceKey("work".to_owned()), removal)
        .await;
    assert!(ended.is_err());
}
