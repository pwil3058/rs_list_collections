// Copyright 2019 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::slice::IterMut;

use crate::OrderedMap;
use crate::OrderedSet;

pub trait ToList<'a, T>: Iterator<Item = &'a T>
where
    T: 'a + Clone,
{
    /// Create a Vec<T> list from the items in the Iterator's output
    fn to_list(&mut self) -> Vec<T> {
        self.cloned().collect()
    }
}

pub trait ToSet<'a, T>: ToList<'a, T>
where
    T: 'a + Ord + Clone,
{
    /// Create a OrderedSet<T> from the items in the Iterator's output
    fn to_set(&mut self) -> OrderedSet<T> {
        OrderedSet::<T> {
            ordered_list: self.to_list(),
        }
    }
}

pub trait ToTupleList<'a, K, V>: Iterator<Item = (&'a K, &'a V)>
where
    K: 'a + Clone,
    V: 'a + Clone,
{
    /// Create a Vec<T> list from the items in the Iterator's output
    fn to_list(&mut self) -> Vec<(K, V)> {
        self.map(|(x, y)| (x.clone(), y.clone())).collect()
    }
}

pub trait ToMap<'a, K, V>: ToTupleList<'a, K, V>
where
    K: 'a + Ord + Clone,
    V: 'a + Clone,
{
    /// Create a OrderedSet<T> from the items in the Iterator's output
    fn to_map(&mut self) -> OrderedMap<K, V> {
        OrderedMap::<K, V> {
            ordered_list: self.to_list(),
        }
    }
}

/// Iterator enhancement to provide a skip ahead feature. This mechanism
/// is used to optimise implementation of set operation (difference, intersection, etc)
/// iterators.
pub trait SkipAheadIterator<'a, K: 'a + Ord, V: 'a>: Iterator<Item = V> {
    /// Return the next item in the iterator whose value is greater than
    /// to the given value.
    fn next_after(&mut self, key: &K) -> Option<Self::Item>;

    /// Return the next item in the iterator whose value is greater than
    /// or equal to the given value.  Used to optimise set operation
    /// iterators.
    fn next_from(&mut self, key: &K) -> Option<Self::Item>;
}

pub trait ScopedIterator<'a, K: 'a + Ord, V: 'a>: SkipAheadIterator<'a, K, V> {
    fn from(self, key: &K) -> Self;
}

/// Return true if the data stream from the Iterator is ordered and
/// contains no duplicates.  Useful for testing.
pub fn output_is_ordered_nodups<'a, T, I>(iter: &mut I) -> bool
where
    T: 'a + Ord,
    I: SkipAheadIterator<'a, T, &'a T>,
{
    let mut o_previous = iter.next();
    while let Some(previous) = o_previous {
        if let Some(item) = iter.next() {
            if previous >= item {
                return false;
            }
            o_previous = Some(item);
        } else {
            o_previous = None;
        }
    }
    true
}

// SET ITERATOR

/// An Iterator over the items in an ordered list
pub struct SetIter<'a, T: Ord> {
    ordered_list: &'a [T],
    index: usize,
}

impl<'a, T: Ord> SetIter<'a, T> {
    pub fn new(ordered_list: &'a [T]) -> Self {
        Self {
            ordered_list,
            index: 0,
        }
    }
}

