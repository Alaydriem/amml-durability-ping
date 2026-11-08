use std::ffi::c_void;

use amml_game::{EntryId, EntryNeed, EntryResolver, GameEntries, GameError};
use amml_sdk::{AmmlMod, Api, Logger, ModDirs, SdkError};

use crate::detours::{
    attack, client_update, continue_destroy_block, item_use_on, start_destroy_block, survival_attack, use_item,
};
use crate::services::{AmmlHost, HookInstaller, NotingHost, PingRuntime};
use crate::traits::GameHost;
use crate::types::HookSpec;

// the loader a game host stands in for, as the resolver the game entries are read with
struct HostResolver<'a, H: GameHost>(&'a H);

impl<H: GameHost> EntryResolver for HostResolver<'_, H> {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.0.resolve(id)
    }
}

// the AMML entry point: load makes the runtime, enable installs the hook, disable removes it and leaves the
// mod manager. The manager link is made on the first ClientInstance::update after enable
pub struct PingMod;

impl PingMod {
    // the id as the module string the loader and the hook host take; a test checks it against the namespace and name
    pub const MODULE: &'static str = "@amml/durability-ping";
    pub const UPDATE: &'static str = "ClientInstance.update";
    pub const GET_LOCAL_PLAYER: &'static str = "ClientInstance.getLocalPlayer";
    pub const PLAYER_INVENTORY: &'static str = "Player.playerInventory";
    pub const ITEMS: &'static str = "FillingContainer.mItems";
    pub const GET_DAMAGE_VALUE: &'static str = "ItemStackBase.getDamageValue";
    pub const GET_MAX_DAMAGE: &'static str = "Item.getMaxDamage";
    pub const ITEM_FULL_NAME: &'static str = EntryId::ITEM_FULL_NAME.as_str();
    pub const IS_SHOWING_MENU: &'static str = EntryId::CLIENT_INSTANCE_IS_SHOWING_MENU.as_str();

    pub const GAME_MODE_VTABLE: &'static str = EntryId::GAME_MODE_VTABLE.as_str();
    pub const GAME_MODE_M_PLAYER: &'static str = EntryId::GAME_MODE_M_PLAYER.as_str();
    pub const START_DESTROY_BLOCK: &'static str = EntryId::GAME_MODE_START_DESTROY_BLOCK.as_str();
    pub const CONTINUE_DESTROY_BLOCK: &'static str = EntryId::GAME_MODE_CONTINUE_DESTROY_BLOCK.as_str();
    pub const USE_ITEM: &'static str = EntryId::GAME_MODE_USE_ITEM.as_str();
    pub const ATTACK: &'static str = EntryId::GAME_MODE_ATTACK.as_str();
    pub const SURVIVAL_MODE_VTABLE: &'static str = EntryId::SURVIVAL_MODE_VTABLE.as_str();
    pub const ITEM_USE_ON: &'static str = EntryId::ITEM_USE_ON.as_str();

    // the update hook alone drives the alert
    const ALERT_FEATURE: &'static str = "the durability alert";
    // the held tool reads feed both the alert and the tool guard
    const HELD_TOOL_FEATURE: &'static str = "the durability alert and the tool guard";
    // without isShowingMenu the alert can play on the loading screen, before the world shows
    const MENU_WAIT_FEATURE: &'static str = "the alert's wait for the world to show";
    // the GameMode hooks find their player through GameMode::mPlayer; without it they are left out
    const GAME_MODE_GUARD_FEATURE: &'static str = "the tool guard (GameMode hooks)";

    // every hook, in install order; the client update drives the whole mod. The guard hooks are optional. The
    // survival attack slot has its own body that skips the base slot, so attack is hooked on both tables; the other
    // survival slots reach the base bodies (Task 8 slots.md)
    pub fn hooks() -> [HookSpec; 7] {
        [
            HookSpec {
                entry: Self::UPDATE,
                table: None,
                detour: client_update::detour as *const c_void,
                original: client_update::GUARD.original(),
                required: true,
                feature: Self::ALERT_FEATURE,
            },
            HookSpec {
                entry: Self::START_DESTROY_BLOCK,
                table: Some(Self::GAME_MODE_VTABLE),
                detour: start_destroy_block::detour as *const c_void,
                original: start_destroy_block::GUARD.original(),
                required: false,
                feature: "the tool guard (startDestroyBlock)",
            },
            HookSpec {
                entry: Self::CONTINUE_DESTROY_BLOCK,
                table: Some(Self::GAME_MODE_VTABLE),
                detour: continue_destroy_block::detour as *const c_void,
                original: continue_destroy_block::GUARD.original(),
                required: false,
                feature: "the tool guard (continueDestroyBlock)",
            },
            HookSpec {
                entry: Self::USE_ITEM,
                table: Some(Self::GAME_MODE_VTABLE),
                detour: use_item::detour as *const c_void,
                original: use_item::GUARD.original(),
                required: false,
                feature: "the tool guard (useItem)",
            },
            HookSpec {
                entry: Self::ATTACK,
                table: Some(Self::SURVIVAL_MODE_VTABLE),
                detour: survival_attack::detour as *const c_void,
                original: survival_attack::GUARD.original(),
                required: false,
                feature: "the tool guard (attack)",
            },
            HookSpec {
                entry: Self::ATTACK,
                table: Some(Self::GAME_MODE_VTABLE),
                detour: attack::detour as *const c_void,
                original: attack::GUARD.original(),
                required: false,
                feature: "the tool guard (attack)",
            },
            HookSpec {
                entry: Self::ITEM_USE_ON,
                table: None,
                detour: item_use_on::detour as *const c_void,
                original: item_use_on::GUARD.original(),
                required: false,
                feature: "the tool guard (useOn)",
            },
        ]
    }

