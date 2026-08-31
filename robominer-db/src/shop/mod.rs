//! Shop catalog reads and part buy/sell mutations.
//!
//! Primary entry points: [`buy_robot_part`], [`sell_robot_part`],
//! [`list_shop_robot_part_states`].

mod assets;
mod read;
mod write;

pub(crate) use assets::*;
pub use read::*;
pub use write::*;
