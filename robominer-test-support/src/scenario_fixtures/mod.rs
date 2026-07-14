mod achievement;
mod achievement_cli;
mod cancel_mining_queue;
mod claim_results;
mod default_robot_parts;
mod enqueue_mining;
mod mining_queue;
mod pool;
mod program_source;
mod rally;
mod robot_apply;
mod robot_config;
mod shop;
mod web_smoke;

pub use achievement::AchievementScenario;
pub use achievement_cli::AchievementCliFixture;
pub use cancel_mining_queue::CancelMiningQueueFixture;
pub use claim_results::ClaimResultsFixture;
pub use default_robot_parts::ensure_default_robot_parts;
pub use enqueue_mining::EnqueueMiningFixture;
pub use mining_queue::{IdleMiningAreaFixture, QueuedMiningAreaFixture, RobotMiningAreaFixture};
pub use pool::PoolFixture;
pub use program_source::{parse_created_program_source_id, ProgramSourceFixture};
pub use rally::RallyFixture;
pub use robot_apply::RobotApplyFixture;
pub use robot_config::{
    assert_robot_parameters, insert_robot_config_part, insert_user_robot_part_asset,
    RobotConfigFixture,
};
pub use shop::{ShopCatalog, ShopFixture};
pub use web_smoke::{web_smoke_prefix, WebSmokeDbFixture};
