//! Recording and serialization of rally animation payloads.

mod recorder;
mod serialize;
mod types;

pub use serialize::is_legacy_javascript_result_data;
pub use types::{ANIMATION_PAYLOAD_VERSION, OreAnimationData, RecordedCpuStep, RobotCycleStatus};

pub(crate) use recorder::AnimationRecorder;
