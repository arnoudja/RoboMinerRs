use crate::catalog::PartCatalog;
use crate::ga::RankedIndividual;

pub fn format_top_results(
    ranked: &[RankedIndividual],
    catalog: &PartCatalog,
    depot_capacity: i32,
    top_n: usize,
) -> String {
    let mut out = String::new();
    let limit = top_n.max(1);
    for (index, individual) in ranked.iter().take(limit).enumerate() {
        if !individual.fitness.fitness.is_finite() {
            continue;
        }
        out.push_str(&format!(
            "=== #{} fitness={:.4} (avg after tax) ===\n",
            index + 1,
            individual.fitness.fitness
        ));
        out.push_str("Parts:\n");
        for (slot, part_id) in individual.genome.part_ids.iter().enumerate() {
            let type_id = slot as i64 + 1;
            let name = catalog
                .get(*part_id)
                .map(|part| part.part_name.as_str())
                .unwrap_or("?");
            out.push_str(&format!(
                "  {}: {name} (id={part_id})\n",
                PartCatalog::type_name(type_id)
            ));
        }
        out.push_str(&format!(
            "Depot A capacity: {depot_capacity} (fixed for this run)\n"
        ));
        if let Some(parameters) = individual.fitness.parameters {
            out.push_str(&format!(
                "Stats: maxOre={} miningSpeed={} maxTurns={} memory={} cpu={} forward={:.3} backward={:.3} rotate={} size={:.3} scanTime={} scanDistance={}\n",
                parameters.max_ore,
                parameters.mining_speed,
                parameters.max_turns,
                parameters.memory_size,
                parameters.cpu_speed,
                parameters.forward_speed,
                parameters.backward_speed,
                parameters.rotate_speed,
                parameters.robot_size,
                parameters.scan_time,
                parameters.scan_distance,
            ));
        }
        if !individual.fitness.per_area.is_empty() {
            out.push_str("Per-area:");
            for (area_id, score) in &individual.fitness.per_area {
                out.push_str(&format!(" {area_id}={score:.4}"));
            }
            out.push('\n');
        }
        out.push_str("----- program (paste into Edit code) -----\n");
        out.push_str(&individual.fitness.source_code);
        out.push('\n');
        out.push_str("----- end program -----\n\n");
    }
    if out.is_empty() {
        out.push_str("No valid individuals found.\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::PartCatalog;
    use crate::fitness::FitnessResult;
    use crate::ga::RankedIndividual;
    use crate::genome::Genome;
    use robominer_db::{RobotParameters, RobotPartRecord};
    use robominer_program::compile_executable_source;

    fn sample_part(id: i64, type_id: i64) -> RobotPartRecord {
        RobotPartRecord {
            id,
            type_id,
            tier_id: Some(1),
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

    fn sample_catalog() -> PartCatalog {
        PartCatalog::from_parts(
            (1..=7)
                .map(|type_id| sample_part(type_id * 10, type_id))
                .collect(),
            9,
        )
    }

    fn sample_parameters() -> RobotParameters {
        RobotParameters {
            recharge_time: 1,
            max_ore: 10,
            mining_speed: 4,
            max_turns: 20,
            memory_size: 50,
            cpu_speed: 5,
            forward_speed: 1.5,
            backward_speed: 0.75,
            rotate_speed: 10,
            robot_size: 0.5,
            scan_time: 1,
            scan_distance: 3,
        }
    }

    fn ranked(
        part_ids: [i64; 7],
        source: &str,
        fitness: f64,
        with_details: bool,
    ) -> RankedIndividual {
        let program = compile_executable_source(source).expect("compile");
        RankedIndividual {
            genome: Genome::with_parts(part_ids, program),
            fitness: FitnessResult {
                fitness,
                per_area: if with_details {
                    vec![(1001, 1.25), (1002, 0.5)]
                } else {
                    Vec::new()
                },
                parameters: if with_details {
                    Some(sample_parameters())
                } else {
                    None
                },
                source_code: source.to_string(),
                compiled_size: Some(8),
            },
        }
    }

    #[test]
    fn format_top_results_reports_no_valid_individuals() {
        let catalog = sample_catalog();
        let ranked = vec![ranked(
            [10, 20, 30, 40, 50, 60, 70],
            "mine();",
            f64::NEG_INFINITY,
            false,
        )];
        let report = format_top_results(&ranked, &catalog, 40, 3);
        assert_eq!(report, "No valid individuals found.\n");
    }

    #[test]
    fn format_top_results_includes_parts_stats_areas_and_program() {
        let catalog = sample_catalog();
        let ranked = vec![
            ranked([10, 20, 30, 40, 50, 60, 70], "mine(); dump();", 3.5, true),
            ranked([10, 20, 30, 40, 50, 60, 70], "rotate(90);", 2.0, false),
        ];
        let report = format_top_results(&ranked, &catalog, 40, 2);
        assert!(report.contains("=== #1 fitness=3.5000 (avg after tax) ==="));
        assert!(report.contains("Ore container: part-10 (id=10)"));
        assert!(report.contains("Ore scanner: part-70 (id=70)"));
        assert!(report.contains("Depot A capacity: 40 (fixed for this run)"));
        assert!(report.contains("Stats: maxOre=10 miningSpeed=4"));
        assert!(report.contains("Per-area: 1001=1.2500 1002=0.5000"));
        assert!(report.contains("----- program (paste into Edit code) -----"));
        assert!(report.contains("mine(); dump();"));
        assert!(report.contains("----- end program -----"));
        assert!(report.contains("=== #2 fitness=2.0000"));
        assert!(report.contains("rotate(90);"));
    }

    #[test]
    fn format_top_results_skips_non_finite_before_valid() {
        let catalog = sample_catalog();
        let ranked = vec![
            ranked([10, 20, 30, 40, 50, 60, 70], "mine();", f64::NAN, false),
            ranked([10, 20, 30, 40, 50, 60, 70], "dump();", 1.0, false),
        ];
        let report = format_top_results(&ranked, &catalog, 10, 5);
        assert!(report.contains("=== #2 fitness=1.0000"));
        assert!(!report.contains("=== #1"));
        assert!(report.contains("dump();"));
    }

    #[test]
    fn format_top_results_uses_question_mark_for_unknown_part() {
        let catalog = sample_catalog();
        let ranked = vec![ranked(
            [999, 20, 30, 40, 50, 60, 70],
            "mine();",
            1.0,
            false,
        )];
        let report = format_top_results(&ranked, &catalog, 1, 1);
        assert!(report.contains("Ore container: ? (id=999)"));
    }
}
