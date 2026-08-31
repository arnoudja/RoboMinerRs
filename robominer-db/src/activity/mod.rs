//! Activity feed and rally view read models.
//!
//! Primary entry points: [`list_activity_recent_rally_feed`],
//! [`list_activity_recent_rallies`], [`rally_view_state`].

mod read;
mod view;

pub use read::*;
pub use view::*;
