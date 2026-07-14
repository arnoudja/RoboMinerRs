use crate::physics::{find_collision_time, position_at_time};

use super::Simulation;

impl Simulation {
    pub(super) fn check_collisions(&mut self) {
        let mut min_collision_time = 0.0;

        loop {
            let mut collision_time = 1.0;
            let mut collision_pair = None;

            for first in 0..self.robots.len().saturating_sub(1) {
                for second in (first + 1)..self.robots.len() {
                    let next_time = find_collision_time(
                        &self.robots[first],
                        &self.robots[second],
                        min_collision_time,
                        collision_time,
                    );

                    if next_time < collision_time {
                        collision_time = next_time;
                        collision_pair = Some((first, second));
                    }
                }
            }

            let Some((first, second)) = collision_pair else {
                break;
            };

            if collision_time >= 1.0 {
                break;
            }

            min_collision_time = collision_time;

            if self.robots[first].time_fraction > collision_time {
                self.robots[first].destination = position_at_time(
                    self.robots[first].position,
                    self.robots[first].destination,
                    self.robots[first].time_fraction,
                    collision_time,
                );
                self.robots[first].time_fraction = collision_time;
            }

            if self.robots[second].time_fraction > collision_time {
                self.robots[second].destination = position_at_time(
                    self.robots[second].position,
                    self.robots[second].destination,
                    self.robots[second].time_fraction,
                    collision_time,
                );
                self.robots[second].time_fraction = collision_time;
            }
        }
    }
}
