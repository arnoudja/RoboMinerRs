use anyhow::{Context, Result, anyhow};
use robominer_db::{MySqlPool, RobotPartRecord, list_robot_parts};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PartCatalog {
    by_type: HashMap<i64, Vec<RobotPartRecord>>,
    by_id: HashMap<i64, RobotPartRecord>,
}

fn part_within_max_tier(part: &RobotPartRecord, max_tier_id: i64) -> bool {
    part.tier_id.is_some_and(|tier_id| tier_id <= max_tier_id)
}

impl PartCatalog {
    pub async fn load(pool: &MySqlPool, max_tier_id: i64) -> Result<Self> {
        let parts = list_robot_parts(pool)
            .await
            .context("list_robot_parts")?
            .into_iter()
            .filter(|part| part_within_max_tier(part, max_tier_id))
            .collect::<Vec<_>>();

        Ok(Self::from_filtered_parts(parts))
    }

    pub fn from_parts(parts: Vec<RobotPartRecord>, max_tier_id: i64) -> Self {
        Self::from_filtered_parts(
            parts
                .into_iter()
                .filter(|part| part_within_max_tier(part, max_tier_id))
                .collect(),
        )
    }

    fn from_filtered_parts(parts: Vec<RobotPartRecord>) -> Self {
        let mut by_type: HashMap<i64, Vec<RobotPartRecord>> = HashMap::new();
        let mut by_id = HashMap::new();
        for part in parts {
            by_id.insert(part.id, part.clone());
            by_type.entry(part.type_id).or_default().push(part);
        }
        Self { by_type, by_id }
    }

    pub fn is_complete(&self) -> bool {
        (1..=7).all(|type_id| {
            self.parts_for_type(type_id)
                .is_some_and(|parts| !parts.is_empty())
        })
    }

    pub fn parts_for_type(&self, type_id: i64) -> Option<&[RobotPartRecord]> {
        self.by_type.get(&type_id).map(Vec::as_slice)
    }

    pub fn get(&self, part_id: i64) -> Option<&RobotPartRecord> {
        self.by_id.get(&part_id)
    }

    pub fn resolve_parts(
        &self,
        part_ids: &[i64; 7],
    ) -> Result<robominer_db::RequestedRobotParts, anyhow::Error> {
        let pick = |index: usize, type_id: i64| -> Result<RobotPartRecord> {
            let id = part_ids[index];
            let part = self
                .get(id)
                .ok_or_else(|| anyhow!("unknown part id {id}"))?
                .clone();
            if part.type_id != type_id {
                return Err(anyhow!(
                    "part {id} has type {} but slot expects {type_id}",
                    part.type_id
                ));
            }
            Ok(part)
        };

        Ok(robominer_db::RequestedRobotParts {
            ore_container: pick(0, 1)?,
            mining_unit: pick(1, 2)?,
            battery: pick(2, 3)?,
            memory_module: pick(3, 4)?,
            cpu: pick(4, 5)?,
            engine: pick(5, 6)?,
            ore_scanner: pick(6, 7)?,
        })
    }

    pub fn type_name(type_id: i64) -> &'static str {
        match type_id {
            1 => "Ore container",
            2 => "Mining unit",
            3 => "Battery",
            4 => "Memory module",
            5 => "CPU",
            6 => "Engine",
            7 => "Ore scanner",
            _ => "Unknown",
        }
    }
}
