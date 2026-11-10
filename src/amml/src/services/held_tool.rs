use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{PoisonError, RwLock};

use amml_game::{ClientInstance, GameEntries, PlayerInventoryRef};
use durability_ping_core::structs::ToolReading;

use crate::traits::GameView;

// the game's primary ClientInstance and the entries to read it with.
// The client is the first `this` seen by ClientInstance::update: the primary client is created at start-up
// and lives until the game closes, while split-screen clients come and go
pub struct HeldTool {
    client: AtomicUsize,
    entries: RwLock<Option<GameEntries>>,
}

impl HeldTool {
    pub const fn new() -> Self {
        Self { client: AtomicUsize::new(0), entries: RwLock::new(None) }
    }

    pub fn set_entries(&self, entries: GameEntries) {
        *self.entries.write().unwrap_or_else(PoisonError::into_inner) = Some(entries);
    }

    // keeps the first non-null client; later calls change nothing
    pub fn capture(&self, client: usize) {
        if client != 0 {
            let _ = self.client.compare_exchange(0, client, Ordering::Relaxed, Ordering::Relaxed);
        }
    }

    pub fn client(&self) -> usize {
        self.client.load(Ordering::Relaxed)
    }

    // None before the first update or without the entries
    fn with_client<R>(&self, body: impl FnOnce(&ClientInstance<'_>, &GameEntries) -> R) -> Option<R> {
        let client = self.client();
        if client == 0 {
            return None;
        }
        let entries = self.entries.read().unwrap_or_else(PoisonError::into_inner);
        let entries = entries.as_ref()?;
        // SAFETY: `client` is the ClientInstance* the game passed to ClientInstance::update, and the primary
        // client lives until the game closes; `entries` were resolved on this game build. The calls run on the client
        // thread: in the tick inside ClientInstance::update, or in a guard detour only after PingRuntime::should_block
        // checked that the caller runs on the thread of the last tick (the client thread) and matched it to the
        // local player of the last tick.
        #[allow(unsafe_code)]
        let instance = unsafe { ClientInstance::from_raw(client as *mut c_void, entries) }?;
        Some(body(&instance, entries))
    }
}

impl GameView for HeldTool {
    fn showing_menu(&self) -> bool {
        self.with_client(|client, _| client.is_showing_menu()).unwrap_or(false)
    }

    fn local_player(&self) -> usize {
        self.with_client(|client, _| client.local_player().map_or(0, |player| player.as_ptr() as usize)).unwrap_or(0)
    }

    fn held_tool(&self) -> Option<ToolReading> {
        self.with_client(|client, entries| {
            let player = client.local_player()?;
            // SAFETY: `player` is the live local player the client holds, read on the client thread (in the tick, or
            // in a guard detour after the client-thread check and the local-player match, as above); the inventory is
            // not kept past this call.
            #[allow(unsafe_code)]
            let inventory = unsafe { PlayerInventoryRef::of_player(player.as_ptr(), entries) }?;
            let slot = inventory.selected()?;
            let stack = inventory.selected_item()?;
            if stack.is_empty() {
                return None;
            }
            ToolReading::new(slot, stack.type_name()?, stack.damage()?, stack.max_damage()?)
        })
        .flatten()
    }
}

impl Default for HeldTool {
    fn default() -> Self {
        Self::new()
    }
}
