//! Sets implemented as a sorted list.
//! Useful for those situations when ordered iteration over a set's
//! contents is a frequent requirement.

pub mod iter_ops;
pub mod map_entry;
pub mod ordered_iterators;
pub mod ordered_map;
pub mod ordered_set;

pub use ordered_map::OrderedMap;
pub use ordered_set::OrderedSet;

#[cfg(test)]
mod tests {
    //use super::*;
}