    // the game reads of the held tool; all are optional, and a missing one turns off the alert and the tool guard
    // (the guard decides from the same local player and held tool reads)
    pub fn wanted() -> [(EntryId, EntryNeed); 8] {
        let held = EntryNeed::Optional { feature: Self::HELD_TOOL_FEATURE };
        let guard = EntryNeed::Optional { feature: Self::GAME_MODE_GUARD_FEATURE };
        [
            (EntryId::CLIENT_INSTANCE_GET_LOCAL_PLAYER, held),
            (EntryId::PLAYER_PLAYER_INVENTORY, held),
            (EntryId::FILLING_CONTAINER_ITEMS, held),
            (EntryId::ITEM_STACK_BASE_GET_DAMAGE_VALUE, held),
            (EntryId::ITEM_GET_MAX_DAMAGE, held),
            // ItemStackBaseRef::type_name reads Item::mFullName; a reading needs the name
            (EntryId::ITEM_FULL_NAME, held),
            // the alert waits while a menu shows (proven in game: the loading screen reads true, play reads false)
            (EntryId::CLIENT_INSTANCE_IS_SHOWING_MENU, EntryNeed::Optional { feature: Self::MENU_WAIT_FEATURE }),
            (EntryId::GAME_MODE_M_PLAYER, guard),
        ]
    }

    // the body of enable: the required hook entries must resolve, then the hook goes in and the next tick reaches
    // the manager. One line says which features are off, or that none is
    pub fn enable_with<H: GameHost>(host: &H, runtime: &PingRuntime) -> Result<(), SdkError> {
        runtime.clear_startup();
        let host = &NotingHost::new(host, runtime);
        let entries = GameEntries::resolve(&HostResolver(host), &Self::wanted()).map_err(|error| match error {
            GameError::MissingEntry { id } => SdkError::MissingEntry(id),
            other => SdkError::Mod(other.to_string()),
        })?;
        let player_offset = entries.get(EntryId::GAME_MODE_M_PLAYER).and_then(|offset| usize::try_from(offset).ok());
        runtime.set_game_mode_player_offset(player_offset.unwrap_or(0));
        let hooks: Vec<HookSpec> =
            Self::hooks().into_iter().filter(|spec| player_offset.is_some() || spec.table.is_none()).collect();
        let hooks_off = HookInstaller::install(host, runtime, &hooks)?;
        let off = entries.off_features();
        if off.is_empty() && hooks_off.is_empty() {
            host.info("features off: none");
        }
        for (feature, cause) in off {
            host.warn(&format!("{feature} is off ({cause})"));
        }
        runtime.client().set_entries(entries);
        runtime.enable_link();
        Ok(())
    }
}

impl AmmlMod for PingMod {
    const NAMESPACE: &'static str = "amml";
    const NAME: &'static str = "durability-ping";

    // the audio thread does not start here: load runs under the loader lock
    fn load(api: &Api) -> Result<(), SdkError> {
        let dirs = ModDirs::new(api, Self::MODULE)?;
        PingRuntime::install(&dirs.config_dir);
        Ok(())
    }

    fn enable(api: &Api) -> Result<(), SdkError> {
        let runtime = PingRuntime::get().ok_or_else(|| SdkError::Mod("load did not run".to_string()))?;
        Self::enable_with(&AmmlHost::new(api, Self::MODULE), runtime)?;
        Logger::new(api, Self::MODULE).info("@amml/durability-ping enabled");
        runtime.note_startup(curia::Level::Info, "@amml/durability-ping enabled");
        Ok(())
    }

    fn disable(api: &Api) {
        let Some(runtime) = PingRuntime::get() else { return };
        HookInstaller::remove_all(&AmmlHost::new(api, Self::MODULE), runtime);
        runtime.detach();
    }
}
