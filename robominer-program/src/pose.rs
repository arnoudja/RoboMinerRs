//! Rally-relative pose for program-visible `robot.xPos` / `robot.yPos` / `robot.orientation`.

/// Map-fixed east/west and north/south delta from the rally start center.
///
/// Positive `x` is east of the start; positive `y` is north of the start.
/// Prefer [`rally_robot_pose`] for values exposed to robot programs.
pub fn rally_map_position(
    center_x: f64,
    center_y: f64,
    initial_center_x: f64,
    initial_center_y: f64,
) -> (f64, f64) {
    (center_x - initial_center_x, center_y - initial_center_y)
}

/// World facing of robot slot 0 at spawn. Local `xPos`/`yPos` keep the same signs for every
/// corner as this SW spawn does when moving into the map.
const REFERENCE_SPAWN_ORIENTATION_DEG: i32 = 45;

/// Rally-relative robot pose for program-visible `robot.*` position properties.
///
/// At every spawn corner, `xPos` and `yPos` start at 0 and `orientation` starts at 135.
/// Position axes are rotated into the spawn's local frame so moving into the map increases
/// both coordinates the same way in every corner (matching robot slot 0). Orientation is
/// remapped the same way: program heading 270 reduces `xPos`, heading 0 reduces `yPos`.
pub fn rally_robot_pose(
    center_x: f64,
    center_y: f64,
    orientation_deg: i32,
    initial_center_x: f64,
    initial_center_y: f64,
    initial_orientation_deg: i32,
) -> (f64, f64, f64) {
    let (map_x, map_y) = rally_map_position(center_x, center_y, initial_center_x, initial_center_y);
    let (x_pos, y_pos) = spawn_relative_position(map_x, map_y, initial_orientation_deg);
    let orientation = (135 + (orientation_deg - initial_orientation_deg)).rem_euclid(360) as f64;
    (x_pos, y_pos, orientation)
}

fn spawn_relative_position(map_x: f64, map_y: f64, initial_orientation_deg: i32) -> (f64, f64) {
    let alpha = (initial_orientation_deg - REFERENCE_SPAWN_ORIENTATION_DEG).rem_euclid(360);
    let (cos, sin) = orientation_trigonometry(alpha);
    // Rotate map delta by -alpha into the reference (SW) spawn frame.
    (map_x * cos + map_y * sin, -map_x * sin + map_y * cos)
}

fn orientation_trigonometry(orientation_deg: i32) -> (f64, f64) {
    match orientation_deg.rem_euclid(360) {
        0 => (1.0, 0.0),
        90 => (0.0, 1.0),
        180 => (-1.0, 0.0),
        270 => (0.0, -1.0),
        orientation => {
            let radians = f64::from(orientation) * std::f64::consts::PI / 180.0;
            (radians.cos(), radians.sin())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_relative_keeps_sw_identity() {
        let (x, y) = spawn_relative_position(2.0, 3.0, 45);
        assert!((x - 2.0).abs() < f64::EPSILON);
        assert!((y - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spawn_relative_se_maps_into_map_to_positive_axes() {
        // SE spawn faces 135° into the map (-x, +y on the map).
        let (x, y) = spawn_relative_position(-2.0, 3.0, 135);
        assert!((x - 3.0).abs() < f64::EPSILON);
        assert!((y - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spawn_relative_nw_maps_into_map_to_positive_axes() {
        // NW spawn faces 315° into the map (+x, -y on the map).
        let (x, y) = spawn_relative_position(2.0, -3.0, 315);
        assert!((x - 3.0).abs() < f64::EPSILON);
        assert!((y - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spawn_relative_ne_maps_into_map_to_positive_axes() {
        // NE spawn faces 225° into the map (-x, -y on the map).
        let (x, y) = spawn_relative_position(-2.0, -3.0, 225);
        assert!((x - 2.0).abs() < f64::EPSILON);
        assert!((y - 3.0).abs() < f64::EPSILON);
    }
}
