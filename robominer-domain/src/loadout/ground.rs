use robominer_db::{MiningAreaOreSupplyRecord, MiningAreaRecord, RobotRecord};
use robominer_sim::{Ground, MAX_ORE_TYPES, RobotSpec};

use crate::error::DomainError;

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

pub(super) fn legacy_heap_center(
    size: usize,
    radius: usize,
    rng: &mut LegacyHeapPlacement,
) -> usize {
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
pub(super) struct LegacyHeapPlacement {
    state: u64,
}

impl LegacyHeapPlacement {
    pub(super) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_usize(&mut self, upper_bound: usize) -> usize {
        debug_assert!(upper_bound > 0);
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((self.state / 65_536) % 32_768) as usize % upper_bound
    }
}
