use std::sync::Arc;

use amml_api::amml_abi::{
    BlockBreakContinue, BlockBreakStart, ClientTick, ItemUse, ItemUseOnBlock, PlayerAttack, WorldLeft,
};
use amml_api::{ApiError, Cancel, Ctx, Event, EventCtx, HookOrder, Listener};

use crate::services::{ClientView, PingRuntime};

// the loader listeners of the mod; each hands its event to the runtime. Dropping them removes them
pub struct PingListeners {
    listeners: Vec<Listener>,
    runtime: Arc<PingRuntime>,
}

impl PingListeners {
    pub fn start(ctx: &Ctx, runtime: Arc<PingRuntime>) -> Result<Self, ApiError> {
        let events = ctx.events();
        let mut listeners = Vec::with_capacity(7);
        let ticking = Arc::clone(&runtime);
        listeners.push(events.listen::<ClientTick>(HookOrder::NORMAL, move |_, at| match at.client() {
            Some(client) => ticking.tick(Some(&ClientView::new(&client))),
            None => ticking.tick::<ClientView>(None),
        })?);
        let leaving = Arc::clone(&runtime);
        listeners.push(events.listen::<WorldLeft>(HookOrder::NORMAL, move |_, _| leaving.world_left())?);
        listeners.push(Self::guard::<BlockBreakStart>(ctx, &runtime, |event| event.is_local_player)?);
        listeners.push(Self::guard::<BlockBreakContinue>(ctx, &runtime, |event| event.is_local_player)?);
        listeners.push(Self::guard::<ItemUse>(ctx, &runtime, |event| event.is_local_player)?);
        listeners.push(Self::guard::<PlayerAttack>(ctx, &runtime, |event| event.is_local_player)?);
        listeners.push(Self::guard::<ItemUseOnBlock>(ctx, &runtime, |event| event.is_local_player)?);
        Ok(Self { listeners, runtime })
    }

    // the listeners go first, so no listener runs while the runtime lets go of the link
    pub fn stop(self) {
        let Self { listeners, runtime } = self;
        drop(listeners);
        runtime.detach();
    }

    // a guard listener on one player action: cancels it when the runtime refuses it. It runs first, so a refused
    // action is cancelled before the other mods see it
    fn guard<E: Event + Cancel>(
        ctx: &Ctx,
        runtime: &Arc<PingRuntime>,
        local: fn(&E) -> u8,
    ) -> Result<Listener, ApiError> {
        let runtime = Arc::clone(runtime);
        ctx.events().listen::<E>(HookOrder::FIRST, move |event, at: &EventCtx| {
            let is_local = local(event) != 0;
            let refused = match at.client() {
                Some(client) => runtime.action(<E as Event>::ID, is_local, Some(&ClientView::new(&client))),
                None => runtime.action::<ClientView>(<E as Event>::ID, is_local, None),
            };
            if refused {
                event.cancel();
            }
        })
    }
}
