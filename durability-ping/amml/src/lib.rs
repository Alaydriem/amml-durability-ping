//! @amml/durability-ping as an AMML mod: a tone when the held tool is low, and an optional guard before it breaks.

#![deny(unsafe_code)]

pub mod detours;
pub mod ping_mod;
pub mod services;
pub mod traits;
pub mod types;

pub use ping_mod::PingMod;
pub use services::{CuePlayer, DeviceLost, RodioOutput};
pub use traits::CueOutput;

amml_sdk::amml_mod!(PingMod);
