use crate::catalog::PartCatalog;
use rand::Rng;
use robominer_program::{
    ExecutableProgram, RngLike, compatibility_fixture_source, compile_executable_source,
    crossover_programs, mutate_program, seed_program_sources, unparse_program,
};

#[derive(Debug, Clone)]
pub struct Genome {
    pub part_ids: [i64; 7],
    pub program: ExecutableProgram,
}

impl Genome {
    pub fn random(catalog: &PartCatalog, rng: &mut impl Rng) -> Self {
        let part_ids = random_part_ids(catalog, rng);
        let program = random_program(rng);
        Self { part_ids, program }
    }

    /// Random parts with a fixed (already compiled) program.
    pub fn with_program(
        catalog: &PartCatalog,
        program: ExecutableProgram,
        rng: &mut impl Rng,
    ) -> Self {
        Self {
            part_ids: random_part_ids(catalog, rng),
            program,
        }
    }

    pub fn with_parts(part_ids: [i64; 7], program: ExecutableProgram) -> Self {
        Self { part_ids, program }
    }

    pub fn source_code(&self) -> String {
        unparse_program(&self.program)
    }

    pub fn crossover(&self, other: &Self, rng: &mut impl Rng) -> (Self, Self) {
        let mut left_parts = self.part_ids;
        let mut right_parts = other.part_ids;
        for slot in 0..7 {
            if rng.gen_bool(0.5) {
                std::mem::swap(&mut left_parts[slot], &mut right_parts[slot]);
            }
        }

        let mut adapter = RandAdapter(rng);
        let (left_program, right_program) =
            crossover_programs(&self.program, &other.program, &mut adapter)
                .unwrap_or_else(|| (self.program.clone(), other.program.clone()));

        (
            Self {
                part_ids: left_parts,
                program: left_program,
            },
            Self {
                part_ids: right_parts,
                program: right_program,
            },
        )
    }

    /// Crossover part slots only; both children keep `self.program`.
    pub fn crossover_parts(&self, other: &Self, rng: &mut impl Rng) -> (Self, Self) {
        let mut left_parts = self.part_ids;
        let mut right_parts = other.part_ids;
        for slot in 0..7 {
            if rng.gen_bool(0.5) {
                std::mem::swap(&mut left_parts[slot], &mut right_parts[slot]);
            }
        }
        (
            Self {
                part_ids: left_parts,
                program: self.program.clone(),
            },
            Self {
                part_ids: right_parts,
                program: self.program.clone(),
            },
        )
    }

    pub fn mutate(&self, catalog: &PartCatalog, rng: &mut impl Rng) -> Self {
        let mut part_ids = self.part_ids;
        let slot = rng.gen_range(0..7);
        let type_id = (slot as i64) + 1;
        if let Some(parts) = catalog.parts_for_type(type_id)
            && !parts.is_empty()
        {
            part_ids[slot] = parts[rng.gen_range(0..parts.len())].id;
        }

        let mut adapter = RandAdapter(rng);
        let program = mutate_program(&self.program, &mut adapter);
        Self { part_ids, program }
    }

    /// Mutate one part slot; keep the program unchanged.
    pub fn mutate_parts(&self, catalog: &PartCatalog, rng: &mut impl Rng) -> Self {
        let mut part_ids = self.part_ids;
        let slot = rng.gen_range(0..7);
        let type_id = (slot as i64) + 1;
        if let Some(parts) = catalog.parts_for_type(type_id)
            && !parts.is_empty()
        {
            part_ids[slot] = parts[rng.gen_range(0..parts.len())].id;
        }
        Self {
            part_ids,
            program: self.program.clone(),
        }
    }

    /// Mutate the program only; keep part ids unchanged.
    pub fn mutate_program_only(&self, rng: &mut impl Rng) -> Self {
        let mut adapter = RandAdapter(rng);
        let program = mutate_program(&self.program, &mut adapter);
        Self {
            part_ids: self.part_ids,
            program,
        }
    }

    /// Crossover programs only; both children keep `self.part_ids`.
    pub fn crossover_programs_only(&self, other: &Self, rng: &mut impl Rng) -> (Self, Self) {
        let mut adapter = RandAdapter(rng);
        let (left_program, right_program) =
            crossover_programs(&self.program, &other.program, &mut adapter)
                .unwrap_or_else(|| (self.program.clone(), other.program.clone()));
        (
            Self {
                part_ids: self.part_ids,
                program: left_program,
            },
            Self {
                part_ids: self.part_ids,
                program: right_program,
            },
        )
    }
}

fn random_part_ids(catalog: &PartCatalog, rng: &mut impl Rng) -> [i64; 7] {
    let mut ids = [0; 7];
    for (slot, id) in ids.iter_mut().enumerate() {
        let type_id = (slot as i64) + 1;
        let parts = catalog
            .parts_for_type(type_id)
            .expect("catalog complete for types 1-7");
        *id = parts[rng.gen_range(0..parts.len())].id;
    }
    ids
}

