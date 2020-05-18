use crate::{
    game::{Game, GetAsset},
    gml,
    types::ID,
};

impl Game {
    /// Runs an event for all objects which hold the given event.
    /// If no "other" instance is provided, "self" will be used as "other". This is what GM8 tends to do.
    pub fn run_object_event(&mut self, event_id: usize, event_sub: u32, other: Option<usize>) -> gml::Result<()> {
        let holders = match self.event_holders.get(event_id).and_then(|x| x.get(&event_sub)) {
            Some(e) => e.clone(),
            None => return Ok(()),
        };
        let mut position = 0;
        while let Some(&object_id) = holders.borrow().get(position) {
            let mut iter = self.instance_list.iter_by_object(object_id);
            while let Some(instance) = iter.next(&self.instance_list) {
                self.run_instance_event(event_id, event_sub, instance, other.unwrap_or(instance), None)?;
            }
            position += 1;
        }
        Ok(())
    }

    /// Runs an event for a given instance. Does nothing if that instance doesn't have the specified event.
    pub fn run_instance_event(
        &mut self,
        event_id: usize,
        mut event_sub: u32,
        instance: usize,
        other: usize,
        as_object: Option<ID>,
    ) -> gml::Result<()> {
        // Running instance events is not allowed if a room change is pending. This appears to be
        // how GM8 is implemented as well, given the related room creation bug and collision/solid bugs.
        if self.room_target.is_none() {
            let original_object_id = if let Some(id) = as_object {
                id
            } else {
                self.instance_list.get(instance).ok_or(gml::Error::InvalidInstanceHandle(instance))?.object_index.get()
            };
            let mut object_id = original_object_id;
            let event = loop {
                if object_id < 0 {
                    if event_id == gml::ev::COLLISION {
                        // For collision events, we need to check the target's parent tree too..
                        if let Some(target_object) = self.assets.objects.get_asset(event_sub as _) {
                            if target_object.parent_index < 0 {
                                return Ok(())
                            } else {
                                object_id = original_object_id;
                                event_sub = target_object.parent_index as u32;
                            }
                        }
                    } else {
                        return Ok(())
                    }
                }
                if let Some(object) = self.assets.objects.get_asset(object_id) {
                    if let Some(event) = object.events.get(event_id).and_then(|x| x.get(&event_sub)) {
                        break event.clone()
                    } else {
                        object_id = object.parent_index;
                    }
                } else {
                    return Ok(())
                }
            };

            self.execute_tree(event, instance, other, event_id, event_sub as _, object_id)
        } else {
            Ok(())
        }
    }

