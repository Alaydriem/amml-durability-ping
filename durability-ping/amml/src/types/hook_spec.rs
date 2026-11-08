use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

// one game hook: the entry of its target, the detour written for that function's signature, the detour's
// original slot, and the feature it serves. With `table`, `entry` is a slot and the target is the code pointer in
// that slot of the vtable the `table` entry names; without it, `entry` is the function address. A hook that is
// not required is skipped when one of its entries is missing
#[derive(Clone, Copy)]
pub struct HookSpec {
    pub entry: &'static str,
    pub table: Option<&'static str>,
    pub detour: *const c_void,
    pub original: &'static AtomicPtr<c_void>,
    pub required: bool,
    pub feature: &'static str,
}
