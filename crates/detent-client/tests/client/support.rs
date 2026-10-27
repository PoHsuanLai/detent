//! What every flow starts from: a client builder over no bus, with its files in a folder nothing
//! reads or writes.

use detent_client::{BusSource, ClientBuilder, ClientHomes};
use detent_files::{CacheHome, ConfigHome, DataHome};
use std::path::PathBuf;

/// A builder whose every service is the real one until a test replaces it; the bus is absent, so
/// nothing it holds is ever reached.
pub fn builder() -> ClientBuilder {
    let nowhere = PathBuf::from("/nonexistent/detent-client-tests");
    ClientBuilder::over(
        BusSource::Absent,
        ClientHomes {
            config: ConfigHome(nowhere.join("config")),
            data: DataHome(nowhere.join("data")),
            cache: CacheHome(nowhere.join("cache")),
        },
    )
}
