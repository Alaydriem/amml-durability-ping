use std::sync::{Arc, Mutex, PoisonError};

use amml_api::amml_abi::{
    BlockBreakContinue, BlockBreakStart, ClientTick, InjectedInputReleased, InputLeaseLost, InputLeaseResumed,
    InputLeaseSuspended, ItemUse, ItemUseOnBlock, ModsLoaded, PlayerAttack, ProviderGone, ProviderReady,
    ScreenshotTaken, ScreenshotWritten, WorldLeft,
};
use amml_api::{AmmlMod, ApiError, Area, Ctx, DisableCtx, Uses};

use crate::services::{PingLink, PingListeners, PingRuntime};

// what the enabled mod owns. `disable` takes it, which drops the listeners and closes the link
static RUNNING: Mutex<Option<PingListeners>> = Mutex::new(None);

// the AMML entry point: enable starts the manager link and the listeners, disable removes them and leaves the
// mod manager
pub struct PingMod;

impl PingMod {
    // the id as the module string the loader logs under; a test checks it against the namespace and name
    pub const MODULE: &'static str = "@alaydriem/durability-ping";
}

impl AmmlMod for PingMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "durability-ping";
    const USES: Uses = amml_api::amml_uses! {
        feature "alert" {
            required ClientTick;
            required WorldLeft;
            required ProviderReady;
            required ProviderGone;
            required area Area::CLIENT_READ;
            required area Area::INVENTORY_READ;
            // the manager link uses these
            required ModsLoaded;
            required InputLeaseSuspended;
            required InputLeaseResumed;
            required InputLeaseLost;
            required InjectedInputReleased;
            required ScreenshotTaken;
            required ScreenshotWritten;
            required area Area::INPUT_LEASE;
        }
        feature "guard" {
            required BlockBreakStart;
            required BlockBreakContinue;
            required ItemUse;
            required PlayerAttack;
            required ItemUseOnBlock;
        }
    };

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        let log = ctx.log();
        let link = PingLink::start(ctx).map_err(|error| {
            log.error("durability-ping off", &[("error", &error.to_string())]);
            ApiError::Refused
        })?;
        let runtime = Arc::new(PingRuntime::new(log));
        runtime.attach(link);
        let listeners = PingListeners::start(ctx, runtime).inspect_err(|error| {
            log.error("durability-ping off", &[("error", &error.to_string())]);
        })?;
        *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(listeners);
        log.info("@alaydriem/durability-ping enabled", &[]);
        Ok(())
    }

    fn disable(_ctx: &Ctx, _how: &DisableCtx) {
        let running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(listeners) = running {
            listeners.stop();
        }
    }
}
