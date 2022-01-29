use crate::{gml::Value, instance::DummyFieldHolder, types::ID};

#[derive(Default)]
pub struct Context {
    /// InstanceList handle to the "self" instance
    pub this: usize,

    /// InstanceList handle to the "other" instance
    pub other: usize,

    /// Index of the action currently being executed, starting at 0
    pub event_action: usize,

    /// Whether the drag-n-drop "relative" box was checked - accessed in GML with argument_relative
    pub relative: bool,

    /// Type of event (0-11) currently being executed
    pub event_type: usize,

    /// Sub-event ID
    pub event_number: usize,

    /// The ID of the object which owns this event - note that this isn't necessarily the same as
    /// self.object_index, as the event could have been inherited from a parent object
    pub event_object: ID,

    /// Arguments passed to scripts and such. There are always 16 arguments in a Context,
    /// regardless of argument_count. The extra ones can be written and read under some circumstances.
    pub arguments: [Value; 16],

    /// Number of initialized arguments
    /// May only be 0-16 usually, but could theoretically go up to u32::max in corrupted gamedata
    pub argument_count: usize,

    /// Local variables specific to this context
    pub locals: DummyFieldHolder,

    /// Return value from this execution - should be initialized to zero as it won't necessarily be written
    pub return_value: Value,
}

impl Context {
    /// Creates a default context with a given instance handle as both the `self` and `other` parameters.
    #[inline(always)]
    pub fn with_single_instance(handle: usize) -> Self {
        Self { this: handle, other: handle, ..Default::default() }
    }

    /// Creates a new context with some given arguments and a default `locals` map,
    /// and all other values copied from another context.
    #[inline(always)]
    pub fn copy_with_args(context: &Self, arguments: [Value; 16], argument_count: usize) -> Self {
        Self {
            arguments,
            argument_count,
            locals: DummyFieldHolder::new(),
            return_value: Default::default(),
            ..*context
        }
    }
}
