use std::fmt;

use robominer_program::CompileError;
use robominer_sim::MAX_ORE_TYPES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobotPartSlot {
    OreContainer,
    MiningUnit,
    Battery,
    MemoryModule,
    Cpu,
    Engine,
    OreScanner,
}

impl fmt::Display for RobotPartSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::OreContainer => "ore container",
            Self::MiningUnit => "mining unit",
            Self::Battery => "battery",
            Self::MemoryModule => "memory module",
            Self::Cpu => "CPU",
            Self::Engine => "engine",
            Self::OreScanner => "ore scanner",
        };

        f.write_str(name)
    }
}

/// Opaque persistence failure. Keeps `sqlx` out of the domain error surface.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct DatabaseError {
    message: String,
}

impl DatabaseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(error: sqlx::Error) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("database error: {0}")]
    Database(#[source] DatabaseError),
    /// A `spawn_blocking` join failed (task panicked or was cancelled).
    #[error("background task failed: {operation}")]
    BackgroundTask { operation: &'static str },
    #[error("mining area {mining_area_id} references missing AI robot {robot_id}")]
    ReferencedAiRobotMissing { mining_area_id: i64, robot_id: i64 },
    #[error("robot {robot_id} references missing {slot} robot part {part_id}")]
    ReferencedRobotPartMissing {
        robot_id: i64,
        slot: RobotPartSlot,
        part_id: i64,
    },
    #[error("mining queue item {mining_queue_id} references missing robot {robot_id}")]
    ReferencedQueueRobotMissing { mining_queue_id: i64, robot_id: i64 },
    #[error("pool {pool_id} references missing mining area {mining_area_id}")]
    ReferencedPoolMiningAreaMissing { pool_id: i64, mining_area_id: i64 },
    #[error("pool item {pool_item_id} references missing robot {robot_id}")]
    ReferencedPoolRobotMissing { pool_item_id: i64, robot_id: i64 },
    #[error("robot id {0} does not fit simulator robot ids")]
    RobotIdOutOfRange(i64),
    #[error("mining area {mining_area_id} has invalid simulator size {size_x}x{size_y}")]
    InvalidMiningAreaSize {
        mining_area_id: i64,
        size_x: i32,
        size_y: i32,
    },
    #[error(
        "mining area ore supply {supply_id} has invalid ore_id={ore_id}, supply={supply}, radius={radius}"
    )]
    InvalidMiningAreaOreSupply {
        supply_id: i64,
        ore_id: i64,
        supply: i32,
        radius: i32,
    },
    #[error(
        "mining area {mining_area_id} uses {ore_type_count} ore types, but the simulator supports {max}",
        max = MAX_ORE_TYPES
    )]
    TooManyMiningAreaOreTypes {
        mining_area_id: i64,
        ore_type_count: usize,
    },
    #[error("mining area {mining_area_id} has invalid rally queue size {queue_entries}")]
    InvalidRallyLoadout {
        mining_area_id: i64,
        queue_entries: usize,
    },
    #[error("pool {pool_id} has invalid rally item count {items}")]
    InvalidPoolLoadout { pool_id: i64, items: usize },
    #[error("robot {robot_id} program does not compile: {source}")]
    ProgramCompile {
        robot_id: i64,
        #[source]
        source: CompileError,
    },
    #[error("rally outcome does not match mining area {mining_area_id} loadout")]
    RallyOutcomeMismatch { mining_area_id: i64 },
    #[error("pool outcome does not match pool {pool_id} loadout")]
    PoolOutcomeMismatch { pool_id: i64 },
}

impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(DatabaseError::from(error))
    }
}

