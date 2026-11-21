//! @alaydriem/durability-ping as an AMML mod: a tone when the held tool is low, and an optional guard before it breaks.

#![deny(unsafe_code)]

pub mod ping_mod;
pub mod services;
pub mod traits;

pub use ping_mod::PingMod;
pub use services::{CuePlayer, DeviceLost, RodioOutput};
pub use traits::CueOutput;

amml_api::amml_mod!(PingMod);