    /// Decrements all active alarms and runs subsequent alarm events for all instances
    pub fn run_alarms(&mut self) -> gml::Result<()> {
        for alarm_id in 0..=11 {
            // Get all the objects which have this alarm event registered
            if let Some(objects) = self.event_holders[gml::ev::ALARMS].get(&alarm_id).map(|x| x.clone()) {
                for object_id in objects.borrow().iter().copied() {
                    // Iter all instances of this object
                    let mut iter = self.instance_list.iter_by_object(object_id);
                    while let Some(handle) = iter.next(&self.instance_list) {
                        // Check if this has the alarm set
                        let instance = self.instance_list.get(handle).unwrap();
                        let run_event = match instance.alarms.borrow_mut().get_mut(&alarm_id) {
                            Some(alarm) if *alarm >= 0 => {
                                // Decrement it, run the event if it hit 0
                                *alarm -= 1;
                                *alarm == 0
                            },
                            _ => false,
                        };
                        if run_event {
                            self.run_instance_event(gml::ev::ALARMS, alarm_id, handle, handle, None)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Runs all keyboard events for which the relevant key is held
    pub fn run_keyboard_events(&mut self) -> gml::Result<()> {
        let mut i = 0;
        while let Some((key, objects)) =
            self.event_holders[gml::ev::KEYBOARD].get_index(i).map(|(x, y)| (*x, y.clone()))
        {
            if self.input_manager.key_check(key as usize) {
                // Get all the objects which have this key event registered
                for object_id in objects.borrow().iter().copied() {
                    // Iter all instances of this object
                    let mut iter = self.instance_list.iter_by_object(object_id);
                    while let Some(handle) = iter.next(&self.instance_list) {
                        self.run_instance_event(gml::ev::KEYBOARD, key, handle, handle, None)?;
                    }
                }
            }
            i += 1;
        }
        Ok(())
    }

    /// Runs all keyboard events for which the relevant key was pressed
    pub fn run_key_press_events(&mut self) -> gml::Result<()> {
        let mut i = 0;
        while let Some((key, objects)) =
            self.event_holders[gml::ev::KEYPRESS].get_index(i).map(|(x, y)| (*x, y.clone()))
        {
            if self.input_manager.key_check_pressed(key as usize) {
                // Get all the objects which have this key event registered
                for object_id in objects.borrow().iter().copied() {
                    // Iter all instances of this object
                    let mut iter = self.instance_list.iter_by_object(object_id);
                    while let Some(handle) = iter.next(&self.instance_list) {
                        self.run_instance_event(gml::ev::KEYPRESS, key, handle, handle, None)?;
                    }
                }
            }
            i += 1;
        }
        Ok(())
    }

    /// Runs all keyboard events for which the relevant key was released
    pub fn run_key_release_events(&mut self) -> gml::Result<()> {
        let mut i = 0;
        while let Some((key, objects)) =
            self.event_holders[gml::ev::KEYRELEASE].get_index(i).map(|(x, y)| (*x, y.clone()))
        {
            if self.input_manager.key_check_released(key as usize) {
                // Get all the objects which have this key event registered
                for object_id in objects.borrow().iter().copied() {
                    // Iter all instances of this object
                    let mut iter = self.instance_list.iter_by_object(object_id);
                    while let Some(handle) = iter.next(&self.instance_list) {
                        self.run_instance_event(gml::ev::KEYRELEASE, key, handle, handle, None)?;
                    }
                }
            }
            i += 1;
        }
        Ok(())
    }

    /// Runs all outside room, intersect boundary, and outside/intersect view events.
    pub fn run_bound_events(&mut self) -> gml::Result<()> {
        // Outside room events
        let holders = match self.event_holders.get(gml::ev::OTHER).and_then(|x| x.get(&0)) {
            Some(e) => e.clone(),
            None => return Ok(()),
        };
        let mut position = 0;
        while let Some(&object_id) = holders.borrow().get(position) {
            let mut iter = self.instance_list.iter_by_object(object_id);
            while let Some(handle) = iter.next(&self.instance_list) {
                let instance = self.instance_list.get(handle).unwrap();
                instance.update_bbox(self.get_instance_mask_sprite(handle));
                if instance.bbox_right.get() < 0
                    || instance.bbox_bottom.get() < 0
                    || instance.bbox_left.get() > self.room_width
                    || instance.bbox_top.get() > self.room_height
                {
                    self.run_instance_event(gml::ev::OTHER, 0, handle, handle, None)?;
                }
            }
            position += 1;
        }

        // Intersect room boundary events
        let holders = match self.event_holders.get(gml::ev::OTHER).and_then(|x| x.get(&1)) {
            Some(e) => e.clone(),
            None => return Ok(()),
        };
        let mut position = 0;
        while let Some(&object_id) = holders.borrow().get(position) {
            let mut iter = self.instance_list.iter_by_object(object_id);
            while let Some(handle) = iter.next(&self.instance_list) {
                let instance = self.instance_list.get(handle).unwrap();
                instance.update_bbox(self.get_instance_mask_sprite(handle));
                if instance.bbox_left.get() < 0
                    || instance.bbox_top.get() < 0
                    || instance.bbox_right.get() > self.room_width
                    || instance.bbox_bottom.get() > self.room_height
                {
                    self.run_instance_event(gml::ev::OTHER, 1, handle, handle, None)?;
                }
            }
            position += 1;
        }

        let view_count = self.views.len().min(8);

        // Outside view events
        for i in 0..view_count {
            let event_number = (40 + i) as u32;
            let holders = match self.event_holders.get(gml::ev::OTHER).and_then(|x| x.get(&event_number)) {
                Some(e) => e.clone(),
                None => return Ok(()),
            };
            let mut position = 0;
            while let Some(&object_id) = holders.borrow().get(position) {
                let mut iter = self.instance_list.iter_by_object(object_id);
                while let Some(handle) = iter.next(&self.instance_list) {
                    let instance = self.instance_list.get(handle).unwrap();
                    instance.update_bbox(self.get_instance_mask_sprite(handle));
                    let view = &self.views[i];
                    if instance.bbox_right.get() < view.source_x
                        || instance.bbox_bottom.get() < view.source_y
                        || instance.bbox_left.get() > view.source_x + view.source_w as i32
                        || instance.bbox_top.get() > view.source_y + view.source_h as i32
                    {
                        self.run_instance_event(gml::ev::OTHER, event_number, handle, handle, None)?;
                    }
                }
                position += 1;
            }
        }

        // Intersect view events
        for i in 0..view_count {
            let event_number = (50 + i) as u32;
            let holders = match self.event_holders.get(gml::ev::OTHER).and_then(|x| x.get(&event_number)) {
                Some(e) => e.clone(),
                None => return Ok(()),
            };
            let mut position = 0;
            while let Some(&object_id) = holders.borrow().get(position) {
                let mut iter = self.instance_list.iter_by_object(object_id);
                while let Some(handle) = iter.next(&self.instance_list) {
                    let instance = self.instance_list.get(handle).unwrap();
                    instance.update_bbox(self.get_instance_mask_sprite(handle));
                    let view = &self.views[i];
                    if instance.bbox_left.get() < view.source_x
                        || instance.bbox_top.get() < view.source_y
                        || instance.bbox_right.get() > view.source_x + view.source_w as i32
                        || instance.bbox_bottom.get() > view.source_y + view.source_h as i32
                    {
                        self.run_instance_event(gml::ev::OTHER, event_number, handle, handle, None)?;
                    }
                }
                position += 1;
            }
        }

        Ok(())
    }

    /// Runs all collision events for the current active instances
    pub fn run_collisions(&mut self) -> gml::Result<()> {
        // Iter through every object that has a collision event registered (non-borrowing iter because Rust)
        let mut i = 0;
        while let Some((object, target_list)) =
            self.event_holders[gml::ev::COLLISION].get_index(i).map(|(x, y)| (*x, y.clone()))
        {
            // Iter every instance of this object
            let mut iter1 = self.instance_list.iter_by_object(object as i32);
            while let Some(instance) = iter1.next(&self.instance_list) {
                // Go through all its collision target objects
                for target_obj in target_list.borrow().iter().copied() {
                    // And iter every instance of the target object
                    let mut iter2 = self.instance_list.iter_by_object(target_obj);
                    while let Some(target) = iter2.next(&self.instance_list) {
                        // And finally, check if the two instances collide
                        if self.check_collision(instance, target) {
                            //self.handle_collision(instance, target, target_obj as u32)?;
                            //self.handle_collision(target, instance, object as u32)?;

                            // If either instance is solid, move both back to their previous positions
                            let inst1 = self.instance_list.get(instance).unwrap();
                            let inst2 = self.instance_list.get(target).unwrap();
                            if inst1.solid.get() || inst2.solid.get() {
                                inst1.x.set(inst1.xprevious.get());
                                inst1.y.set(inst1.yprevious.get());
                                inst1.bbox_is_stale.set(true);
                                inst2.x.set(inst2.xprevious.get());
                                inst2.y.set(inst2.yprevious.get());
                                inst2.bbox_is_stale.set(true);
                            }

                            // Run both collision events
                            self.run_instance_event(gml::ev::COLLISION, target_obj as u32, instance, target, None)?;
                            self.run_instance_event(gml::ev::COLLISION, object as u32, target, instance, None)?;

                            // If either instance is solid, apply both instances' hspeed and vspeed
                            let inst1 = self.instance_list.get(instance).unwrap();
                            let inst2 = self.instance_list.get(target).unwrap();
                            if inst1.solid.get() || inst2.solid.get() {
                                inst1.x.set(inst1.x.get() + inst1.hspeed.get());
                                inst1.y.set(inst1.y.get() + inst1.vspeed.get());
                                inst1.bbox_is_stale.set(true);
                                inst2.x.set(inst2.x.get() + inst2.hspeed.get());
                                inst2.y.set(inst2.y.get() + inst2.vspeed.get());
                                inst2.bbox_is_stale.set(true);

                                // If they're still colliding, move them back again
                                if self.check_collision(instance, target) {
                                    inst1.x.set(inst1.xprevious.get());
                                    inst1.y.set(inst1.yprevious.get());
                                    inst1.bbox_is_stale.set(true);
                                    inst2.x.set(inst2.xprevious.get());
                                    inst2.y.set(inst2.yprevious.get());
                                    inst2.bbox_is_stale.set(true);
                                }
                            }
                        }
                    }
                }
            }
            i += 1;
        }

        Ok(())
    }
}
