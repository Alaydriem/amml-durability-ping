use amml_sdk::{HookHandle, SdkError};
use curia::Level;

use crate::services::PingRuntime;
use crate::traits::GameHost;
use crate::types::HookSpec;

// the host of enable, which also keeps each log line in the runtime: no manager link exists at enable, so the
// runtime sends the kept lines through curia to the mod manager's log of this mod once the link attaches
pub struct NotingHost<'a, H: GameHost> {
    host: &'a H,
    runtime: &'a PingRuntime,
}

impl<'a, H: GameHost> NotingHost<'a, H> {
    pub fn new(host: &'a H, runtime: &'a PingRuntime) -> Self {
        Self { host, runtime }
    }
}

impl<H: GameHost> GameHost for NotingHost<'_, H> {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.host.resolve(id)
    }

    fn table_slot(&self, table: u64, slot: u64) -> Result<u64, SdkError> {
        self.host.table_slot(table, slot)
    }

    fn hook(&self, target: u64, spec: &HookSpec) -> Result<HookHandle, SdkError> {
        self.host.hook(target, spec)
    }

    fn unhook(&self, hook: HookHandle) -> Result<(), SdkError> {
        self.host.unhook(hook)
    }

    fn warn(&self, message: &str) {
        self.host.warn(message);
        self.runtime.note_startup(Level::Warn, message);
    }

    fn info(&self, message: &str) {
        self.host.info(message);
        self.runtime.note_startup(Level::Info, message);
    }
}
