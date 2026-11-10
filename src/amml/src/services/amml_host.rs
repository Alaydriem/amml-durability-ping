use amml_sdk::amml_abi::AmmlLogLevel;
use amml_sdk::{Api, HookHandle, SdkError};

use crate::traits::GameHost;
use crate::types::HookSpec;

// the AMML loader as the host the mod installs into
pub struct AmmlHost<'a> {
    api: &'a Api,
    module: &'static str,
}

impl<'a> AmmlHost<'a> {
    // no game vtable the mod reads is this long; a larger slot value is a bad entry, not a slot
    const MAX_SLOT: u64 = 512;

    pub fn new(api: &'a Api, module: &'static str) -> Self {
        Self { api, module }
    }
}

impl GameHost for AmmlHost<'_> {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.api.resolve(id)
    }

    fn table_slot(&self, table: u64, slot: u64) -> Result<u64, SdkError> {
        if table == 0 || slot >= Self::MAX_SLOT {
            return Err(SdkError::Mod(format!("slot {slot} of the table {table:#x} is out of range")));
        }
        // SAFETY: `table` is a vtable entry of this game build, derived and checked to lie in .rdata, and `slot` is
        // a slot entry of that class (below MAX_SLOT); the table is static and lives as long as the game
        #[allow(unsafe_code)]
        let target = unsafe { std::ptr::read((table + slot * 8) as *const u64) };
        if target == 0 {
            return Err(SdkError::Mod(format!("slot {slot} of the table {table:#x} is empty")));
        }
        Ok(target)
    }

    fn hook(&self, target: u64, spec: &HookSpec) -> Result<HookHandle, SdkError> {
        // SAFETY: `target` is the derived start of the game function `spec.entry` names (for a slot entry, the code
        // pointer in that slot of the `spec.table` vtable), and `spec.detour` is the
        // amml_detour! written for that function's signature (this in rcx, then the C++ parameters as the game
        // passes them); PingMod::hooks pairs each entry with its detour. Each original slot belongs to one hook
        // only.
        #[allow(unsafe_code)]
        unsafe {
            self.api.hook(target, spec.detour, 0, spec.original)
        }
    }

    fn unhook(&self, hook: HookHandle) -> Result<(), SdkError> {
        self.api.unhook(hook)
    }

    fn warn(&self, message: &str) {
        self.api.log(self.module, AmmlLogLevel::WARN, message);
    }

    fn info(&self, message: &str) {
        self.api.log(self.module, AmmlLogLevel::INFO, message);
    }
}
