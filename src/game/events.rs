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
                            self.handle_collision(instance, target, target_obj as u32)?;
                            self.handle_collision(target, instance, object as u32)?;
                        }
                    }
                }
            }
            i += 1;
        }

        Ok(())
    }

    fn handle_collision(&mut self, inst1: usize, inst2: usize, sub_event: u32) -> gml::Result<()> {
        // If the target is solid, move outside of it
        if self.instance_list.get(inst2).map(|x| x.solid.get()).unwrap_or(false) {
            self.instance_list.get(inst1).map(|instance| {
                instance.x.set(instance.xprevious.get());
                instance.y.set(instance.yprevious.get());
                instance.bbox_is_stale.set(true);
            });
        }

        // Run collision event
        self.run_instance_event(gml::ev::COLLISION, sub_event, inst1, inst2, None)?;

        // If the target is solid (yes we have to check it a second time) then add hspeed and vspeed to our x/y
        // and then, if colliding with the solid again, move outside it again.
        // TODO: is this 100% accurate? It seems insane...
        if self.instance_list.get(inst2).map(|x| x.solid.get()).unwrap_or(false) {
            self.instance_list.get(inst1).map(|instance| {
                instance.x.set(instance.x.get() + instance.hspeed.get());
                instance.y.set(instance.y.get() + instance.vspeed.get());
                instance.bbox_is_stale.set(true);
            });
            if self.check_collision(inst1, inst2) {
                self.instance_list.get(inst1).map(|instance| {
                    instance.x.set(instance.xprevious.get());
                    instance.y.set(instance.yprevious.get());
                    instance.bbox_is_stale.set(true);
                });
            }
        }

        Ok(())
    }
}
