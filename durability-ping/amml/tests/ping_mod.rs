use std::path::Path;

use amm_abi::types::AmmLogLevel;
use amm_client::test_support::FakeTable;
use amml_durability_ping::PingMod;
use amml_durability_ping::services::PingRuntime;
use amml_game::EntryId;
use amml_sdk::{AmmlMod, SdkError};

use crate::support::FakeOutput;
use crate::support::fake_host::FakeHost;

const ALERT_ENTRIES: [(&str, u64); 7] = [
    (PingMod::GET_LOCAL_PLAYER, 150),
    (PingMod::PLAYER_INVENTORY, 0x700),
    (PingMod::ITEMS, 0x10),
    (PingMod::GET_DAMAGE_VALUE, 12),
    (PingMod::GET_MAX_DAMAGE, 13),
    (PingMod::ITEM_FULL_NAME, 0x120),
    (PingMod::IS_SHOWING_MENU, 0x94),
];

// the two tables differ, so the base and the survival attack slots are two targets
const GUARD_ENTRIES: [(&str, u64); 8] = [
    (PingMod::GAME_MODE_VTABLE, 0xA000),
    (PingMod::GAME_MODE_M_PLAYER, 8),
    (PingMod::START_DESTROY_BLOCK, 1),
    (PingMod::CONTINUE_DESTROY_BLOCK, 3),
    (PingMod::USE_ITEM, 11),
    (PingMod::ATTACK, 15),
    (PingMod::SURVIVAL_MODE_VTABLE, 0xB000),
    (PingMod::ITEM_USE_ON, 0x2000),
];

const EVERY_HOOK: [&str; 7] = [
    PingMod::UPDATE,
    PingMod::START_DESTROY_BLOCK,
    PingMod::CONTINUE_DESTROY_BLOCK,
    PingMod::USE_ITEM,
    PingMod::ATTACK,
    PingMod::ATTACK,
    PingMod::ITEM_USE_ON,
];

fn runtime() -> PingRuntime {
    PingRuntime::with_output(Path::new(FakeTable::CONFIG_DIR), FakeOutput::default())
}

fn every_entry() -> Vec<(&'static str, u64)> {
    let mut entries = vec![(PingMod::UPDATE, 0x1000)];
    entries.extend(ALERT_ENTRIES);
    entries.extend(GUARD_ENTRIES);
    entries
}

fn without(name: &str) -> Vec<(&'static str, u64)> {
    every_entry().into_iter().filter(|(entry, _)| *entry != name).collect()
}

#[test]
fn the_module_string_is_the_namespaced_id() {
    assert_eq!(PingMod::MODULE, format!("@{}/{}", PingMod::NAMESPACE, PingMod::NAME));
}

#[test]
fn the_guard_entry_names_are_the_sdk_ids() {
    assert_eq!(PingMod::GAME_MODE_VTABLE, EntryId::GAME_MODE_VTABLE.as_str());
    assert_eq!(PingMod::GAME_MODE_M_PLAYER, EntryId::GAME_MODE_M_PLAYER.as_str());
    assert_eq!(PingMod::START_DESTROY_BLOCK, EntryId::GAME_MODE_START_DESTROY_BLOCK.as_str());
    assert_eq!(PingMod::CONTINUE_DESTROY_BLOCK, EntryId::GAME_MODE_CONTINUE_DESTROY_BLOCK.as_str());
    assert_eq!(PingMod::USE_ITEM, EntryId::GAME_MODE_USE_ITEM.as_str());
    assert_eq!(PingMod::ATTACK, EntryId::GAME_MODE_ATTACK.as_str());
    assert_eq!(PingMod::SURVIVAL_MODE_VTABLE, EntryId::SURVIVAL_MODE_VTABLE.as_str());
    assert_eq!(PingMod::ITEM_USE_ON, EntryId::ITEM_USE_ON.as_str());
}

#[test]
fn every_entry_installs_every_hook_and_logs_no_feature_off() {
    let host = FakeHost::new(&every_entry());
    let runtime = runtime();

    PingMod::enable_with(&host, &runtime).unwrap();

    assert_eq!(host.hooked(), EVERY_HOOK);
    assert!(host.warnings().is_empty(), "{:?}", host.warnings());
    assert_eq!(host.infos(), ["features off: none"]);
    assert_eq!(runtime.game_mode_player_offset(), 8);
}

