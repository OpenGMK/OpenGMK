use crate::{
    asset::trigger::TriggerTime,
    game::{Game, GetAsset},
    gml,
    instance::Instance,
};
use gmio::input::MouseButton;
use shared::types::ID;

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
        if self.scene_change.is_none() {
            let original_object_id =
                if let Some(id) = as_object { id } else { self.instance_list.get(instance).object_index.get() };
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

    /// Runs room end followed by game end events for all instances. Should be called only when the game ends.
    pub fn run_game_end_events(&mut self) -> gml::Result<()> {
        // Reset this so the events will run
        self.scene_change = None;

        // Room end
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list) {
            self.run_instance_event(gml::ev::OTHER, 5, instance, instance, None)?;
        }

        // Game end
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list) {
            self.run_instance_event(gml::ev::OTHER, 3, instance, instance, None)?;
        }

        Ok(())
    }

    pub fn run_triggers(&mut self, moment: TriggerTime) -> gml::Result<()> {
        let mut i = 0;
        while let Some((trigger_id, objects)) =
            self.event_holders[gml::ev::TRIGGER].get_index(i).map(|(x, y)| (*x, y.clone()))
        {
            if let Some(trigger) = self.assets.triggers[trigger_id as usize].as_ref() {
                if trigger.moment == moment {
                    let trigger = trigger.clone();
                    for object_id in objects.borrow().iter().copied() {
                        let mut iter = self.instance_list.iter_by_object(object_id);
                        while let Some(handle) = iter.next(&self.instance_list) {
                            let mut context = gml::Context {
                                this: handle,
                                other: handle,
                                event_action: 0,
                                relative: false,
                                event_type: 11,
                                event_number: trigger_id as _,
                                event_object: self.instance_list.get(handle).object_index.get(),
                                arguments: Default::default(),
                                argument_count: 0,
                                locals: Default::default(),
                                return_value: Default::default(),
                            };
                            self.execute(&trigger.condition, &mut context)?;
                            if context.return_value.is_truthy() {
                                self.run_instance_event(gml::ev::TRIGGER, trigger_id, handle, handle, None)?;
                            }
                        }
                    }
                }
            }
            i += 1;
        }
        Ok(())
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
                        let instance = self.instance_list.get(handle);
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

    /// Runs all mouse events, including button, button pressed, button released, mouse scroll, mouse enter/leave
    pub fn run_mouse_events(&mut self) -> gml::Result<()> {
        let (mouse_x, mouse_y) = self.get_mouse_in_room();
        let (mouse_x_previous, mouse_y_previous) = self.get_mouse_previous_in_room();

        // Macro which runs a given event for all instances which the mouse is currently over.
        // Event type is gml::ev::MOUSE, you must provide the sub-event.
        macro_rules! try_mouse_events {
            ($sub: literal) => {{
                if let Some(holders) = self.event_holders.get(gml::ev::MOUSE).and_then(|x| x.get(&$sub)) {
                    let holders = holders.clone();
                    let mut position = 0;
                    while let Some(&object_id) = holders.borrow().get(position) {
                        let mut iter = self.instance_list.iter_by_object(object_id);
                        while let Some(handle) = iter.next(&self.instance_list) {
                            if self.check_collision_point(handle, mouse_x, mouse_y, true) {
                                self.run_instance_event(gml::ev::MOUSE, $sub, handle, handle, None)?;
                            }
                        }
                        position += 1;
                    }
                }
            }};
        }

        // Left button
        if self.input_manager.mouse_check(MouseButton::Left) {
            try_mouse_events!(0);
        }

        // Right button
        if self.input_manager.mouse_check(MouseButton::Right) {
            try_mouse_events!(1);
        }

        // Middle button
        if self.input_manager.mouse_check(MouseButton::Left) {
            try_mouse_events!(2);
        }

        // No button
        if !self.input_manager.mouse_check_any() {
            try_mouse_events!(3);
        }

        // Left button pressed
        if self.input_manager.mouse_check_pressed(MouseButton::Left) {
            try_mouse_events!(4);
        }

        // Right button pressed
        if self.input_manager.mouse_check_pressed(MouseButton::Right) {
            try_mouse_events!(5);
        }

        // Middle button pressed
        if self.input_manager.mouse_check_pressed(MouseButton::Left) {
            try_mouse_events!(6);
        }

        // Left button released
        if self.input_manager.mouse_check_released(MouseButton::Left) {
            try_mouse_events!(7);
        }

        // Right button released
        if self.input_manager.mouse_check_released(MouseButton::Right) {
            try_mouse_events!(8);
        }

        // Middle button released
        if self.input_manager.mouse_check_released(MouseButton::Left) {
            try_mouse_events!(9);
        }

        // Mouse enter
        if let Some(holders) = self.event_holders.get(gml::ev::MOUSE).and_then(|x| x.get(&10)) {
            let holders = holders.clone();
            let mut position = 0;
            while let Some(&object_id) = holders.borrow().get(position) {
                let mut iter = self.instance_list.iter_by_object(object_id);
                while let Some(handle) = iter.next(&self.instance_list) {
                    if self.check_collision_point(handle, mouse_x, mouse_y, true)
                        && !self.check_collision_point(handle, mouse_x_previous, mouse_y_previous, true)
                    {
                        self.run_instance_event(gml::ev::MOUSE, 10, handle, handle, None)?;
                    }
                }
                position += 1;
            }
        }

        // Mouse leave
        if let Some(holders) = self.event_holders.get(gml::ev::MOUSE).and_then(|x| x.get(&11)) {
            let holders = holders.clone();
            let mut position = 0;
            while let Some(&object_id) = holders.borrow().get(position) {
                let mut iter = self.instance_list.iter_by_object(object_id);
                while let Some(handle) = iter.next(&self.instance_list) {
                    if self.check_collision_point(handle, mouse_x, mouse_y, true)
                        && !self.check_collision_point(handle, mouse_x_previous, mouse_y_previous, true)
                    {
                        self.run_instance_event(gml::ev::MOUSE, 11, handle, handle, None)?;
                    }
                }
                position += 1;
            }
        }

        // Global left button
        if self.input_manager.mouse_check(MouseButton::Left) {
            self.run_object_event(gml::ev::MOUSE, 50, None)?;
        }

        // Global right button
        if self.input_manager.mouse_check(MouseButton::Right) {
            self.run_object_event(gml::ev::MOUSE, 51, None)?;
        }

        // Global middle button
        if self.input_manager.mouse_check(MouseButton::Middle) {
            self.run_object_event(gml::ev::MOUSE, 52, None)?;
        }

        // Global left button pressed
        if self.input_manager.mouse_check_pressed(MouseButton::Left) {
            self.run_object_event(gml::ev::MOUSE, 53, None)?;
        }

        // Global right button pressed
        if self.input_manager.mouse_check_pressed(MouseButton::Right) {
            self.run_object_event(gml::ev::MOUSE, 54, None)?;
        }

        // Global middle button pressed
        if self.input_manager.mouse_check_pressed(MouseButton::Middle) {
            self.run_object_event(gml::ev::MOUSE, 55, None)?;
        }

        // Global left button released
        if self.input_manager.mouse_check_released(MouseButton::Left) {
            self.run_object_event(gml::ev::MOUSE, 56, None)?;
        }

        // Global right button released
        if self.input_manager.mouse_check_released(MouseButton::Right) {
            self.run_object_event(gml::ev::MOUSE, 57, None)?;
        }

        // Global middle button released
        if self.input_manager.mouse_check_released(MouseButton::Middle) {
            self.run_object_event(gml::ev::MOUSE, 58, None)?;
        }

        // Mouse wheel up
        if self.input_manager.mouse_check_scroll_up() {
            self.run_object_event(gml::ev::MOUSE, 60, None)?;
        }

        // Mouse wheel up
        if self.input_manager.mouse_check_scroll_down() {
            self.run_object_event(gml::ev::MOUSE, 61, None)?;
        }

        Ok(())
    }

    /// Runs all outside room, intersect boundary, and outside/intersect view events.
    pub fn run_bound_events(&mut self) -> gml::Result<()> {
        fn instance_outside_rect(i: &Instance, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
            i.bbox_right.get() < x1 || i.bbox_bottom.get() < y1 || i.bbox_left.get() > x2 || i.bbox_top.get() > y2
        }

        fn point_outside_rect(x: f64, y: f64, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
            (x.floor() as i32) < x1 || (y.floor() as i32) < y1 || (x.ceil() as i32) > x2 || (y.ceil() as i32 > y2)
        }

        fn instance_intersect_rect(i: &Instance, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
            i.bbox_left.get() < x1 || i.bbox_top.get() < y1 || i.bbox_right.get() > x2 || i.bbox_bottom.get() > y2
        }

        // Outside room events
        let holders = match self.event_holders.get(gml::ev::OTHER).and_then(|x| x.get(&0)) {
            Some(e) => e.clone(),
            None => return Ok(()),
        };
        let mut position = 0;
        while let Some(&object_id) = holders.borrow().get(position) {
            let mut iter = self.instance_list.iter_by_object(object_id);
            while let Some(handle) = iter.next(&self.instance_list) {
                let instance = self.instance_list.get(handle);
                let mask = self.get_instance_mask_sprite(handle);

                let outside = if mask.is_some() {
                    instance.update_bbox(mask);
                    instance_outside_rect(instance, 0, 0, self.room_width, self.room_height)
                } else {
                    point_outside_rect(
                        instance.x.get().into(),
                        instance.y.get().into(),
                        0,
                        0,
                        self.room_width,
                        self.room_height,
                    )
                };
                if outside {
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
                let instance = self.instance_list.get(handle);
                let mask = self.get_instance_mask_sprite(handle);

                let intersect = if mask.is_some() {
                    instance.update_bbox(mask);
                    instance_intersect_rect(instance, 0, 0, self.room_width, self.room_height)
                } else {
                    point_outside_rect(
                        instance.x.get().into(),
                        instance.y.get().into(),
                        0,
                        0,
                        self.room_width,
                        self.room_height,
                    )
                };
                if intersect {
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
                    let instance = self.instance_list.get(handle);
                    let mask = self.get_instance_mask_sprite(handle);
                    let view = &self.views[i];

                    let outside = if mask.is_some() {
                        instance.update_bbox(mask);
                        instance_outside_rect(
                            instance,
                            view.source_x,
                            view.source_y,
                            view.source_x + view.source_w as i32,
                            view.source_y + view.source_h as i32,
                        )
                    } else {
                        point_outside_rect(
                            instance.x.get().into(),
                            instance.y.get().into(),
                            view.source_x,
                            view.source_y,
                            view.source_x + view.source_w as i32,
                            view.source_y + view.source_h as i32,
                        )
                    };
                    if outside {
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
                    let instance = self.instance_list.get(handle);
                    let mask = self.get_instance_mask_sprite(handle);
                    let view = &self.views[i];

                    let intersect = if mask.is_some() {
                        instance.update_bbox(mask);
                        instance_intersect_rect(
                            instance,
                            view.source_x,
                            view.source_y,
                            view.source_x + view.source_w as i32,
                            view.source_y + view.source_h as i32,
                        )
                    } else {
                        point_outside_rect(
                            instance.x.get().into(),
                            instance.y.get().into(),
                            view.source_x,
                            view.source_y,
                            view.source_x + view.source_w as i32,
                            view.source_y + view.source_h as i32,
                        )
                    };
                    if intersect {
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
                            let inst1 = self.instance_list.get(instance);
                            let inst2 = self.instance_list.get(target);
                            if inst1.solid.get() || inst2.solid.get() {
                                inst1.x.set(inst1.xprevious.get());
                                inst1.y.set(inst1.yprevious.get());
                                inst1.bbox_is_stale.set(true);
                                inst1.path_position.set(inst1.path_positionprevious.get());
                                inst2.x.set(inst2.xprevious.get());
                                inst2.y.set(inst2.yprevious.get());
                                inst2.bbox_is_stale.set(true);
                                inst2.path_position.set(inst2.path_positionprevious.get());
                            }

                            // Run both collision events
                            self.run_instance_event(gml::ev::COLLISION, target_obj as u32, instance, target, None)?;
                            self.run_instance_event(gml::ev::COLLISION, object as u32, target, instance, None)?;

                            // If either instance is solid, apply both instances' hspeed and vspeed
                            let inst1 = self.instance_list.get(instance);
                            let inst2 = self.instance_list.get(target);
                            if inst1.solid.get() || inst2.solid.get() {
                                self.apply_speeds(instance);
                                self.apply_speeds(target);

                                // If they're still colliding, move them back again
                                if self.check_collision(instance, target) {
                                    inst1.x.set(inst1.xprevious.get());
                                    inst1.y.set(inst1.yprevious.get());
                                    inst1.bbox_is_stale.set(true);
                                    inst1.path_position.set(inst1.path_positionprevious.get());
                                    inst2.x.set(inst2.xprevious.get());
                                    inst2.y.set(inst2.yprevious.get());
                                    inst2.bbox_is_stale.set(true);
                                    inst2.path_position.set(inst2.path_positionprevious.get());
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