impl From<DatabaseError> for DomainError {
    fn from(error: DatabaseError) -> Self {
        Self::Database(error)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use robominer_program::CompileError;

    use super::{DatabaseError, DomainError, RobotPartSlot};

    #[test]
    fn domain_error_display_includes_database_context() {
        let error = DomainError::Database(DatabaseError::new("database url missing"));
        assert!(error.to_string().contains("database error"));
        assert!(error.source().is_some());
    }

    #[test]
    fn domain_error_display_covers_reference_and_loadout_variants() {
        let cases = [
            (
                DomainError::ReferencedAiRobotMissing {
                    mining_area_id: 1,
                    robot_id: 2,
                },
                "missing AI robot 2",
            ),
            (
                DomainError::ReferencedRobotPartMissing {
                    robot_id: 4,
                    slot: RobotPartSlot::Engine,
                    part_id: 99,
                },
                "missing engine robot part 99",
            ),
            (
                DomainError::ReferencedQueueRobotMissing {
                    mining_queue_id: 3,
                    robot_id: 5,
                },
                "mining queue item 3",
            ),
            (
                DomainError::ReferencedPoolMiningAreaMissing {
                    pool_id: 7,
                    mining_area_id: 8,
                },
                "pool 7 references missing mining area 8",
            ),
            (
                DomainError::ReferencedPoolRobotMissing {
                    pool_item_id: 9,
                    robot_id: 10,
                },
                "pool item 9",
            ),
            (DomainError::RobotIdOutOfRange(1_000_000), "does not fit"),
            (
                DomainError::InvalidMiningAreaSize {
                    mining_area_id: 11,
                    size_x: 0,
                    size_y: 5,
                },
                "invalid simulator size 0x5",
            ),
            (
                DomainError::InvalidMiningAreaOreSupply {
                    supply_id: 12,
                    ore_id: -1,
                    supply: 0,
                    radius: -2,
                },
                "ore_id=-1",
            ),
            (
                DomainError::TooManyMiningAreaOreTypes {
                    mining_area_id: 13,
                    ore_type_count: 99,
                },
                "99 ore types",
            ),
            (
                DomainError::InvalidRallyLoadout {
                    mining_area_id: 14,
                    queue_entries: 1,
                },
                "invalid rally queue size 1",
            ),
            (
                DomainError::InvalidPoolLoadout {
                    pool_id: 15,
                    items: 2,
                },
                "invalid rally item count 2",
            ),
            (
                DomainError::RallyOutcomeMismatch { mining_area_id: 16 },
                "does not match mining area 16",
            ),
            (
                DomainError::PoolOutcomeMismatch { pool_id: 17 },
                "does not match pool 17",
            ),
        ];

        for (error, needle) in cases {
            assert!(
                error.to_string().contains(needle),
                "expected {:?} to contain {needle:?}, got {}",
                std::mem::discriminant(&error),
                error
            );
            assert!(error.source().is_none());
        }
    }

    #[test]
    fn domain_error_program_compile_includes_source() {
        let error = DomainError::ProgramCompile {
            robot_id: 42,
            source: CompileError::new("syntax error"),
        };
        assert!(error.to_string().contains("robot 42"));
        assert!(error.to_string().contains("does not compile"));
        assert!(error.source().is_some());
    }

    #[test]
    fn robot_part_slot_display_names() {
        assert_eq!(RobotPartSlot::OreContainer.to_string(), "ore container");
        assert_eq!(RobotPartSlot::MiningUnit.to_string(), "mining unit");
        assert_eq!(RobotPartSlot::Battery.to_string(), "battery");
        assert_eq!(RobotPartSlot::MemoryModule.to_string(), "memory module");
        assert_eq!(RobotPartSlot::Cpu.to_string(), "CPU");
        assert_eq!(RobotPartSlot::Engine.to_string(), "engine");
        assert_eq!(RobotPartSlot::OreScanner.to_string(), "ore scanner");
    }

    #[test]
    fn domain_error_from_sqlx_wraps_as_database() {
        let error = DomainError::from(sqlx::Error::Configuration("missing".into()));
        assert!(matches!(error, DomainError::Database(_)));
    }
}