impl<'a, T: Ord> Iterator for SetIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a, T: 'a + Ord> SkipAheadIterator<'a, T, &'a T> for SetIter<'a, T> {
    fn next_after(&mut self, t: &T) -> Option<Self::Item> {
        self.index += after_index!(self.ordered_list[self.index..], t);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn next_from(&mut self, t: &T) -> Option<Self::Item> {
        self.index += from_index!(self.ordered_list[self.index..], t);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a, T: 'a + Ord> ScopedIterator<'a, T, &'a T> for SetIter<'a, T> {
     fn from(self, t: &T) -> Self {
        let start = from_index!(self.ordered_list, t);
        Self::new(&self.ordered_list[start..])
    }
}

impl<'a, T: Ord + Clone> ToList<'a, T> for SetIter<'a, T> {}

impl<'a, T: Ord + Clone> ToSet<'a, T> for SetIter<'a, T> {}

// MAP ITERATOR

/// An Iterator over the items in an ordered map
pub struct MapIter<'a, K: Ord, V> {
    ordered_list: &'a [(K, V)],
    index: usize,
}

impl<'a, K: Ord, V> MapIter<'a, K, V> {
    pub fn new(ordered_list: &'a [(K, V)]) -> Self {
        Self {
            ordered_list,
            index: 0,
        }
    }
}

impl<'a, K: Ord, V> Iterator for MapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some((&item.0, &item.1))
        } else {
            None
        }
    }
}

impl<'a, K: 'a + Ord, V: 'a> SkipAheadIterator<'a, K, (&'a K, &'a V)> for MapIter<'a, K, V> {
    fn next_after(&mut self, k: &K) -> Option<Self::Item> {
        self.index += tuple_after_index!(self.ordered_list[self.index..], k);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some((&item.0, &item.1))
        } else {
            None
        }
    }

    fn next_from(&mut self, k: &K) -> Option<Self::Item> {
        self.index += tuple_from_index!(self.ordered_list[self.index..], k);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some((&item.0, &item.1))
        } else {
            None
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> ToTupleList<'a, K, V> for MapIter<'a, K, V> {}

impl<'a, K: Ord + Clone, V: Clone> ToMap<'a, K, V> for MapIter<'a, K, V> {}

// MUT MAP ITERATOR

/// An Iterator over the keys and mutable values in an ordered map in key order
// Use built in mutable iterator due to insoluble lifetime issues
pub struct MapIterMut<'a, K: Ord, V> {
    iter_mut: IterMut<'a, (K, V)>,
}

impl<'a, K: 'a + Ord, V: 'a> MapIterMut<'a, K, V> {
    pub fn new(ordered_list: &'a mut [(K, V)]) -> Self {
        Self {
            iter_mut: ordered_list.iter_mut(),
        }
    }
}

impl<'a, K: Ord, V> Iterator for MapIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((key, value)) = self.iter_mut.next() {
            return Some((&*key, value));
        } else {
            None
        }
    }
}

impl<'a, K: 'a + Ord, V: 'a> SkipAheadIterator<'a, K, (&'a K, &'a mut V)> for MapIterMut<'a, K, V> {
    fn next_after(&mut self, k: &K) -> Option<Self::Item> {
        while let Some((key, value)) = self.iter_mut.next() {
            if *key > *k {
                return Some((&*key, value));
            }
        }
        None
    }

    fn next_from(&mut self, k: &K) -> Option<Self::Item> {
        while let Some((key, value)) = self.iter_mut.next() {
            if *key >= *k {
                return Some((&*key, value));
            }
        }
        None
    }
}

// KEY ITERATOR

/// An Iterator over the keys in an ordered map
pub struct KeyIter<'a, K: Ord, V> {
    ordered_list: &'a [(K, V)],
    index: usize,
}

impl<'a, K: Ord, V> KeyIter<'a, K, V> {
    pub fn new(ordered_list: &'a [(K, V)]) -> Self {
        Self {
            ordered_list,
            index: 0,
        }
    }
}

impl<'a, K: Ord, V> Iterator for KeyIter<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(&item.0)
        } else {
            None
        }
    }
}

impl<'a, K: 'a + Ord, V> SkipAheadIterator<'a, K, &'a K> for KeyIter<'a, K, V> {
    fn next_after(&mut self, k: &K) -> Option<Self::Item> {
        self.index += tuple_after_index!(self.ordered_list[self.index..], k);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(&item.0)
        } else {
            None
        }
    }

    fn next_from(&mut self, t: &K) -> Option<Self::Item> {
        self.index += tuple_from_index!(self.ordered_list[self.index..], t);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(&item.0)
        } else {
            None
        }
    }
}

