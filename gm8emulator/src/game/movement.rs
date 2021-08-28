use crate::{
    game::{Game, GetAsset},
    math::Real,
    util,
};

impl Game {
    /// Processes movement (friction, gravity, speed/direction) for all instances
    pub fn process_speeds(&mut self) {
        let mut iter = self.room.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.room.instance_list).map(|i| self.room.instance_list.get(i)) {
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
        }
    }

    // Returns true if path end event should be called
    pub fn apply_speeds(&self, handle: usize) -> bool {
        let instance = self.room.instance_list.get(handle);
        // Apply hspeed and vspeed to x and y
        let hspeed = instance.hspeed.get();
        let vspeed = instance.vspeed.get();
        if hspeed != Real::from(0.0) || vspeed != Real::from(0.0) {
            instance.x.set(instance.x.get() + hspeed);
            instance.y.set(instance.y.get() + vspeed);
            instance.bbox_is_stale.set(true);
        }

        // Advance paths
        let mut run_event = false;
        if let Some(path) = self.assets.paths.get_asset(instance.path_index.get()) {
            if path.length != 0.into() && instance.path_speed.get() != 0.into() {
                // Prepare this for later
                let angle = instance.path_orientation.get().to_radians();

                // Calculate how much offset (0-1) we want to add to the instance's path position
                let point_speed = path.get_point(instance.path_position.get()).speed;
                let offset = instance.path_speed.get() * (point_speed / Real::from(100.0))
                    / (path.length * instance.path_scale.get());

                // Work out what the new position should be
                let new_position = instance.path_position.get() + offset;
                if new_position <= Real::from(0.0) || new_position >= Real::from(1.0) {
                    // Path end
                    let reversed = new_position < Real::from(0.0);
                    let opposite_position =
                        if reversed { new_position + Real::from(1.0) } else { new_position - Real::from(1.0) };
                    match instance.path_endaction.get() {
                        1 => {
                            // Continue from start
                            instance.path_position.set(opposite_position);
                        },
                        2 => {
                            // Continue from end
                            let path_start_pos = if reversed { Real::from(1.0) } else { Real::from(0.0) };
                            let path_end_pos = if reversed { Real::from(0.0) } else { Real::from(1.0) };

                            instance.path_position.set(opposite_position);
                            let start_point = path.get_point(path_start_pos);
                            let end_point = path.get_point(path_end_pos);
                            let mut size_h = end_point.x - start_point.x;
                            let mut size_v = end_point.y - start_point.y;
                            util::rotate_around_center(
                                size_h.as_mut_ref(),
                                size_v.as_mut_ref(),
                                angle.sin().into(),
                                angle.cos().into(),
                            );
                            instance.path_xstart.set(instance.path_xstart.get() + size_h * instance.path_scale.get());
                            instance.path_ystart.set(instance.path_ystart.get() + size_v * instance.path_scale.get());
                        },
                        3 => {
                            // Reverse
                            instance.path_position.set(Real::from(1.0) - (opposite_position));
                            instance.path_speed.set(if reversed {
                                instance.path_speed.get().abs()
                            } else {
                                -instance.path_speed.get().abs()
                            });
                        },
                        _ => {
                            // Stop
                            instance.path_position.set(1.into());
                            instance.path_index.set(-1);
                        },
                    }

                    // Set flag to run path end event
                    run_event = true;
                } else {
                    // Normally update path_position
                    instance.path_position.set(new_position);
                }

                // Figure out the new coordinates for this instance based on its path_position and path vars
                let mut point = path.get_point(instance.path_position.get());
                point.x -= path.start.x;
                point.y -= path.start.y;
                point.x *= instance.path_scale.get();
                point.y *= instance.path_scale.get();
                util::rotate_around_center(
                    point.x.as_mut_ref(),
                    point.y.as_mut_ref(),
                    angle.sin().into(),
                    angle.cos().into(),
                );

                // Update the instance's x, y and direction
                let new_x = point.x + instance.path_xstart.get();
                let new_y = point.y + instance.path_ystart.get();
                instance.set_direction((instance.y.get() - new_y).arctan2(new_x - instance.x.get()).to_degrees());
                instance.set_speed(0.into());
                instance.x.set(new_x);
                instance.y.set(new_y);
                instance.bbox_is_stale.set(true);
            }
        }

        // Run path end event
        run_event
    }

    /// "bounces" the instance against any instances or only solid ones, depending on solid_only
    pub fn bounce(&self, handle: usize, solids_only: bool) {
        let instance = self.room.instance_list.get(handle);
        let collider = if solids_only { Game::check_collision_solid } else { Game::check_collision_any };

        if collider(self, handle).is_some() {
            instance.x.set(instance.xprevious.get());
            instance.y.set(instance.yprevious.get());
        }

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
        let instance = self.room.instance_list.get(handle);
        let collider = if solids_only { Game::check_collision_solid } else { Game::check_collision_any };

        let mut bounce = false;

        if collider(self, handle).is_some() {
            instance.x.set(instance.xprevious.get());
            instance.y.set(instance.yprevious.get());
            bounce = true;
        }

        let old_x = instance.x.get();
        let old_y = instance.y.get();

        let start_angle = Real::from((instance.direction.get() / Real::from(10.0)).round().to_i32() * 10);

        let mut get_side_collision_angle = |angle_step: i32| {
            let angle_step = Real::from(angle_step);

            let mut side_angle = start_angle;

            for _ in 0..36 {
                side_angle += angle_step;
                instance.x.set(old_x + instance.speed.get() * side_angle.to_radians().cos());
                instance.y.set(old_y - instance.speed.get() * side_angle.to_radians().sin());
                instance.bbox_is_stale.set(true);
                if collider(self, handle).is_none() {
                    break
                }
                bounce = true;
            }
            side_angle
        };

        let cw = get_side_collision_angle(-10);
        let ccw = get_side_collision_angle(10);
        if bounce {
            instance.set_direction(cw + ccw + Real::from(180.0) - start_angle);
        }

        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);
    }
}
