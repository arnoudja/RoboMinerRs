#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub orientation: i32,
}

impl Position {
    pub fn new(x: f64, y: f64, orientation: i32) -> Self {
        Self { x, y, orientation }
    }

    pub fn rotate(&mut self, rotation: i32) {
        self.orientation = (self.orientation + rotation).rem_euclid(360);
    }

    pub fn calculate_move_position(&self, speed: f64) -> Self {
        let (cos, sin) = orientation_trigonometry(self.orientation);

        Self {
            x: self.x + speed * cos,
            y: self.y + speed * sin,
            orientation: self.orientation,
        }
    }

    pub fn distance(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

pub(crate) fn orientation_trigonometry(orientation_deg: i32) -> (f64, f64) {
    match orientation_deg.rem_euclid(360) {
        0 => (1.0, 0.0),
        90 => (0.0, 1.0),
        180 => (-1.0, 0.0),
        270 => (0.0, -1.0),
        orientation => {
            let radians = orientation as f64 * std::f64::consts::PI / 180.0;
            (radians.cos(), radians.sin())
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0)
    }
}