#[test]
fn slot_hooks_go_on_the_slot_of_their_table() {
    let host = FakeHost::new(&every_entry());

    PingMod::enable_with(&host, &runtime()).unwrap();

    let targets = host.targets();
    assert_eq!(targets[1], FakeHost::slot_target(0xA000, 1));
    assert_eq!(targets[2], FakeHost::slot_target(0xA000, 3));
    assert_eq!(targets[3], FakeHost::slot_target(0xA000, 11));
    assert_eq!(targets[4], FakeHost::slot_target(0xB000, 15));
    assert_eq!(targets[5], FakeHost::slot_target(0xA000, 15));
    assert_eq!(targets[6], 0x2000);
}

#[test]
fn a_missing_optional_entry_turns_off_only_the_alert_with_one_warning() {
    let mut entries = vec![(PingMod::UPDATE, 0x1000), (PingMod::IS_SHOWING_MENU, 0x94)];
    entries.extend(GUARD_ENTRIES);
    let host = FakeHost::new(&entries);

    PingMod::enable_with(&host, &runtime()).unwrap();

    assert_eq!(host.hooked(), EVERY_HOOK);
    let warnings = host.warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("the durability alert and the tool guard is off"));
    assert!(host.infos().is_empty());
}

// without isShowingMenu the alert still works, only without the wait for the world to show
#[test]
fn a_missing_menu_entry_turns_off_only_the_menu_wait() {
    let host = FakeHost::new(&without(PingMod::IS_SHOWING_MENU));

    PingMod::enable_with(&host, &runtime()).unwrap();

    assert_eq!(host.hooked(), EVERY_HOOK);
    let warnings = host.warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("the alert's wait for the world to show is off"));
}

#[test]
fn missing_block_entry_turns_off_only_that_hook() {
    let host = FakeHost::new(&without(PingMod::ATTACK));

    PingMod::enable_with(&host, &runtime()).unwrap();

    let warnings = host.warnings();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains(PingMod::ATTACK));
    assert!(warnings[0].contains("the tool guard (attack) is off"));
    assert_eq!(
        host.hooked(),
        [
            PingMod::UPDATE,
            PingMod::START_DESTROY_BLOCK,
            PingMod::CONTINUE_DESTROY_BLOCK,
            PingMod::USE_ITEM,
            PingMod::ITEM_USE_ON,
        ]
    );
    assert!(host.infos().is_empty());
}

#[test]
fn a_missing_survival_table_leaves_the_base_attack_hook() {
    let host = FakeHost::new(&without(PingMod::SURVIVAL_MODE_VTABLE));

    PingMod::enable_with(&host, &runtime()).unwrap();

    let warnings = host.warnings();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains(PingMod::SURVIVAL_MODE_VTABLE));
    assert_eq!(host.hooked().iter().filter(|entry| **entry == PingMod::ATTACK).count(), 1);
    assert_eq!(host.targets()[4], FakeHost::slot_target(0xA000, 15));
}

#[test]
fn a_missing_game_mode_table_leaves_the_update_and_use_on_hooks() {
    let host = FakeHost::new(&without(PingMod::GAME_MODE_VTABLE));

    PingMod::enable_with(&host, &runtime()).unwrap();

    let hooked = host.hooked();
    assert!(hooked.contains(&PingMod::UPDATE));
    assert!(hooked.contains(&PingMod::ITEM_USE_ON));
    // the survival attack slot does not need the base table
    assert_eq!(hooked.len(), 3, "{hooked:?}");
    assert!(host.warnings().iter().all(|warning| warning.contains(PingMod::GAME_MODE_VTABLE)));
}

#[test]
fn a_missing_player_offset_leaves_out_the_game_mode_hooks() {
    let host = FakeHost::new(&without(PingMod::GAME_MODE_M_PLAYER));
    let runtime = runtime();

    PingMod::enable_with(&host, &runtime).unwrap();

    assert_eq!(host.hooked(), [PingMod::UPDATE, PingMod::ITEM_USE_ON]);
    let warnings = host.warnings();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains(PingMod::GAME_MODE_M_PLAYER));
    assert_eq!(runtime.game_mode_player_offset(), 0);
}

#[test]
fn two_slots_on_one_address_are_hooked_once() {
    let mut entries = without(PingMod::SURVIVAL_MODE_VTABLE);
    entries.push((PingMod::SURVIVAL_MODE_VTABLE, 0xA000));
    let host = FakeHost::new(&entries);

    PingMod::enable_with(&host, &runtime()).unwrap();

    assert_eq!(host.hooked().iter().filter(|entry| **entry == PingMod::ATTACK).count(), 1);
    assert!(host.warnings().is_empty());
    assert_eq!(host.infos(), ["features off: none"]);
}

#[test]
fn every_hook_fits_the_runtime() {
    assert!(PingMod::hooks().len() <= PingRuntime::HOOK_CAPACITY);
}

