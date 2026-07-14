use robominer_db::{
    MiningAreaOreSupplyRecord, MiningAreaRecord, MiningRallyQueueRecord, MySqlPool, PoolItemRecord,
    PoolRecord, RobotPartRecord, RobotRecord,
};
use robominer_sim::{Ground, MAX_ORE_TYPES, Position, RobotSpec};

use crate::constants::{RALLY_EXPIRY_START_SECONDS, RALLY_SIZE};
use crate::error::{DomainError, RobotPartSlot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RobotLoadoutParts {
    pub ore_container: Option<RobotPartRecord>,
    pub mining_unit: Option<RobotPartRecord>,
    pub battery: Option<RobotPartRecord>,
    pub memory_module: Option<RobotPartRecord>,
    pub cpu: Option<RobotPartRecord>,
    pub engine: Option<RobotPartRecord>,
    pub ore_scanner: Option<RobotPartRecord>,
}

impl RobotLoadoutParts {
    pub fn empty() -> Self {
        Self {
            ore_container: None,
            mining_unit: None,
            battery: None,
            memory_module: None,
            cpu: None,
            engine: None,
            ore_scanner: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RobotLoadout {
    pub robot: RobotRecord,
    pub parts: RobotLoadoutParts,
}

impl RobotLoadout {
    pub fn new(robot: RobotRecord, parts: RobotLoadoutParts) -> Self {
        Self { robot, parts }
    }

    pub fn simulator_spec(&self) -> Result<RobotSpec, DomainError> {
        robot_record_to_spec(&self.robot)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MiningAreaLoadout {
    pub area: MiningAreaRecord,
    pub ore_supplies: Vec<MiningAreaOreSupplyRecord>,
    pub ai_robot: RobotLoadout,
}

impl MiningAreaLoadout {
    pub fn new(
        area: MiningAreaRecord,
        ore_supplies: Vec<MiningAreaOreSupplyRecord>,
        ai_robot: RobotLoadout,
    ) -> Self {
        Self {
            area,
            ore_supplies,
            ai_robot,
        }
    }

    pub fn simulator_ground_with_seed(&self, seed: u64) -> Result<Ground, DomainError> {
        mining_area_to_ground(&self.area, &self.ore_supplies, seed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyQueueEntry {
    pub queue: MiningRallyQueueRecord,
    pub robot: RobotLoadout,
}

impl RallyQueueEntry {
    pub fn new(queue: MiningRallyQueueRecord, robot: RobotLoadout) -> Self {
        Self { queue, robot }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyLoadout {
    pub mining_area: MiningAreaLoadout,
    pub queue_entries: Vec<RallyQueueEntry>,
}

impl RallyLoadout {
    pub fn new(mining_area: MiningAreaLoadout, queue_entries: Vec<RallyQueueEntry>) -> Self {
        Self {
            mining_area,
            queue_entries,
        }
    }

    pub fn ai_robot_count(&self) -> usize {
        RALLY_SIZE.saturating_sub(self.queue_entries.len())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolItemLoadout {
    pub item: PoolItemRecord,
    pub robot: RobotLoadout,
}

impl PoolItemLoadout {
    pub fn new(item: PoolItemRecord, robot: RobotLoadout) -> Self {
        Self { item, robot }
    }

    pub fn source_code(&self) -> &str {
        &self.item.source_code
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolLoadout {
    pub pool: PoolRecord,
    pub mining_area: MiningAreaLoadout,
    pub items: Vec<PoolItemLoadout>,
}

impl PoolLoadout {
    pub fn new(
        pool: PoolRecord,
        mining_area: MiningAreaLoadout,
        items: Vec<PoolItemLoadout>,
    ) -> Self {
        Self {
            pool,
            mining_area,
            items,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.items
            .iter()
            .all(|item| item.item.runs_done >= self.pool.required_runs)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyOutcome {
    pub mining_area_id: i64,
    pub final_time: i32,
    pub participants: Vec<RallyParticipantOutcome>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyRun {
    pub outcome: RallyOutcome,
    pub result_data: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyParticipantOutcome {
    pub player_number: usize,
    pub queue_id: Option<i64>,
    pub robot_id: i64,
    pub is_ai: bool,
    pub position: Position,
    pub ore: [i32; MAX_ORE_TYPES],
    pub score: f64,
    pub actions_done: [i32; 8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolRallyOutcome {
    pub pool_id: i64,
    pub mining_area_id: i64,
    pub final_time: i32,
    pub items: Vec<PoolItemOutcome>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolItemOutcome {
    pub player_number: usize,
    pub pool_item_id: i64,
    pub robot_id: i64,
    pub score: f64,
    pub ore_results: Vec<PoolItemOreOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolItemOreOutcome {
    pub ore_id: i64,
    pub amount: i32,
}

pub fn mining_rally_queue_is_ready(queue_rows: &[MiningRallyQueueRecord]) -> bool {
    !queue_rows.is_empty()
        && (queue_rows.len() >= RALLY_SIZE
            || queue_rows[0].seconds_left < RALLY_EXPIRY_START_SECONDS)
}

pub async fn load_robot_loadout(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Option<RobotLoadout>, DomainError> {
    let Some(robot) = robominer_db::get_robot(pool, robot_id).await? else {
        return Ok(None);
    };

    let parts = load_robot_parts(pool, &robot).await?;

    Ok(Some(RobotLoadout::new(robot, parts)))
}

pub async fn load_mining_area_loadout(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Option<MiningAreaLoadout>, DomainError> {
    let Some(area) = robominer_db::get_mining_area(pool, mining_area_id).await? else {
        return Ok(None);
    };

    let ore_supplies = robominer_db::list_mining_area_ore_supplies(pool, area.id).await?;
    let ai_robot = load_robot_loadout(pool, area.ai_robot_id).await?.ok_or(
        DomainError::ReferencedAiRobotMissing {
            mining_area_id: area.id,
            robot_id: area.ai_robot_id,
        },
    )?;

    Ok(Some(MiningAreaLoadout::new(area, ore_supplies, ai_robot)))
}

pub async fn load_next_rally_loadout(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Option<RallyLoadout>, DomainError> {
    let Some(mining_area) = load_mining_area_loadout(pool, mining_area_id).await? else {
        return Ok(None);
    };

    let queue_rows =
        robominer_db::list_next_mining_rally_queue_for_area(pool, mining_area_id).await?;

    if !mining_rally_queue_is_ready(&queue_rows) {
        return Ok(None);
    }

    let mut queue_entries = Vec::with_capacity(queue_rows.len());

    for queue in queue_rows {
        let robot = load_robot_loadout(pool, queue.queue.robot_id)
            .await?
            .ok_or(DomainError::ReferencedQueueRobotMissing {
                mining_queue_id: queue.queue.id,
                robot_id: queue.queue.robot_id,
            })?;

        queue_entries.push(RallyQueueEntry::new(queue, robot));
    }

    Ok(Some(RallyLoadout::new(mining_area, queue_entries)))
}
pub async fn load_pool_loadout(
    pool: &MySqlPool,
    pool_id: i64,
) -> Result<Option<PoolLoadout>, DomainError> {
    let Some(pool_record) = robominer_db::get_pool(pool, pool_id).await? else {
        return Ok(None);
    };

    let mining_area = load_mining_area_loadout(pool, pool_record.mining_area_id)
        .await?
        .ok_or(DomainError::ReferencedPoolMiningAreaMissing {
            pool_id: pool_record.id,
            mining_area_id: pool_record.mining_area_id,
        })?;
    let item_rows = robominer_db::list_pool_items(pool, pool_record.id).await?;
    let mut items = Vec::with_capacity(item_rows.len());

    for item in item_rows {
        let robot = load_robot_loadout(pool, item.robot_id).await?.ok_or(
            DomainError::ReferencedPoolRobotMissing {
                pool_item_id: item.id,
                robot_id: item.robot_id,
            },
        )?;

        items.push(PoolItemLoadout::new(item, robot));
    }

    Ok(Some(PoolLoadout::new(pool_record, mining_area, items)))
}

pub async fn load_next_pool_rally_loadout(
    pool: &MySqlPool,
    pool_id: i64,
) -> Result<Option<PoolLoadout>, DomainError> {
    let Some(pool_record) = robominer_db::get_pool(pool, pool_id).await? else {
        return Ok(None);
    };

    let mining_area = load_mining_area_loadout(pool, pool_record.mining_area_id)
        .await?
        .ok_or(DomainError::ReferencedPoolMiningAreaMissing {
            pool_id: pool_record.id,
            mining_area_id: pool_record.mining_area_id,
        })?;
    let item_rows = robominer_db::list_next_pool_rally_items(pool, pool_record.id).await?;
    let mut items = Vec::with_capacity(item_rows.len());

    for item in item_rows {
        let robot = load_robot_loadout(pool, item.robot_id).await?.ok_or(
            DomainError::ReferencedPoolRobotMissing {
                pool_item_id: item.id,
                robot_id: item.robot_id,
            },
        )?;

        items.push(PoolItemLoadout::new(item, robot));
    }

    Ok(Some(PoolLoadout::new(pool_record, mining_area, items)))
}
pub fn robot_record_to_spec(robot: &RobotRecord) -> Result<RobotSpec, DomainError> {
    Ok(RobotSpec {
        robot_id: i32::try_from(robot.id).map_err(|_| DomainError::RobotIdOutOfRange(robot.id))?,
        max_turns: robot.max_turns,
        max_ore: robot.max_ore,
        mining_speed: robot.mining_speed,
        cpu_speed: robot.cpu_speed,
        forward_speed: robot.forward_speed,
        backward_speed: robot.backward_speed,
        rotate_speed: robot.rotate_speed,
        robot_size: robot.robot_size,
        scan_time: robot.scan_time,
        scan_distance: robot.scan_distance,
    })
}

pub fn mining_area_to_ground(
    area: &MiningAreaRecord,
    ore_supplies: &[MiningAreaOreSupplyRecord],
    seed: u64,
) -> Result<Ground, DomainError> {
    let (size_x, size_y) = simulator_dimensions(area)?;
    let mut ground = Ground::new(size_x, size_y);
    let mut rng = LegacyHeapPlacement::new(seed);
    let mut ore_data = Vec::new();
    let mut supplies = ore_supplies.to_vec();

    supplies.sort_by(|left, right| {
        right
            .ore_id
            .cmp(&left.ore_id)
            .then_with(|| left.id.cmp(&right.id))
    });

    for supply in &supplies {
        validate_ore_supply(supply)?;
        let ore_type = legacy_ore_type(area.id, &mut ore_data, supply.ore_id)?;
        let radius = supply.radius as usize;
        let center_x = legacy_heap_center(size_x, radius, &mut rng);
        let center_y = legacy_heap_center(size_y, radius, &mut rng);

        ground.add_ore_heap(center_x, center_y, ore_type, supply.supply, supply.radius);
    }

    Ok(ground)
}
async fn load_robot_parts(
    pool: &MySqlPool,
    robot: &RobotRecord,
) -> Result<RobotLoadoutParts, DomainError> {
    Ok(RobotLoadoutParts {
        ore_container: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::OreContainer,
            robot.ore_container_id,
        )
        .await?,
        mining_unit: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::MiningUnit,
            robot.mining_unit_id,
        )
        .await?,
        battery: load_optional_robot_part(pool, robot.id, RobotPartSlot::Battery, robot.battery_id)
            .await?,
        memory_module: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::MemoryModule,
            robot.memory_module_id,
        )
        .await?,
        cpu: load_optional_robot_part(pool, robot.id, RobotPartSlot::Cpu, robot.cpu_id).await?,
        engine: load_optional_robot_part(pool, robot.id, RobotPartSlot::Engine, robot.engine_id)
            .await?,
        ore_scanner: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::OreScanner,
            robot.ore_scanner_id,
        )
        .await?,
    })
}

async fn load_optional_robot_part(
    pool: &MySqlPool,
    robot_id: i64,
    slot: RobotPartSlot,
    part_id: Option<i64>,
) -> Result<Option<RobotPartRecord>, DomainError> {
    let Some(part_id) = part_id else {
        return Ok(None);
    };

    robominer_db::get_robot_part(pool, part_id)
        .await?
        .ok_or(DomainError::ReferencedRobotPartMissing {
            robot_id,
            slot,
            part_id,
        })
        .map(Some)
}
fn simulator_dimensions(area: &MiningAreaRecord) -> Result<(usize, usize), DomainError> {
    if area.size_x < 2 || area.size_y < 2 {
        return Err(DomainError::InvalidMiningAreaSize {
            mining_area_id: area.id,
            size_x: area.size_x,
            size_y: area.size_y,
        });
    }

    let size_x = usize::try_from(area.size_x).map_err(|_| DomainError::InvalidMiningAreaSize {
        mining_area_id: area.id,
        size_x: area.size_x,
        size_y: area.size_y,
    })?;
    let size_y = usize::try_from(area.size_y).map_err(|_| DomainError::InvalidMiningAreaSize {
        mining_area_id: area.id,
        size_x: area.size_x,
        size_y: area.size_y,
    })?;

    Ok((size_x, size_y))
}

pub(crate) fn validate_ore_supply(supply: &MiningAreaOreSupplyRecord) -> Result<(), DomainError> {
    if supply.ore_id <= 0 || supply.supply < 0 || supply.radius <= 0 {
        return Err(DomainError::InvalidMiningAreaOreSupply {
            supply_id: supply.id,
            ore_id: supply.ore_id,
            supply: supply.supply,
            radius: supply.radius,
        });
    }

    Ok(())
}

fn legacy_ore_type(
    mining_area_id: i64,
    ore_data: &mut Vec<i64>,
    ore_id: i64,
) -> Result<usize, DomainError> {
    if let Some(index) = ore_data
        .iter()
        .position(|known_ore_id| *known_ore_id == ore_id)
    {
        return Ok(index);
    }

    if ore_data.len() == MAX_ORE_TYPES {
        return Err(DomainError::TooManyMiningAreaOreTypes {
            mining_area_id,
            ore_type_count: ore_data.len() + 1,
        });
    }

    let index = ore_data.len();
    ore_data.push(ore_id);

    Ok(index)
}

fn legacy_heap_center(size: usize, radius: usize, rng: &mut LegacyHeapPlacement) -> usize {
    if size == 0 {
        return 0;
    }

    let max_center = size.saturating_sub(1).saturating_sub(radius);
    if radius > max_center {
        return size / 2;
    }

    radius + rng.next_usize(max_center - radius + 1)
}
#[derive(Clone, Debug)]
struct LegacyHeapPlacement {
    state: u64,
}

impl LegacyHeapPlacement {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_usize(&mut self, upper_bound: usize) -> usize {
        debug_assert!(upper_bound > 0);
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((self.state / 65_536) % 32_768) as usize % upper_bound
    }
}

#[cfg(test)]
mod tests {
    use super::{legacy_heap_center, mining_area_to_ground, LegacyHeapPlacement};
    use crate::validate_ore_supply;
    use robominer_db::{MiningAreaOreSupplyRecord, MiningAreaRecord};

    #[test]
    fn legacy_heap_center_keeps_radius_inside_bounds() {
        let mut rng = LegacyHeapPlacement::new(42);
        for size in 4..=20 {
            for radius in 1..size {
                for _ in 0..20 {
                    let center = legacy_heap_center(size, radius, &mut rng);
                    if radius * 2 < size {
                        assert!(center >= radius, "size={size} radius={radius} center={center}");
                        assert!(
                            center + radius < size,
                            "size={size} radius={radius} center={center}"
                        );
                    } else {
                        assert_eq!(center, size / 2);
                    }
                }
            }
        }
    }

    #[test]
    fn single_ore_heap_fits_inside_mining_area() {
        let area = MiningAreaRecord {
            id: 1001,
            area_name: "Cerbonium-mini".to_string(),
            ore_price_id: 10001,
            size_x: 10,
            size_y: 10,
            max_moves: 15,
            mining_time: 5,
            tax_rate: 25,
            ai_robot_id: 1,
        };
        let supply = MiningAreaOreSupplyRecord {
            id: 1,
            mining_area_id: 1001,
            ore_id: 1,
            supply: 4,
            radius: 4,
        };
        validate_ore_supply(&supply).expect("valid supply");

        let ground = mining_area_to_ground(&area, &[supply], 999).expect("ground should build");

        for y in 0..ground.size_y() {
            for edge_x in [0, ground.size_x() - 1] {
                assert_eq!(
                    ground.at(edge_x, y).ore_at(0),
                    0,
                    "ore should not reach x={edge_x}"
                );
            }
        }

        for x in 0..ground.size_x() {
            for edge_y in [0, ground.size_y() - 1] {
                assert_eq!(
                    ground.at(x, edge_y).ore_at(0),
                    0,
                    "ore should not reach y={edge_y}"
                );
            }
        }
    }
}