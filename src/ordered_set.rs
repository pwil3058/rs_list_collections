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

//! Sets implemented as a sorted list.
//! Useful for those situations when ordered iteration over a set's
//! contents is a frequent requirement.

extern crate rand;

use std::convert::From;
use std::default::Default;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitOr, BitXor, Sub};
use std::slice::Iter;
use std::vec::Drain;

pub use crate::iterators::*;
pub use crate::ordered_map::{Keys, OrderedMap};
use crate::{a_contains_b, are_disjoint};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OrderedSet<T: Ord> {
    ordered_list: Vec<T>,
}

impl<T: Ord> OrderedSet<T> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Return the number of items in this set.
    pub fn len(&self) -> usize {
        self.ordered_list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ordered_list.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.ordered_list.capacity()
    }

    pub fn clear(&mut self) {
        self.ordered_list.clear()
    }

    /// Return false if the item was already a member otherwise true
    pub fn insert(&mut self, item: T) -> bool {
        if let Err(index) = self.ordered_list.binary_search(&item) {
            self.ordered_list.insert(index, item);
            true
        } else {
            false
        }
    }

    /// Return true if the item was a member and false otherwise
    pub fn remove(&mut self, item: &T) -> bool {
        if let Ok(index) = self.ordered_list.binary_search(item) {
            self.ordered_list.remove(index);
            true
        } else {
            false
        }
    }

    /// Return false if the item is already a member
    pub fn contains(&self, item: &T) -> bool {
        self.ordered_list.binary_search(item).is_ok()
    }

    pub fn first(&self) -> Option<&T> {
        self.ordered_list.first()
    }

    pub fn iter(&self) -> SetIter<T, Iter<T>> {
        SetIter::new(self.ordered_list.iter())
    }

    /// Return an iterator over the items in the set that occur after the
    /// given item in the sorting order
    pub fn iter_after(&self, item: &T) -> SetIter<T, Iter<T>> {
        match self.ordered_list.binary_search(item) {
            Ok(index) => SetIter::new(self.ordered_list[index + 1..].iter()),
            Err(index) => SetIter::new(self.ordered_list[index..].iter()),
        }
    }

    pub fn drain(&mut self) -> Drain<T> {
        self.ordered_list.drain(..)
    }

    /// Return true if ordered_list is sorted and contains no duplicates
    pub fn is_valid(&self) -> bool {
        for i in 1..self.ordered_list.len() {
            if self.ordered_list[i - 1] >= self.ordered_list[i] {
                return false;
            }
        }
        true
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        are_disjoint(
            &mut self.ordered_list.iter(),
            &mut other.ordered_list.iter(),
        )
    }

    pub fn is_map_disjoint<V>(&self, other: &OrderedMap<T, V>) -> bool {
        are_disjoint(&mut self.ordered_list.iter(), &mut other.keys())
    }

    /// Return true if self is a subset of other
    pub fn is_subset(&self, other: &Self) -> bool {
        a_contains_b(
            &mut other.ordered_list.iter(),
            &mut self.ordered_list.iter(),
        )
    }

    /// Return true if self is a subset of other
    pub fn is_proper_subset(&self, other: &Self) -> bool {
        other.len() > self.len()
            && a_contains_b(
                &mut other.ordered_list.iter(),
                &mut self.ordered_list.iter(),
            )
    }

    /// Return true if self is a superset of other
    pub fn is_superset(&self, other: &Self) -> bool {
        a_contains_b(
            &mut self.ordered_list.iter(),
            &mut other.ordered_list.iter(),
        )
    }

    /// Return true if self is a superset of other
    pub fn is_proper_superset(&self, other: &Self) -> bool {
        self.len() > other.len()
            && a_contains_b(
                &mut self.ordered_list.iter(),
                &mut other.ordered_list.iter(),
            )
    }
}

impl<T: Ord> Default for OrderedSet<T> {
    fn default() -> Self {
        Self {
            ordered_list: vec![],
        }
    }
}

/// Convert to OrderedSet<T> from ordered Vec<T> with no duplicates
impl<T: Ord> From<Vec<T>> for OrderedSet<T> {
    fn from(ordered_list: Vec<T>) -> Self {
        let list = Self { ordered_list };
        assert!(list.is_valid());
        list
    }
}

impl<T: Ord> FromIterator<T> for OrderedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut ordered_set = OrderedSet::<T>::default();

        for i in iter {
            ordered_set.insert(i);
        }

        ordered_set
    }
}

impl<'a, T: 'a + Ord + Clone> FromIterator<&'a T> for OrderedSet<T> {
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        let mut ordered_set = OrderedSet::<T>::default();

        for i in iter.into_iter().cloned() {
            ordered_set.insert(i);
        }

        ordered_set
    }
}