#[test]
fn a_missing_update_entry_fails_enable_and_installs_nothing() {
    let host = FakeHost::new(&without(PingMod::UPDATE));

    let result = PingMod::enable_with(&host, &runtime());

    assert!(matches!(result, Err(SdkError::MissingEntry(id)) if id == PingMod::UPDATE));
    assert!(host.hooked().is_empty());
}

// two hooks on one original slot would overwrite each other's trampoline
#[test]
fn every_hook_has_its_own_original_slot() {
    let hooks = PingMod::hooks();
    for (index, hook) in hooks.iter().enumerate() {
        for other in &hooks[index + 1..] {
            assert!(!std::ptr::eq(hook.original, other.original), "{} shares an original slot", hook.entry);
        }
    }
}

#[test]
fn a_refused_guard_hook_turns_off_only_its_feature() {
    let host = FakeHost::refusing(&every_entry(), PingMod::START_DESTROY_BLOCK);
    let runtime = runtime();

    PingMod::enable_with(&host, &runtime).unwrap();

    let warnings = host.warnings();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("the tool guard (startDestroyBlock) is off"));
    let hooked = host.hooked();
    assert_eq!(hooked.len(), 6, "{hooked:?}");
    assert!(hooked.contains(&PingMod::UPDATE));
    assert!(!hooked.contains(&PingMod::START_DESTROY_BLOCK));
    assert_eq!(runtime.take_hooks().len(), 6);
    assert!(host.infos().is_empty());
}

#[test]
fn a_refused_update_hook_fails_enable_and_leaves_no_hook() {
    let host = FakeHost::refusing(&every_entry(), PingMod::UPDATE);
    let runtime = runtime();

    assert!(PingMod::enable_with(&host, &runtime).is_err());
    assert!(runtime.take_hooks().is_empty());
}

// the SDK's ItemStackBaseRef::type_name reads Item::mFullName; without the entry every reading is None and both
// features stay silent (found in game, 2026-11-08)
#[test]
fn the_held_tool_read_wants_the_item_name() {
    assert!(PingMod::wanted().iter().any(|(id, _)| *id == EntryId::ITEM_FULL_NAME));
}

// the loader resolves only the entries amml.hcl lists, so each wanted entry and each hook entry must be in it
#[test]
fn the_manifest_lists_every_wanted_and_hook_entry() {
    let manifest = include_str!("../amml.hcl");
    let mut names: Vec<&str> = PingMod::wanted().iter().map(|(id, _)| id.as_str()).collect();
    for spec in PingMod::hooks() {
        names.push(spec.entry);
        names.extend(spec.table);
    }
    for name in names {
        assert!(manifest.contains(&format!("\"{name}\"")), "{name} is not in amml.hcl");
    }
}

// the lines of enable go to the loader log at once (the manager may be missing), and to the mod manager's log of
// this mod through curia once the link attaches: no link exists at enable, so they are kept until then
fn manager_log() -> Vec<(u8, String)> {
    FakeTable::with(|state| state.logs.iter().map(|call| (call.level, call.message.clone())).collect())
}

#[test]
fn enable_lines_reach_the_manager_log_when_the_link_attaches() {
    let host = FakeHost::new(&every_entry());
    let runtime = runtime();

    PingMod::enable_with(&host, &runtime).unwrap();
    assert!(manager_log().iter().all(|(_, message)| message != "features off: none"));
    runtime.attach_with(Ok(FakeTable::table()));

    assert!(
        manager_log().contains(&(AmmLogLevel::Info as u8, "features off: none".to_string())),
        "{:?}",
        manager_log()
    );
}

#[test]
fn a_feature_warning_reaches_the_manager_log_as_a_warning() {
    let host = FakeHost::new(&without(PingMod::IS_SHOWING_MENU));
    let runtime = runtime();

    PingMod::enable_with(&host, &runtime).unwrap();
    runtime.attach_with(Ok(FakeTable::table()));

    let log = manager_log();
    assert!(
        log.iter().any(|(level, message)| *level == AmmLogLevel::Warn as u8
            && message.contains("the alert's wait for the world to show is off")),
        "{log:?}"
    );
}

#[test]
fn a_hook_installer_warning_reaches_the_manager_log() {
    let host = FakeHost::new(&without(PingMod::ATTACK));
    let runtime = runtime();

    PingMod::enable_with(&host, &runtime).unwrap();
    runtime.attach_with(Ok(FakeTable::table()));

    let log = manager_log();
    assert!(
        log.iter()
            .any(|(level, message)| *level == AmmLogLevel::Warn as u8
                && message.contains("GameMode.attack did not resolve")),
        "{log:?}"
    );
}
