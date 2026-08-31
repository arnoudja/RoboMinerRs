//! Mining area metadata, supplies, and overview reads.
//!
//! Primary entry points: [`list_mining_areas`], [`get_mining_area`],
//! [`list_mining_area_overview_areas`].

mod core;
mod overview;

pub use core::*;
pub use overview::*;
