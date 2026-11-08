use amm_core::structs::event::{KeybindPhase, ModEvent};

use crate::services::PingLink;

// whether the override key is down, from the manager's keybind events
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OverrideKey {
    held: bool,
}

impl OverrideKey {
    pub fn apply(&mut self, event: &ModEvent) {
        match event {
            ModEvent::Keybind { action, phase } if *action == PingLink::OVERRIDE_ACTION => match phase {
                KeybindPhase::Pressed => self.held = true,
                KeybindPhase::Released => self.held = false,
                KeybindPhase::Repeat => {}
            },
            // a disabled mod gets no release, so the hold ends here
            ModEvent::Disabled => self.held = false,
            _ => {}
        }
    }

    pub fn held(&self) -> bool {
        self.held
    }
}
