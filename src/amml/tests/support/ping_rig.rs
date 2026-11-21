use alaydriem_durability_ping::services::{PingDescriptor, PingLink, PingRuntime};
use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;
use amml_api::testing::FakeGame;

use crate::support::{FakeOutput, FakeView};

// a runtime with a fake audio output and a link to a fake manager that is there from the start
pub struct PingRig {
    pub game: FakeGame,
    pub output: FakeOutput,
    pub runtime: PingRuntime,
}

impl PingRig {
    // the manager is provided with `config` as its settings text
    pub fn with_config(config: &str) -> Self {
        let game = FakeTable::game();
        FakeTable::provide(&game.ctx()).unwrap();
        FakeTable::set_config(config);
        let ctx = FakeTable::ctx_of(&game, PingDescriptor::TEXT);
        let output = FakeOutput::default();
        let runtime = PingRuntime::with_output(ctx.log(), output.clone());
        runtime.attach(PingLink::start(&ctx).unwrap());
        Self { game, output, runtime }
    }

    // the guard on, with its limits at 2 uses (percent off)
    pub fn guarding() -> Self {
        let rig = Self::with_config("block {\n  enabled = true\n  percent = 0\n  uses = 2\n}\n");
        rig.tick(&FakeView::holding(100));
        rig
    }

    pub fn tick(&self, view: &FakeView) {
        self.runtime.tick(Some(view));
    }

    pub fn push(&self, event: ModEvent) {
        FakeTable::push_event(event);
    }

    // true when the mod log holds `line`
    pub fn logged(&self, line: &str) -> bool {
        self.game.logs().iter().any(|logged| logged == line)
    }

    pub fn count_logged(&self, prefix: &str) -> usize {
        self.game.logs().iter().filter(|logged| logged.starts_with(prefix)).count()
    }
}