// TODO: add doc strings to arguments for these macros.
macro_rules! define_set_operation {
    ( $iter:ident, $function:ident, $osi_function:ident, $op:ident, $op_fn:ident  ) => {
        impl<T: Ord> OrderedSet<T> {
            pub fn $function<'a>(&'a self, other: &'a Self) -> $iter<'a, T, Iter<T>, Iter<T>> {
                $iter::new(self.ordered_list.iter(), other.ordered_list.iter())
            }

            pub fn $osi_function<'a, I: Iterator<Item = &'a T>>(
                &'a self,
                other_iter: I,
            ) -> $iter<'a, T, Iter<T>, I> {
                $iter::new(self.ordered_list.iter(), other_iter)
            }
        }

        impl<T: Ord + Clone> $op for OrderedSet<T> {
            type Output = Self;

            fn $op_fn(self, other: Self) -> Self::Output {
                self.$function(&other).to_set()
            }
        }

        impl<T: Ord + Clone> $op for &OrderedSet<T> {
            type Output = OrderedSet<T>;

            fn $op_fn(self, other: Self) -> Self::Output {
                self.$function(&other).to_set()
            }
        }
    };
}

define_set_operation!(Difference, difference, osi_difference, Sub, sub);
define_set_operation!(
    SymmetricDifference,
    symmetric_difference,
    osi_symmetric_difference,
    BitXor,
    bitxor
);
define_set_operation!(Union, union, osi_union, BitOr, bitor);
define_set_operation!(Intersection, intersection, osi_intersection, BitAnd, bitand);

macro_rules! define_set_map_operation {
    ( $iter:ident, $function:ident ) => {
        impl<T: Ord> OrderedSet<T> {
            pub fn $function<'a, V>(
                &'a self,
                other: &'a OrderedMap<T, V>,
            ) -> $iter<'a, T, Iter<T>, Keys<T, V>> {
                $iter::new(self.ordered_list.iter(), other.keys())
            }
        }
    };
}

