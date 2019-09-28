//! Contains common type definitions used across GameMaker 8.

/// Represents an object, instance, tile or special values.
///
/// When positive, it would refer to:
/// - Asset Index in range 0..100_000
/// - Instance Index in range 100_000..10_000_000
/// - Tile Index in range 10_000_000..= (Undefined Behaviour)
///
/// Negative values have special meaning in GML, those being:
/// - `self` / -1, referring to the context of the executing object
/// - `other` / -2, referring to the context of other instances in special events (ex: collision with other instance)
/// - `all` / -3, referring to the context of every instance
/// - `noone` / -4, referring to a nonexistent instance
/// - `global` / -5, referring to a global dummy object accessible anywhere
/// - `local` / -7, referring to the context of a dummy object
/// that holds variables of the current script
///
/// Regarding `local`, this snippet:
///
/// ```gml
/// var x;
/// x = 10;
/// ```
///
/// ... is equivalent to this snippet:
///
/// ```gml
/// local.x = 10;
/// ```
pub type ID = i32;
