use amml_sdk::{HookHandle, SdkError};

use crate::types::HookSpec;

// what installing the mod needs from the loader: entry values, vtable slots, hooks and log lines
pub trait GameHost {
    fn resolve(&self, id: &str) -> Result<u64, SdkError>;

    // the code pointer in `slot` of the vtable at `table`, a resolved vtable entry of this game build
    fn table_slot(&self, table: u64, slot: u64) -> Result<u64, SdkError>;

    // installs `spec.detour` on `target`, the resolved value of `spec.entry`
    fn hook(&self, target: u64, spec: &HookSpec) -> Result<HookHandle, SdkError>;

    fn unhook(&self, hook: HookHandle) -> Result<(), SdkError>;

    fn warn(&self, message: &str);

    fn info(&self, message: &str);
}
