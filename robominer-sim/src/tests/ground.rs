use crate::*;

#[test]
fn scan_detects_closest_ore_along_direction() {
    let mut ground = Ground::new(10, 10);
    ground.at_mut(5, 5).add_ore(0, 20);

    let ore_ids = vec![1_i64];
    let origin = Position::new(2.0, 5.0, 0);
    assert_eq!(ground.scan_ore(origin, 0.0, 10, &ore_ids).ore_type, 1.0);

    let mut ground = Ground::new(10, 10);
    ground.at_mut(5, 2).add_ore(0, 20);
    let origin = Position::new(5.0, 5.0, 0);
    assert_eq!(ground.scan_ore(origin, 90.0, 10, &ore_ids).ore_type, 0.0);
    assert_eq!(ground.scan_ore(origin, -90.0, 10, &ore_ids).ore_type, 1.0);
}

#[test]
fn scan_returns_higher_quality_index_at_same_distance() {
    let mut ground = Ground::new(10, 10);
    ground.at_mut(4, 5).add_ore(0, 10);
    ground.at_mut(4, 5).add_ore(1, 10);

    let ore_ids = vec![1_i64, 3_i64];
    let origin = Position::new(2.0, 5.0, 0);
    assert_eq!(
        ground.scan_ore(origin, 0.0, 10, &ore_ids).ore_type,
        2.0
    );
}

#[test]
fn scan_respects_max_distance() {
    let mut ground = Ground::new(20, 20);
    ground.at_mut(15, 10).add_ore(0, 50);

    let ore_ids = vec![5_i64];
    let origin = Position::new(5.0, 10.0, 0);
    assert_eq!(ground.scan_ore(origin, 0.0, 5, &ore_ids).ore_type, 0.0);
    assert_eq!(ground.scan_ore(origin, 0.0, 12, &ore_ids).ore_type, 1.0);
}

