use crate::{
    gml::Value,
    instance::{Field, Instance},
    
};
use std::collections::HashMap;

pub struct Context<'a> {
    /// Reference to the "self" instance
    pub this: &'a Instance,

    /// Reference to the "other" instance
    pub other: &'a Instance,

    /// Index of the action currently being executed, starting at 0
    pub event_action: usize,

    /// Type of event (0-11) currently being executed
    pub event_type: usize,

    /// Sub-event ID
    pub event_number: usize,

    /// The ID of the object which owns this event - note that this isn't necessarily the same as
    /// self.object_index, as the event could have been inherited from a parent object
    pub event_object: u32,

    /// Arguments passed to scripts and such
    pub arguments: &'a [Value],

    /// Local variables specific to this context
    /// TODO: replace this with a dummy field-holder object? Global behaves the same way.
    pub locals: HashMap<usize, Field>,

    /// Return value from this execution - should be initialized to zero as it won't necessarily be written
    pub return_value: Value,
}