impl<'a, K: 'a + Ord, V> ScopedIterator<'a, K, &'a K> for KeyIter<'a, K, V> {
    fn from(self, k: &K) -> Self {
        let start = tuple_from_index!(self.ordered_list, k);
        Self::new(&self.ordered_list[start..])
    }
}

impl<'a, K: Ord + Clone, V> ToList<'a, K> for KeyIter<'a, K, V> {}

impl<'a, K: Ord + Clone, V> ToSet<'a, K> for KeyIter<'a, K, V> {}

// VALUE ITERATOR

/// An Iterator over the values in an ordered map in key order
pub struct ValueIter<'a, K: Ord, V> {
    ordered_list: &'a [(K, V)],
    index: usize,
}

impl<'a, K: Ord, V> ValueIter<'a, K, V> {
    pub fn new(ordered_list: &'a [(K, V)]) -> Self {
        Self {
            ordered_list,
            index: 0,
        }
    }
}

impl<'a, K: Ord, V> Iterator for ValueIter<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(&item.1)
        } else {
            None
        }
    }
}

impl<'a, K: Ord, V> SkipAheadIterator<'a, K, &'a V> for ValueIter<'a, K, V> {
    fn next_after(&mut self, k: &K) -> Option<Self::Item> {
        self.index += tuple_after_index!(self.ordered_list[self.index..], k);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(&item.1)
        } else {
            None
        }
    }

    fn next_from(&mut self, t: &K) -> Option<Self::Item> {
        self.index += tuple_from_index!(self.ordered_list[self.index..], t);
        if let Some(item) = self.ordered_list.get(self.index) {
            self.index += 1;
            Some(&item.1)
        } else {
            None
        }
    }
}

impl<'a, K: Ord, V: Clone> ToList<'a, V> for ValueIter<'a, K, V> {}

// MUT VALUE ITERATOR

/// An Iterator over the values in an ordered map in key order
// Use built in mutable iterator due to insoluble lifetime issues
pub struct ValueIterMut<'a, K: Ord, V> {
    iter_mut: IterMut<'a, (K, V)>,
}

impl<'a, K: 'a + Ord, V: 'a> ValueIterMut<'a, K, V> {
    pub fn new(ordered_list: &'a mut [(K, V)]) -> Self {
        Self {
            iter_mut: ordered_list.iter_mut(),
        }
    }
}

impl<'a, K: Ord, V> Iterator for ValueIterMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, value)) = self.iter_mut.next() {
            return Some(value);
        } else {
            None
        }
    }
}

