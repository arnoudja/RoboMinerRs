use super::ground::{LegacyHeapPlacement, legacy_heap_center, mining_area_to_ground};
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
                    assert!(
                        center >= radius,
                        "size={size} radius={radius} center={center}"
                    );
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
