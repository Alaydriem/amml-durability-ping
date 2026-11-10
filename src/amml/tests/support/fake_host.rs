use std::cell::RefCell;
use std::collections::BTreeMap;

use alaydriem_durability_ping::traits::GameHost;
use alaydriem_durability_ping::types::HookSpec;
use amml_sdk::{HookHandle, SdkError};

// a loader stand-in: entries it knows resolve, the rest are missing; hooks are recorded, never installed
pub struct FakeHost {
    entries: BTreeMap<&'static str, u64>,
    hooked: RefCell<Vec<&'static str>>,
    targets: RefCell<Vec<u64>>,
    warnings: RefCell<Vec<String>>,
    infos: RefCell<Vec<String>>,
    refused: Option<&'static str>,
}

impl FakeHost {
    pub fn new(entries: &[(&'static str, u64)]) -> Self {
        Self {
            entries: entries.iter().copied().collect(),
            hooked: RefCell::new(Vec::new()),
            targets: RefCell::new(Vec::new()),
            warnings: RefCell::new(Vec::new()),
            infos: RefCell::new(Vec::new()),
            refused: None,
        }
    }

    // a host whose hook call refuses every hook on `entry`
    pub fn refusing(entries: &[(&'static str, u64)], entry: &'static str) -> Self {
        Self { refused: Some(entry), ..Self::new(entries) }
    }

    pub fn hooked(&self) -> Vec<&'static str> {
        self.hooked.borrow().clone()
    }

    // the target of each hook, in install order
    pub fn targets(&self) -> Vec<u64> {
        self.targets.borrow().clone()
    }

    // what the fake reads from `slot` of the table at `table`
    pub fn slot_target(table: u64, slot: u64) -> u64 {
        table * 0x100 + slot
    }

    pub fn warnings(&self) -> Vec<String> {
        self.warnings.borrow().clone()
    }

    pub fn infos(&self) -> Vec<String> {
        self.infos.borrow().clone()
    }
}

impl GameHost for FakeHost {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.entries.get(id).copied().ok_or_else(|| SdkError::MissingEntry(id.to_string()))
    }

    fn table_slot(&self, table: u64, slot: u64) -> Result<u64, SdkError> {
        Ok(Self::slot_target(table, slot))
    }

    fn hook(&self, target: u64, spec: &HookSpec) -> Result<HookHandle, SdkError> {
        if self.refused == Some(spec.entry) {
            return Err(SdkError::Mod(format!("{} refused", spec.entry)));
        }
        self.targets.borrow_mut().push(target);
        let mut hooked = self.hooked.borrow_mut();
        hooked.push(spec.entry);
        Ok(HookHandle(hooked.len() as u64))
    }

    fn unhook(&self, _hook: HookHandle) -> Result<(), SdkError> {
        Ok(())
    }

    fn warn(&self, message: &str) {
        self.warnings.borrow_mut().push(message.to_string());
    }

    fn info(&self, message: &str) {
        self.infos.borrow_mut().push(message.to_string());
    }
}
