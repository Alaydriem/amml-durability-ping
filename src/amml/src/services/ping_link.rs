use amm_client::link::{LinkState, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use amml_api::Ctx;
use durability_ping_core::config::PingConfig;

use crate::services::{PingDescriptor, RawSettings, SettingsReader};

// the registration with the mod manager and the settings read from it; `events` is cleared at the start of every
// drain, the link itself keeps a bounded number of unread events
pub struct PingLink {
    link: ManagerLink,
    events: Vec<ModEvent>,
}

impl PingLink {
    // the index of the override keybind in the descriptor; it is the only keybind
    pub const OVERRIDE_ACTION: u16 = 0;

    // the link waits for the manager's tables; `state` says when it is connected. The mod needs only its keybind
    // events, so it does not watch keys and never takes an input lease
    pub fn start(ctx: &Ctx) -> Result<Self, LinkError> {
        let link = ManagerLink::start_without_frame(ctx, PingDescriptor::TEXT)?;
        Ok(Self { link, events: Vec::new() })
    }

    pub fn state(&self) -> LinkState {
        self.link.state()
    }

    // reads every queued manager event
    pub fn drain(&mut self) -> &[ModEvent] {
        self.events.clear();
        self.events.extend(self.link.events());
        &self.events
    }

    // the settings of the manager; the defaults, with a warning, when they cannot be read
    pub fn settings(&self) -> PingConfig {
        match self.link.config::<RawSettings>() {
            Ok(raw) => SettingsReader::convert(&raw),
            Err(error) => {
                curia::warn!("config could not be read; using the defaults", { error: error.to_string() });
                PingConfig::default()
            }
        }
    }
}
