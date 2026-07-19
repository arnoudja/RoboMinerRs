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