define_set_map_operation!(Union, map_union);
define_set_map_operation!(Intersection, map_intersection);
define_set_map_operation!(Difference, map_difference);
define_set_map_operation!(SymmetricDifference, map_symmetric_difference);

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use super::*;
    use rand::prelude::*;

    static TEST_STRS: &[&str] = &[
        "hhh", "aaa", "ggg", "sss", "zzz", "bbb", "fff", "iii", "qqq", "jjj", "ddd", "eee", "ccc",
        "mmm", "lll", "nnn", "ppp", "rrr",
    ];

    fn random_sequence(length: usize) -> Vec<u64> {
        let mut v = vec![];
        for _ in 0..length {
            let t: u64 = random();
            v.push(t)
        }
        v
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn default_works() {
        assert!(OrderedSet::<String>::default().len() == 0);
        assert!(OrderedSet::<u32>::default().len() == 0);
    }

    #[test]
    fn check_constraints() {
        // This is to chek what constraints are required for T
        // to give full functionality to the sets
        // Won't compile with Clone.
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
        struct Item {
            i: u32,
        }
        let list_1 = OrderedSet::<Item>::default();
        let list_2 = OrderedSet::<Item>::default();
        let list_3 = list_1 | list_2;
        assert!(list_3.len() == 0);
    }

    #[test]
    fn insert_works() {
        let mut str_set = OrderedSet::<String>::default();
        assert!(str_set.is_valid());
        assert!(str_set.first().is_none());
        for text in TEST_STRS.iter() {
            assert!(str_set.insert(text.to_string()));
            assert!(str_set.is_valid());
            assert!(str_set.contains(&text.to_string()));
            assert!(!str_set.insert(text.to_string()));
            assert!(str_set.is_valid());
            assert!(str_set.contains(&text.to_string()));
        }
        for text in TEST_STRS.iter() {
            assert!(!str_set.insert(text.to_string()));
            assert!(str_set.is_valid());
            assert!(str_set.contains(&text.to_string()));
        }
        assert_eq!(str_set.first(), Some(&"aaa".to_string()));
    }

    #[test]
    fn from_iter_works() {
        let str_set: OrderedSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        assert!(str_set.is_valid());
        for text in TEST_STRS.iter() {
            assert!(str_set.contains(&text.to_string()))
        }
        for string in str_set.iter() {
            assert!(TEST_STRS.contains(&string.as_str()));
        }

        let u64_seq = random_sequence(1000);
        assert_eq!(u64_seq.len(), 1000);
        let u64_set: OrderedSet<u64> = u64_seq.iter().map(|u| *u).collect();
        assert!(u64_set.is_valid());
        for u in u64_seq.iter() {
            assert!(u64_set.contains(u));
        }
        for u in u64_set.iter() {
            assert!(u64_seq.contains(u));
        }
        assert_eq!(u64_seq.len(), u64_set.len());
    }

    #[test]
    fn iter_after_works() {
        let str_set: OrderedSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        for item in str_set.iter_after(&"jjj".to_string()) {
            assert!(item > &"jjj".to_string());
            assert!(TEST_STRS.contains(&item.as_str()));
        }
        for item in str_set.iter_after(&"zzz".to_string()) {
            assert!(item > &"zzz".to_string());
            assert!(false);
        }
    }

    #[test]
    fn remove_works() {
        let mut str_set: OrderedSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        for text in TEST_STRS.iter() {
            assert!(str_set.remove(&text.to_string()));
            assert!(!str_set.remove(&text.to_string()));
        }
        assert!(str_set.is_empty());
    }

    #[test]
    fn equality_and_hash_work() {
        let str_set1: OrderedSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        let mut str_set2: OrderedSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(str_set1, str_set2);
        assert_eq!(calculate_hash(&str_set1), calculate_hash(&str_set2));
        assert!(str_set2.remove(&TEST_STRS.first().unwrap().to_string()));
        assert!(str_set1 != str_set2);
        assert!(calculate_hash(&str_set1) != calculate_hash(&str_set2));
    }

    #[test]
    fn test_is_disjoint() {
        let str_set1: OrderedSet<String> =
            TEST_STRS[0..5].into_iter().map(|s| s.to_string()).collect();
        let str_set2: OrderedSet<String> = TEST_STRS[5..].into_iter().map(|s| s.to_string()).collect();
        assert!(str_set1.is_disjoint(&str_set2));
        let str_set1: OrderedSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: OrderedSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        assert!(!str_set1.is_disjoint(&str_set2));

        let u64_seq = random_sequence(1000);
        let set1: OrderedSet<u64> = u64_seq[0..500].iter().map(|u| *u).collect();
        let set2: OrderedSet<u64> = u64_seq[500..].iter().map(|u| *u).collect();
        assert!(set1.is_disjoint(&set2));
        let set1: OrderedSet<u64> = u64_seq[0..700].iter().map(|u| *u).collect();
        let set2: OrderedSet<u64> = u64_seq[300..].iter().map(|u| *u).collect();
        assert!(!set1.is_disjoint(&set2));
    }

    #[test]
    fn test_is_subset() {
        let max = TEST_STRS.len();
        for test in &[
            ((0, 7), (7, max), false, false),
            ((7, max), (5, max), true, true),
            ((5, max), (7, max), false, false),
            ((1, 7), (1, 7), true, false),
            ((0, 9), (5, max), false, false),
            ((1, max), (1, 7), false, false),
        ] {
            println!("TEST: {:?}", test); // to help identify failed tests
            let set1: OrderedSet<String> = TEST_STRS[(test.0).0..(test.0).1]
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            let set2: OrderedSet<String> = TEST_STRS[(test.1).0..(test.1).1]
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            assert!(set1.is_subset(&set2) == test.2);
            if set1.is_subset(&set2) {
                for item in set1.iter() {
                    assert!(set2.contains(item));
                }
            }
            assert!(set1.is_proper_subset(&set2) == test.3);
        }
    }

    #[test]
    fn test_is_superset() {
        let max = TEST_STRS.len();
        for test in &[
            ((0, 7), (7, max), false, false),
            ((7, max), (5, max), false, false),
            ((5, max), (7, max), true, true),
            ((1, 7), (1, 7), true, false),
            ((0, 9), (5, max), false, false),
            ((1, max), (1, 7), true, true),
        ] {
            println!("TEST: {:?}", test); // to help identify failed tests
            let set1: OrderedSet<String> = TEST_STRS[(test.0).0..(test.0).1]
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            let set2: OrderedSet<String> = TEST_STRS[(test.1).0..(test.1).1]
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            assert!(set1.is_superset(&set2) == test.2);
            if set1.is_superset(&set2) {
                for item in set2.iter() {
                    assert!(set1.contains(item));
                }
            }
            assert!(set1.is_proper_superset(&set2) == test.3);
        }
    }

    #[test]
    fn test_difference() {
        let str_set1: OrderedSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: OrderedSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let expected: OrderedSet<String> =
            TEST_STRS[0..4].into_iter().map(|s| s.to_string()).collect();
        let result = str_set1 - str_set2;
        assert!(result.is_valid());
        assert_eq!(expected, result);
    }

    #[test]
    fn test_symmetric_difference() {
        let str_set1: OrderedSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: OrderedSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let mut expected: OrderedSet<String> =
            TEST_STRS[0..4].into_iter().map(|s| s.to_string()).collect();
        for item in TEST_STRS[8..].into_iter().map(|s| s.to_string()) {
            expected.insert(item);
        }
        let result = str_set1 ^ str_set2;
        assert!(result.is_valid());
        assert_eq!(expected, result);
    }

    #[test]
    fn test_union() {
        let str_set1: OrderedSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: OrderedSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let expected: OrderedSet<String> = TEST_STRS[0..].into_iter().map(|s| s.to_string()).collect();
        let result = str_set1 | str_set2;
        assert!(result.is_valid());
        assert_eq!(expected, result);
    }

    #[test]
    fn test_intersection() {
        let str_set1: OrderedSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: OrderedSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let expected: OrderedSet<String> =
            TEST_STRS[4..8].into_iter().map(|s| s.to_string()).collect();
        let result = str_set1 & str_set2;
        assert!(result.is_valid());
        assert_eq!(expected, result);
    }
}