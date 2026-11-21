use alaydriem_durability_ping::PingMod;
use amm_client::link::LinkUses;
use amm_client::test_support::FakeTable;
use amml_api::amml_abi::{
    BlockBreakContinue, BlockBreakStart, ClientTick, ItemUse, ItemUseOnBlock, PlayerAttack, WorldLeft,
};
use amml_api::testing::{FakeClient, FakeGame, FakeInventory, FakeItem};
use amml_api::{AmmlMod, Area};

const MANIFEST: &str = include_str!("../amml.hcl");

// a fake game that serves every row of the mod's `USES`, the manager provided, the mod enabled
fn enabled() -> FakeGame {
    let game = FakeTable::game();
    game.offer::<ClientTick>();
    game.offer::<WorldLeft>();
    game.offer::<BlockBreakStart>();
    game.offer::<BlockBreakContinue>();
    game.offer::<ItemUse>();
    game.offer::<PlayerAttack>();
    game.offer::<ItemUseOnBlock>();
    game.offer_area(Area::CLIENT_READ);
    game.offer_area(Area::INVENTORY_READ);
    game.offer_area(Area::INPUT_LEASE);
    FakeTable::provide(&game.ctx()).unwrap();
    game.load::<PingMod>().unwrap();
    game.enable::<PingMod>().unwrap();
    game.open_gate();
    game
}

// a client in a world whose local player holds `item` in slot 3
fn in_world(game: &FakeGame, item: Option<FakeItem>) {
    game.set_client(FakeClient { local_player: true, world_active: true, ..FakeClient::default() });
    game.set_inventory(FakeInventory { held: item, selected_slot: Some(3), ..FakeInventory::default() });
}

fn pickaxe(damage: i32) -> FakeItem {
    FakeItem { type_name: "minecraft:iron_pickaxe", count: 1, damage, max_damage: 250, ..FakeItem::EMPTY }
}

#[test]
fn the_module_string_is_the_namespaced_id() {
    assert_eq!(PingMod::MODULE, PingMod::id());
}

#[test]
fn the_uses_carry_every_row_the_manager_link_needs() {
    assert_eq!(LinkUses::check(&PingMod::USES), Ok(()));
}

#[test]
fn every_row_is_required() {
    assert!(PingMod::USES.rows().iter().all(|row| row.required != 0));
}

#[test]
fn the_manifest_uses_block_is_the_uses_of_the_mod() {
    let manifest = MANIFEST.replace("\r\n", "\n");
    assert_eq!(PingMod::USES.write_into(&manifest), manifest);
}

#[test]
fn enable_logs_and_a_tick_in_a_world_reads_the_held_slot() {
    let game = enabled();
    in_world(&game, Some(pickaxe(10)));

    game.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });

    let logs = game.logs();
    assert!(logs.contains(&"info @alaydriem/durability-ping enabled".to_string()), "{logs:?}");
    assert!(logs.contains(&"info held read slot=3 empty=false".to_string()), "{logs:?}");
}

#[test]
fn a_block_break_of_the_local_player_with_a_good_tool_is_seen_and_not_cancelled() {
    let game = enabled();
    in_world(&game, Some(pickaxe(10)));
    game.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    let mut event = BlockBreakStart {
        header: FakeGame::header::<BlockBreakStart>(),
        pos: Default::default(),
        face: Default::default(),
        is_local_player: 1,
        _reserved: [0; 2],
    };

    game.raise_with_client(&mut event);

    assert_eq!(event.header.cancelled, 0);
    assert!(game.logs().contains(&"info action seen kind=player.block_break_start local=true".to_string()));
}

#[test]
fn disable_removes_the_listeners() {
    let game = enabled();
    game.disable::<PingMod>();
    in_world(&game, Some(pickaxe(10)));

    game.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });

    assert!(!game.logs().iter().any(|line| line.starts_with("info held read")));
}
