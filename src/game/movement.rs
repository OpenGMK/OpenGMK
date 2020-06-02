use crate::{game::Game, math::Real};

impl Game {
    /// Processes movement (friction, gravity, speed/direction) for all instances
    pub fn process_movement(&mut self) {
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list).map(|i| self.instance_list.get(i)) {
            let friction = instance.friction.get();
            if friction != Real::from(0.0) {
                // "Subtract" friction from speed towards 0
                let speed = instance.speed.get();
                if speed >= Real::from(0.0) {
                    if friction > speed {
                        instance.set_speed(Real::from(0.0));
                    } else {
                        instance.set_speed(speed - friction);
                    }
                } else {
                    if friction > -speed {
                        instance.set_speed(Real::from(0.0));
                    } else {
                        instance.set_speed(speed + friction);
                    }
                }
            }

            let gravity = instance.gravity.get();
            if gravity != Real::from(0.0) {
                // Apply gravity in gravity_direction to hspeed and vspeed
                let gravity_direction = instance.gravity_direction.get().to_radians();
                instance.set_hvspeed(
                    instance.hspeed.get() + (gravity_direction.cos() * gravity),
                    instance.vspeed.get() - (gravity_direction.sin() * gravity),
                );
            }

            // Apply hspeed and vspeed to x and y
            let hspeed = instance.hspeed.get();
            let vspeed = instance.vspeed.get();
            if hspeed != Real::from(0.0) || vspeed != Real::from(0.0) {
                instance.x.set(instance.x.get() + hspeed);
                instance.y.set(instance.y.get() + vspeed);
                instance.bbox_is_stale.set(true);
            }
        }
    }

    /// "bounces" the instance against any instances or only solid ones, depending on solid_only
    pub fn bounce(&self, handle: usize, solids_only: bool) {
        let instance = self.instance_list.get(handle);
        let collider = if solids_only { Game::check_collision_solid } else { Game::check_collision_any };

        let old_x = instance.x.get();
        let old_y = instance.y.get();

        // Check collision in x axis
        instance.x.set(old_x + instance.hspeed.get());
        instance.bbox_is_stale.set(true);
        let x_bounce = collider(self, handle).is_some();

        // Check collision in y axis
        instance.x.set(old_x);
        instance.y.set(old_y + instance.vspeed.get());
        instance.bbox_is_stale.set(true);
        let y_bounce = collider(self, handle).is_some();

        // Update direction
        if x_bounce {
            instance.set_hspeed(-instance.hspeed.get());
        }
        if y_bounce {
            instance.set_vspeed(-instance.vspeed.get());
        }

        // If neither check passed, check collision after applying both offsets
        if !x_bounce && !y_bounce {
            instance.x.set(old_x + instance.hspeed.get());
            instance.bbox_is_stale.set(true);
            if collider(self, handle).is_some() {
                instance.set_hvspeed(-instance.hspeed.get(), -instance.vspeed.get());
            }
        }

        // Finally, set x and y back to normal
        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);
    }

    /// "bounces" the instance against any instances or only solid ones, depending on solid_only
    /// Uses GM8's "advanced bouncing" algorithm which is very broken
    pub fn bounce_advanced(&self, handle: usize, solids_only: bool) {
        let instance = self.instance_list.get(handle);
        let collider = if solids_only { Game::check_collision_solid } else { Game::check_collision_any };

        let old_x = instance.x.get();
        let old_y = instance.y.get();

        let mut cw = (instance.direction.get() / Real::from(10.0)).round() * 10;
        for _ in 0..36 {
            cw -= 10;
            instance.x.set(instance.x.get() + instance.speed.get() * Real::from(cw).to_radians().cos());
            instance.y.set(instance.y.get() + instance.speed.get() * Real::from(cw).to_radians().cos());
            instance.bbox_is_stale.set(true);
            if collider(self, handle).is_some() {
                break
            }
        }

        let mut ccw = (instance.direction.get() / Real::from(10.0)).round() * 10;
        for _ in 0..36 {
            ccw += 10;
            instance.x.set(instance.x.get() + instance.speed.get() * Real::from(ccw).to_radians().cos());
            instance.y.set(instance.y.get() + instance.speed.get() * Real::from(ccw).to_radians().cos());
            instance.bbox_is_stale.set(true);
            if collider(self, handle).is_some() {
                break
            }
        }

        instance.set_direction(Real::from(cw) + Real::from(ccw) + Real::from(180.0) - instance.direction.get());
        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);
    }
}
