use std::path::Path;

use amm_client::link::{ApiTable, EventDrain, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use durability_ping_core::config::PingConfig;

use crate::services::{PingDescriptor, RawSettings, SettingsReader};

// the registration with the mod manager and the settings read from it; `events` holds at most
// MAX_EVENTS_PER_TICK and is cleared at the start of every drain
pub struct PingLink {
    link: ManagerLink,
    config: PingConfig,
    events: Vec<ModEvent>,
}

impl PingLink {
    pub const MAX_DRAINS_PER_TICK: usize = 4;
    pub const MAX_EVENTS_PER_TICK: usize = Self::MAX_DRAINS_PER_TICK * EventDrain::CAPACITY;
    // the index of the override keybind in the descriptor; it is the only keybind
    pub const OVERRIDE_ACTION: u16 = 0;

    // the mod needs only its keybind events, so it does not watch keys
    pub fn new(table: ApiTable, config_dir: &Path) -> Result<Self, LinkError> {
        let link = ManagerLink::without_frame(table, PingDescriptor::TEXT, config_dir)?;
        Ok(Self::from_link(link))
    }

    pub fn from_link(link: ManagerLink) -> Self {
        let mut this =
            Self { config: PingConfig::default(), link, events: Vec::with_capacity(Self::MAX_EVENTS_PER_TICK) };
        this.config = this.read_settings();
        this
    }

    pub fn config(&self) -> &PingConfig {
        &self.config
    }

    pub fn reload(&mut self) -> &PingConfig {
        self.config = self.read_settings();
        &self.config
    }

    // reads the queued manager events, at most MAX_EVENTS_PER_TICK; the rest wait for the next tick
    pub fn drain(&mut self) -> Vec<ModEvent> {
        self.events.clear();
        for _ in 0..Self::MAX_DRAINS_PER_TICK {
            let before = self.events.len();
            self.events.extend(self.link.events());
            if self.events.len() - before < EventDrain::CAPACITY {
                break;
            }
        }
        self.events.clone()
    }

    fn read_settings(&self) -> PingConfig {
        match self.link.config::<RawSettings>() {
            Ok(raw) => SettingsReader::convert(&raw),
            Err(error) => {
                curia::warn!("config could not be read; using the defaults", { error: error.to_string() });
                PingConfig::default()
            }
        }
    }
}
