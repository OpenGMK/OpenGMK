use crate::{game::Game, gml};

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
                self.run_instance_event(event_id, event_sub, instance, other.unwrap_or(instance))?;
            }
            position += 1;
        }
        Ok(())
    }

    /// Runs an event for a given instance. Does nothing if that instance doesn't have the specified event.
    pub fn run_instance_event(
        &mut self,
        event_id: usize,
        event_sub: u32,
        instance: usize,
        other: usize,
    ) -> gml::Result<()> {
        // Running instance events is not allowed if a room change is pending. This appears to be
        // how GM8 is implemented as well, given the related room creation bug and collision/solid bugs.
        if self.room_target.is_none() {
            let mut object_id =
                self.instance_list.get(instance).ok_or(gml::Error::InvalidInstanceHandle(instance))?.object_index.get();
            let event = loop {
                if object_id < 0 {
                    return Ok(())
                }
                if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
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
}
