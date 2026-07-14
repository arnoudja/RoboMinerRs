use crate::MAX_ORE_TYPES;
use crate::position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroundUnit {
    ore: [i32; MAX_ORE_TYPES],
}

impl GroundUnit {
    pub fn ore(&self) -> &[i32; MAX_ORE_TYPES] {
        &self.ore
    }

    pub fn ore_at(&self, ore_type: usize) -> i32 {
        self.ore[ore_type]
    }

    pub fn add_ore(&mut self, ore_type: usize, amount: i32) {
        self.ore[ore_type] += amount;
    }

    pub fn remove_ore(&mut self, ore_type: usize, amount: i32) {
        assert!(amount >= 0);
        assert!(amount <= self.ore[ore_type]);

        self.ore[ore_type] -= amount;
    }

    pub(crate) fn ore_type_count(&self) -> i32 {
        self.ore.iter().filter(|amount| **amount > 0).count() as i32
    }
}

impl Default for GroundUnit {
    fn default() -> Self {
        Self {
            ore: [0; MAX_ORE_TYPES],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ground {
    size_x: usize,
    size_y: usize,
    units: Vec<GroundUnit>,
}

impl Ground {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        assert!(size_x > 0);
        assert!(size_y > 0);

        Self {
            size_x,
            size_y,
            units: vec![GroundUnit::default(); size_x * size_y],
        }
    }

    pub fn size_x(&self) -> usize {
        self.size_x
    }

    pub fn size_y(&self) -> usize {
        self.size_y
    }

    pub fn at(&self, x: usize, y: usize) -> &GroundUnit {
        &self.units[self.index(x, y)]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut GroundUnit {
        let index = self.index(x, y);
        &mut self.units[index]
    }

    pub fn at_position(&self, position: Position) -> &GroundUnit {
        self.at(position.x as usize, position.y as usize)
    }

    pub fn at_position_mut(&mut self, position: Position) -> &mut GroundUnit {
        self.at_mut(position.x as usize, position.y as usize)
    }

    pub fn add_ore_heap(
        &mut self,
        center_x: usize,
        center_y: usize,
        ore_type: usize,
        top_amount: i32,
        radius: i32,
    ) {
        assert!(radius > 0);

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let x = center_x as i32 + dx;
                let y = center_y as i32 + dy;

                if x >= 0 && x < self.size_x as i32 && y >= 0 && y < self.size_y as i32 {
                    let distance = ((dx * dx + dy * dy) as f64).sqrt();
                    let amount = (0.5
                        + top_amount as f64 * (radius as f64 - distance) / radius as f64)
                        as i32;
                    let amount = amount - self.at(x as usize, y as usize).ore_at(ore_type);

                    if amount > 0 {
                        self.at_mut(x as usize, y as usize)
                            .add_ore(ore_type, amount);
                    }
                }
            }
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.size_x);
        assert!(y < self.size_y);

        x * self.size_y + y
    }

    pub fn scan_ore(
        &self,
        origin: Position,
        scan_direction: f64,
        max_distance: i32,
        ore_ids: &[i64],
    ) -> ScanResult {
        if max_distance <= 0 {
            return ScanResult::empty();
        }

        let world_angle =
            (origin.orientation as f64 + scan_direction) * std::f64::consts::PI / 180.0;
        let step = 0.25;
        let max_distance = f64::from(max_distance);
        let mut closest_distance = None;
        let mut closest_slot = 0_usize;
        let mut closest_ore_id = 0_i64;

        let mut distance = 0.0;
        while distance <= max_distance {
            let x = origin.x + distance * world_angle.cos();
            let y = origin.y + distance * world_angle.sin();

            if x < 0.0 || y < 0.0 || x >= self.size_x as f64 || y >= self.size_y as f64 {
                break;
            }

            let cell_x = x as usize;
            let cell_y = y as usize;
            let ground_unit = self.at(cell_x, cell_y);

            for (slot, amount) in ground_unit.ore().iter().enumerate() {
                if *amount <= 0 {
                    continue;
                }

                let ore_id = ore_ids.get(slot).copied().unwrap_or(0);
                let is_closer = closest_distance.is_none_or(|closest| distance < closest);
                let is_same_cell_richer = closest_distance.is_some_and(|closest| {
                    (distance - closest).abs() < f64::EPSILON && ore_id > closest_ore_id
                });

                if is_closer || is_same_cell_richer {
                    closest_distance = Some(distance);
                    closest_slot = slot;
                    closest_ore_id = ore_id;
                }
            }

            distance += step;
        }

        if let Some(distance) = closest_distance {
            ScanResult {
                distance,
                ore_type: (closest_slot + 1) as f64,
            }
        } else {
            ScanResult::empty()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScanResult {
    pub distance: f64,
    pub ore_type: f64,
}

impl ScanResult {
    pub fn empty() -> Self {
        Self {
            distance: -1.0,
            ore_type: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScanSnapshot {
    pub started: bool,
    pub complete: bool,
    pub distance: f64,
    pub ore_type: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum ScanState {
    #[default]
    Idle,
    Scanning {
        direction: f64,
        cycles_remaining: i32,
    },
    Complete(ScanResult),
}
