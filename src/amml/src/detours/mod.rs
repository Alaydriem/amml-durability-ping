// The game functions the mod hooks. bool arguments and results cross as bytes, so no bool invariant is assumed of
// the game's registers; register arguments whose width is not proven cross as u64 so they reach the original bit
// for bit. A guard detour that stops an action skips the original and returns the "not handled" value

use amml_sdk::amml_detour;

use crate::services::PingRuntime;

// bool ClientInstance::update(bool isInitFinished) (26.60: update(bool, bool)): once per frame on the game thread
// 26.60 added a bool third argument (r8). It is carried as a raw u64 so the register reaches the original
// bit for bit; 26.51 ignores r8, so one declaration works on both builds
amml_detour!(pub mod client_update(this: usize, init_finished: u8, extra: u64) -> u8 |original| {
    let result = original(this, init_finished, extra);
    if let Some(runtime) = PingRuntime::get() {
        runtime.tick(this);
    }
    result
});

// whether the action of the player that owns the GameMode `this` must be stopped (with the blocked tone)
fn refuse_game_mode(this: usize) -> bool {
    let Some(runtime) = PingRuntime::get() else { return false };
    let offset = runtime.game_mode_player_offset();
    if this == 0 || offset == 0 {
        return false;
    }
    // SAFETY: `this` is the GameMode* the game passed to one of its own GameMode virtuals, a live object for the
    // length of the call, and `offset` is the derived offset of its `Player& mPlayer` field (a pointer) on this
    // build; the read does not outlive the call
    #[allow(unsafe_code)]
    let player = unsafe { std::ptr::read_unaligned((this + offset) as *const usize) };
    runtime.refuse(player)
}

// a refused destroy call leaves `hasDestroyedBlock` false, as the base bodies write it first thing (Task 8 slots.md)
fn clear_destroyed(destroyed: usize) {
    if destroyed != 0 {
        // SAFETY: `destroyed` is the bool& the caller passed for this call to fill; the game's own body writes this
        // byte unconditionally
        #[allow(unsafe_code)]
        unsafe {
            std::ptr::write(destroyed as *mut u8, 0);
        }
    }
}

// bool GameMode::startDestroyBlock(BlockPos const&, uchar face, bool& hasDestroyedBlock) (slot 1): the first hit
// of a block. rdx BlockPos*, r8b face, r9 bool*; false = not handled
amml_detour!(pub mod start_destroy_block(this: usize, pos: usize, face: u64, destroyed: usize) -> u8 |original| {
    if refuse_game_mode(this) {
        clear_destroyed(destroyed);
        return 0;
    }
    original(this, pos, face, destroyed)
});

// bool GameMode::continueDestroyBlock(BlockPos const&, uchar face, Vec3 const& playerPos, bool& hasDestroyedBlock)
// (slot 3): each tick of a held mining action. The fifth argument is on the stack ([rsp+28h]); false = not handled
amml_detour!(pub mod continue_destroy_block(
    this: usize,
    pos: usize,
    face: u64,
    player_pos: usize,
    destroyed: usize,
) -> u8 |original| {
    if refuse_game_mode(this) {
        clear_destroyed(destroyed);
        return 0;
    }
    original(this, pos, face, player_pos, destroyed)
});

// bool GameMode::useItem(ItemStack&, bool) (slot 11): a use in the air (bow draw, rod cast). r8b is a bool the
// header does not name; false = not handled
amml_detour!(pub mod use_item(this: usize, stack: usize, flag: u64) -> u8 |original| {
    if refuse_game_mode(this) {
        return 0;
    }
    original(this, stack, flag)
});

// bool GameMode::attack(Actor& target, Vec3 const&) (slot 15 of the base table: creative, adventure, spectator);
// false = not handled
amml_detour!(pub mod attack(this: usize, target: usize, click: usize) -> u8 |original| {
    if refuse_game_mode(this) {
        return 0;
    }
    original(this, target, click)
});

// the same attack through slot 15 of the SurvivalMode table, whose leaf skips the base slot. A detour of its own,
// so each hook keeps its own original
amml_detour!(pub mod survival_attack(this: usize, target: usize, click: usize) -> u8 |original| {
    if refuse_game_mode(this) {
        return 0;
    }
    original(this, target, click)
});

// InteractionResult Item::useOn(ItemStack&, Actor&, BlockPos const&, uchar face, uchar, Vec3 const& click, u64)
// const: a use on a block (tilling, stripping, a path, flint and steel). The result comes back through the hidden
// pointer in rdx, which rax returns. Mobs and item behaviours reach it too, so only the local player's use is
// tested. A refusal writes the one-byte result 0 (no success, no swing) and returns the pointer
amml_detour!(pub mod item_use_on(
    item: usize,
    result: usize,
    stack: usize,
    actor: usize,
    pos: usize,
    face: u64,
    extra: u64,
    click: usize,
    flags: u64,
) -> usize |original| {
    let refused = result != 0 && PingRuntime::get().is_some_and(|runtime| runtime.refuse(actor));
    if refused {
        // SAFETY: `result` is the InteractionResult the caller passed for this call to fill; its first byte is the
        // result flags the body itself writes
        #[allow(unsafe_code)]
        unsafe {
            std::ptr::write(result as *mut u8, 0);
        }
        return result;
    }
    original(item, result, stack, actor, pos, face, extra, click, flags)
});