fn random_program(rng: &mut impl Rng) -> ExecutableProgram {
    let mut sources = seed_program_sources();
    sources.push(compatibility_fixture_source("default_program"));
    sources.push(compatibility_fixture_source("seed_ai_1"));
    sources.push(compatibility_fixture_source("seed_ai_2"));
    sources.push(compatibility_fixture_source("scan_then_mine"));
    let source = sources[rng.gen_range(0..sources.len())];
    compile_executable_source(source).expect("seed program compiles")
}

struct RandAdapter<'a, R>(&'a mut R);

impl<R: Rng> RngLike for RandAdapter<'_, R> {
    fn gen_range(&mut self, low: usize, high: usize) -> usize {
        if high <= low {
            low
        } else {
            self.0.gen_range(low..high)
        }
    }

    fn gen_f64(&mut self) -> f64 {
        self.0.r#gen()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robominer_db::RobotPartRecord;
    use robominer_program::RngLike;

    fn sample_part(id: i64, type_id: i64, tier_id: i64) -> RobotPartRecord {
        RobotPartRecord {
            id,
            type_id,
            tier_id: Some(tier_id),
            part_name: format!("part-{id}"),
            ore_price_id: 1,
            ore_capacity: 2,
            mining_capacity: 2,
            battery_capacity: 20,
            memory_capacity: 50,
            cpu_capacity: 5,
            forward_capacity: 6,
            backward_capacity: 3,
            rotate_capacity: 2,
            recharge_time: 1,
            scan_time: 1,
            scan_distance: 1,
            weight: 2,
            volume: 8,
            power_usage: 1,
        }
    }

    fn dual_catalog() -> PartCatalog {
        let parts = (1..=7)
            .flat_map(|type_id| {
                [
                    sample_part(type_id * 10, type_id, 1),
                    sample_part(type_id * 10 + 1, type_id, 1),
                ]
            })
            .collect();
        PartCatalog::from_parts(parts, 9)
    }

    #[test]
    fn random_genome_respects_max_tier_id_filter() {
        let parts = (1..=7)
            .flat_map(|type_id| {
                [
                    sample_part(type_id * 10, type_id, 1),
                    sample_part(type_id * 10 + 5, type_id, 2),
                    sample_part(9000 + type_id, type_id, 9),
                ]
            })
            .collect();
        let catalog = PartCatalog::from_parts(parts, 2);
        assert!(catalog.is_complete());
        for type_id in 1..=7 {
            let available = catalog.parts_for_type(type_id).unwrap();
            assert!(
                available
                    .iter()
                    .all(|part| part.tier_id.is_some_and(|tier| tier <= 2))
            );
            assert!(!available.iter().any(|part| part.id >= 9000));
        }
        let mut rng = rand::thread_rng();
        let genome = Genome::random(&catalog, &mut rng);
        for &part_id in &genome.part_ids {
            let part = catalog.get(part_id).expect("part in catalog");
            assert!(part.tier_id.is_some_and(|tier| tier <= 2));
        }
    }

    #[test]
    fn with_parts_source_code_and_operators() {
        let catalog = dual_catalog();
        let left_program = compile_executable_source("mine();").expect("compile");
        let right_program = compile_executable_source("dump();").expect("compile");
        let left = Genome::with_parts([10, 20, 30, 40, 50, 60, 70], left_program.clone());
        let right = Genome::with_parts([11, 21, 31, 41, 51, 61, 71], right_program);
        assert_eq!(left.source_code(), "mine();");

        let mut rng = rand::thread_rng();
        let (c_a, c_b) = left.crossover(&right, &mut rng);
        assert_eq!(c_a.part_ids.len(), 7);
        assert_eq!(c_b.part_ids.len(), 7);

        let (p_a, p_b) = left.crossover_programs_only(&right, &mut rng);
        assert_eq!(p_a.part_ids, left.part_ids);
        assert_eq!(p_b.part_ids, left.part_ids);

        let mutated = left.mutate(&catalog, &mut rng);
        assert!(catalog.get(mutated.part_ids[0]).is_some());

        let program_only = left.mutate_program_only(&mut rng);
        assert_eq!(program_only.part_ids, left.part_ids);

        let with_program = Genome::with_program(&catalog, left_program, &mut rng);
        assert!(with_program.part_ids.iter().all(|&id| catalog.get(id).is_some()));
    }

    #[test]
    fn rand_adapter_handles_empty_range() {
        let mut rng = rand::thread_rng();
        let mut adapter = RandAdapter(&mut rng);
        assert_eq!(adapter.gen_range(5, 5), 5);
        assert_eq!(adapter.gen_range(3, 1), 3);
        let _ = adapter.gen_f64();
    }
}
