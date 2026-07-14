/// Signed east/west and north/south distance from the rally start position on the map.
///
/// Positive `xPos` is east of the start; positive `yPos` is north of the start.
pub fn rally_map_position(
    center_x: f64,
    center_y: f64,
    initial_center_x: f64,
    initial_center_y: f64,
) -> (f64, f64) {
    (center_x - initial_center_x, center_y - initial_center_y)
}

/// Rally-relative robot pose for program-visible `robot.*` position properties.
///
/// Position uses map-fixed east/west (`xPos`) and north/south (`yPos`) distance from the
/// robot's initial center at rally start. Orientation is always `135` at start.
pub fn rally_robot_pose(
    center_x: f64,
    center_y: f64,
    orientation_deg: i32,
    initial_center_x: f64,
    initial_center_y: f64,
    initial_orientation_deg: i32,
) -> (f64, f64, f64) {
    let (x_pos, y_pos) = rally_map_position(center_x, center_y, initial_center_x, initial_center_y);
    let orientation = (135 + (orientation_deg - initial_orientation_deg)).rem_euclid(360) as f64;
    (x_pos, y_pos, orientation)
}