impl<'a, K: Ord, V: 'a> SkipAheadIterator<'a, K, &'a mut V> for ValueIterMut<'a, K, V> {
    fn next_after(&mut self, k: &K) -> Option<Self::Item> {
        while let Some((key, value)) = self.iter_mut.next() {
            if *key > *k {
                return Some(value);
            }
        }
        None
    }

    fn next_from(&mut self, k: &K) -> Option<Self::Item> {
        while let Some((key, value)) = self.iter_mut.next() {
            if *key >= *k {
                return Some(value);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static LIST: &[&str] = &["a", "c", "e", "g", "i", "k", "m"];
    static MAP: &[(&str, i32)] = &[
        ("a", 6),
        ("c", 5),
        ("e", 4),
        ("g", 3),
        ("i", 2),
        ("k", 1),
        ("m", 0),
    ];
    static LIST_UNORDERED: &[&str] = &["a", "c", "e", "z", "g", "i", "k", "m"];

    #[test]
    fn output_is_ordered_nodups_works() {
        assert!(output_is_ordered_nodups(&mut SetIter::new(LIST)));
        let rev: Vec<&str> = LIST.iter().rev().cloned().collect();
        assert!(!output_is_ordered_nodups(&mut SetIter::new(&rev)));
        assert!(!output_is_ordered_nodups(&mut SetIter::new(LIST_UNORDERED)));
        let rev: Vec<&str> = LIST_UNORDERED.iter().rev().cloned().collect();
        assert!(!output_is_ordered_nodups(&mut SetIter::new(&rev)));
        //assert!(output_is_ordered_nodups(&mut MapIter::new(MAP)));
    }

    #[test]
    fn map_next_after_works() {
        assert_eq!(MapIter::new(MAP).next_after(&"g"), Some((&"i", &2)));
        assert_eq!(MapIter::new(MAP).next_after(&"a"), Some((&"c", &5)));
        let mut iter = MapIter::new(MAP);
        assert_eq!(iter.next_after(&"k"), Some((&"m", &0)));
        assert_eq!(iter.next_after(&"k"), None);
    }

    #[test]
    fn map_next_from_works() {
        assert_eq!(MapIter::new(MAP).next_from(&"g"), Some((&"g", &3)));
        assert_eq!(MapIter::new(MAP).next_from(&"a"), Some((&"a", &6)));
        let mut iter = MapIter::new(MAP);
        assert_eq!(iter.next_from(&"m"), Some((&"m", &0)));
        assert_eq!(iter.next_from(&"m"), None);
    }

    #[test]
    fn set_iter_works() {
        let vec = LIST.to_vec();
        assert_eq!(SetIter::new(LIST).to_list(), vec);
        let mut set_iter = SetIter::new(LIST);
        assert_eq!(set_iter.next_after(&"g"), Some(&"i"));
        assert_eq!(set_iter.to_list(), vec[5..].to_vec());
        let mut set_iter = SetIter::new(LIST);
        assert_eq!(set_iter.next_from(&"g"), Some(&"g"));
        assert_eq!(set_iter.to_list(), vec[4..].to_vec());
    }

    #[test]
    fn set_scoped_iter_works() {
        assert_eq!(SetIter::new(LIST).from(&"g").next(), Some(&"g"));
        assert_eq!(SetIter::new(LIST).from(&"f").next(), Some(&"g"));
    }

    #[test]
    fn map_iter_works() {
        let vec = MAP.to_vec();
        let set_iter = MapIter::new(MAP);
        let result: Vec<(&str, i32)> = set_iter.map(|(x, y)| (x.clone(), y.clone())).collect();
        assert_eq!(result, vec);
        let mut set_iter = MapIter::new(MAP);
        assert_eq!(set_iter.next_after(&"g"), Some((&"i", &2)));
        let result: Vec<(&str, i32)> = set_iter.map(|(x, y)| (x.clone(), y.clone())).collect();
        assert_eq!(result, vec[5..].to_vec());
        let mut set_iter = MapIter::new(MAP);
        assert_eq!(set_iter.next_from(&"g"), Some((&"g", &3)));
        let result: Vec<(&str, i32)> = set_iter.map(|(x, y)| (x.clone(), y.clone())).collect();
        assert_eq!(result, vec[4..].to_vec());
    }

    #[test]
    fn map_iter_mut_works() {
        let mut map: Vec<(&str, i32)> = MAP.iter().map(|(x, y)| (x.clone(), y.clone())).collect();
        for (i, pair) in MapIter::new(&map).enumerate() {
            assert_eq!((6 - i as i32), *pair.1);
        }
        for (i, (_, value)) in MapIterMut::new(&mut map).enumerate() {
            *value = i as i32 + 5;
        }
        for (i, pair) in MapIter::new(&map).enumerate() {
            assert_eq!((i as i32 + 5), *pair.1);
        }
    }

    #[test]
    fn key_iter_works() {
        let vec = LIST.to_vec();
        assert_eq!(KeyIter::new(MAP).to_list(), vec);
        let mut set_iter = KeyIter::new(MAP);
        assert_eq!(set_iter.next_after(&"g"), Some(&"i"));
        assert_eq!(set_iter.to_list(), vec[5..].to_vec());
        let mut set_iter = KeyIter::new(MAP);
        assert_eq!(set_iter.next_from(&"g"), Some(&"g"));
        assert_eq!(set_iter.to_list(), vec[4..].to_vec());
    }

    #[test]
    fn value_iter_mut_works() {
        let vec: Vec<i32> = MAP.iter().map(|x| x.1).collect();
        let mut map: Vec<(&str, i32)> = MAP.iter().map(|(x, y)| (x.clone(), y.clone())).collect();
        let set_iter = ValueIterMut::new(&mut map);
        let result: Vec<i32> = set_iter.map(|x| *x).collect();
        assert_eq!(result, vec);
        let mut set_iter = ValueIterMut::new(&mut map);
        assert_eq!(set_iter.next_after(&"g"), Some(&mut 2_i32));
        let result: Vec<i32> = set_iter.map(|x| *x).collect();
        assert_eq!(result, vec[5..].to_vec());
        let mut set_iter = ValueIterMut::new(&mut map);
        assert_eq!(set_iter.next_from(&"g"), Some(&mut 3_i32));
        let result: Vec<i32> = set_iter.map(|x| *x).collect();
        assert_eq!(result, vec[4..].to_vec());
        for (i, value) in ValueIter::new(&map).enumerate() {
            assert!(*value != i as i32 || i == 3);
        }
        for (i, value) in ValueIterMut::new(&mut map).enumerate() {
            *value = i as i32;
        }
        for (i, value) in ValueIter::new(&map).enumerate() {
            assert_eq!(*value, i as i32);
        }
    }

    #[test]
    fn value_iter_works() {
        let vec: Vec<i32> = MAP.iter().map(|x| x.1).collect();
        assert_eq!(ValueIter::new(MAP).to_list(), vec);
        let mut set_iter = ValueIter::new(MAP);
        assert_eq!(set_iter.next_after(&"g"), Some(&2_i32));
        assert_eq!(set_iter.to_list(), vec[5..].to_vec());
        let mut set_iter = ValueIter::new(MAP);
        assert_eq!(set_iter.next_from(&"g"), Some(&3_i32));
        assert_eq!(set_iter.to_list(), vec[4..].to_vec());
    }

    #[test]
    fn iter_after_works() {
        let vec = LIST.to_vec();
        let mut iter_after = SetIter::new(&LIST[after_index!(LIST, &"g")..]);
        assert_eq!(iter_after.to_list(), vec[4..].to_vec());
        let mut iter_after = SetIter::new(&LIST[after_index!(LIST, &"f")..]);
        assert_eq!(iter_after.to_list(), vec[3..].to_vec());
    }

    #[test]
    fn iter_before_works() {
        let vec = LIST.to_vec();
        let mut iter_before = SetIter::new(&LIST[..from_index!(LIST, &"g")]);
        assert_eq!(iter_before.to_list(), vec[..3].to_vec());
        let mut iter_before = SetIter::new(&LIST[..from_index!(LIST, &"f")]);
        assert_eq!(iter_before.to_list(), vec[..3].to_vec());
    }

    #[test]
    fn iter_from_works() {
        let vec = LIST.to_vec();
        let mut iter_from = SetIter::new(&LIST[from_index!(LIST, &"g")..]);
        assert_eq!(iter_from.to_list(), vec[3..].to_vec());
        let mut iter_from = SetIter::new(&LIST[from_index!(LIST, &"f")..]);
        assert_eq!(iter_from.to_list(), vec[3..].to_vec());
    }

    #[test]
    fn iter_until_works() {
        let vec = LIST.to_vec();
        let mut iter_until = SetIter::new(&LIST[..after_index!(LIST, &"g")]);
        assert_eq!(iter_until.to_list(), vec[..4].to_vec());
        let mut iter_until = SetIter::new(&LIST[..after_index!(LIST, &"f")]);
        assert_eq!(iter_until.to_list(), vec[..3].to_vec());
    }
}
