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

use std::cmp::Ordering;
use std::default::Default;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitOr, BitXor, Sub};
use std::slice::Iter;
use std::vec::Drain;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ListSet<T: Ord> {
    ordered_list: Vec<T>,
}

impl<T: Ord> ListSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    // Return the number of items in this set.
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

    // Return false if the item was already a member otherwise true
    pub fn insert(&mut self, item: T) -> bool {
        if let Err(index) = self.ordered_list.binary_search(&item) {
            self.ordered_list.insert(index, item);
            true
        } else {
            false
        }
    }

    // Return true if the item was a member and false otherwise
    pub fn remove(&mut self, item: &T) -> bool {
        if let Ok(index) = self.ordered_list.binary_search(item) {
            self.ordered_list.remove(index);
            true
        } else {
            false
        }
    }

    // Return false if the item is already a member
    pub fn contains(&self, item: &T) -> bool {
        self.ordered_list.binary_search(item).is_ok()
    }

    pub fn first(&self) -> Option<&T> {
        self.ordered_list.first()
    }

    pub fn iter(&self) -> Iter<T> {
        self.ordered_list.iter()
    }

    pub fn drain(&mut self) -> Drain<T> {
        self.ordered_list.drain(..)
    }

    // Return true if ordered_list is sorted and contains no duplicates
    pub fn is_valid(&self) -> bool {
        for i in 1..self.ordered_list.len() {
            if self.ordered_list[i - 1] >= self.ordered_list[i] {
                return false;
            }
        }
        true
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        let mut self_iter = self.ordered_list.iter();
        let mut other_iter = other.ordered_list.iter();
        let mut o_self_cur_item = self_iter.next();
        let mut o_other_cur_item = other_iter.next();
        while let Some(self_cur_item) = o_self_cur_item {
            if let Some(other_cur_item) = o_other_cur_item {
                match self_cur_item.cmp(&other_cur_item) {
                    Ordering::Less => {
                        o_self_cur_item = self_iter.next();
                    }
                    Ordering::Greater => {
                        o_other_cur_item = other_iter.next();
                    }
                    Ordering::Equal => {
                        return false;
                    }
                }
            } else {
                return true;
            }
        }
        true
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        let mut self_iter = self.ordered_list.iter();
        let mut other_iter = other.ordered_list.iter();
        let mut o_self_cur_item = self_iter.next();
        let mut o_other_cur_item = other_iter.next();
        while let Some(self_cur_item) = o_self_cur_item {
            if let Some(other_cur_item) = o_other_cur_item {
                match self_cur_item.cmp(&other_cur_item) {
                    Ordering::Less => {
                        o_self_cur_item = self_iter.next();
                    }
                    Ordering::Greater => {
                        return false;
                    }
                    Ordering::Equal => {
                        o_self_cur_item = self_iter.next();
                        o_other_cur_item = other_iter.next();
                    }
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl<T: Ord> Default for ListSet<T> {
    fn default() -> Self {
        Self {
            ordered_list: vec![],
        }
    }
}

impl<T: Ord> FromIterator<T> for ListSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list_set = ListSet::<T>::default();

        for i in iter {
            list_set.insert(i);
        }

        list_set
    }
}

impl<'a, T: 'a + Ord + Clone> FromIterator<&'a T> for ListSet<T> {
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        let mut list_set = ListSet::<T>::default();

        for i in iter.into_iter().cloned() {
            list_set.insert(i);
        }

        list_set
    }
}

macro_rules! define_set_operation {
    ( $iter:ident, $function:ident, $op:ident, $op_fn:ident  ) => {
        pub struct $iter<'a, T: Ord> {
            o_lh_cur_item: Option<&'a T>,
            o_rh_cur_item: Option<&'a T>,
            lh_iter: Iter<'a, T>,
            rh_iter: Iter<'a, T>,
        }

        impl<T: Ord> ListSet<T> {
            pub fn $function<'a>(&'a self, other: &'a Self) -> $iter<'a, T> {
                let mut self_iter = self.ordered_list.iter();
                let mut other_iter = other.ordered_list.iter();
                $iter::<T> {
                    o_lh_cur_item: self_iter.next(),
                    o_rh_cur_item: other_iter.next(),
                    lh_iter: self_iter,
                    rh_iter: other_iter,
                }
            }
        }

        impl<T: Ord + Clone> $op for ListSet<T> {
            type Output = Self;

            fn $op_fn(self, other: Self) -> Self::Output {
                self.$function(&other).collect()
            }
        }
    };
}

define_set_operation!(Difference, difference, Sub, sub);

impl<'a, T: Ord> Iterator for Difference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(lh_cur_item) = self.o_lh_cur_item {
            if let Some(rh_cur_item) = self.o_rh_cur_item {
                match lh_cur_item.cmp(&rh_cur_item) {
                    Ordering::Less => {
                        self.o_lh_cur_item = self.lh_iter.next();
                        return Some(lh_cur_item);
                    }
                    Ordering::Greater => {
                        self.o_rh_cur_item = self.rh_iter.next();
                    }
                    Ordering::Equal => {
                        self.o_lh_cur_item = self.lh_iter.next();
                        self.o_rh_cur_item = self.rh_iter.next();
                    }
                }
            } else {
                self.o_lh_cur_item = self.lh_iter.next();
                return Some(lh_cur_item);
            }
        }
        None
    }
}

define_set_operation!(SymmetricDifference, symmetric_difference, BitXor, bitxor);

impl<'a, T: Ord> Iterator for SymmetricDifference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(lh_cur_item) = self.o_lh_cur_item {
                if let Some(rh_cur_item) = self.o_rh_cur_item {
                    match lh_cur_item.cmp(&rh_cur_item) {
                        Ordering::Less => {
                            self.o_lh_cur_item = self.lh_iter.next();
                            return Some(lh_cur_item);
                        }
                        Ordering::Greater => {
                            self.o_rh_cur_item = self.rh_iter.next();
                            return Some(rh_cur_item);
                        }
                        Ordering::Equal => {
                            self.o_lh_cur_item = self.lh_iter.next();
                            self.o_rh_cur_item = self.rh_iter.next();
                        }
                    }
                } else {
                    self.o_lh_cur_item = self.lh_iter.next();
                    return Some(lh_cur_item);
                }
            } else if let Some(rh_cur_item) = self.o_rh_cur_item {
                self.o_rh_cur_item = self.rh_iter.next();
                return Some(rh_cur_item);
            } else {
                return None;
            }
        }
    }
}

define_set_operation!(Union, union, BitOr, bitor);

impl<'a, T: Ord> Iterator for Union<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(lh_cur_item) = self.o_lh_cur_item {
                if let Some(rh_cur_item) = self.o_rh_cur_item {
                    match lh_cur_item.cmp(&rh_cur_item) {
                        Ordering::Less => {
                            self.o_lh_cur_item = self.lh_iter.next();
                            return Some(lh_cur_item);
                        }
                        Ordering::Greater => {
                            self.o_rh_cur_item = self.rh_iter.next();
                            return Some(rh_cur_item);
                        }
                        Ordering::Equal => {
                            self.o_lh_cur_item = self.lh_iter.next();
                            self.o_rh_cur_item = self.rh_iter.next();
                            return Some(lh_cur_item);
                        }
                    }
                } else {
                    self.o_lh_cur_item = self.lh_iter.next();
                    return Some(lh_cur_item);
                }
            } else if let Some(rh_cur_item) = self.o_rh_cur_item {
                self.o_rh_cur_item = self.rh_iter.next();
                return Some(rh_cur_item);
            } else {
                return None;
            }
        }
    }
}

define_set_operation!(Intersection, intersection, BitAnd, bitand);

impl<'a, T: Ord> Iterator for Intersection<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(lh_cur_item) = self.o_lh_cur_item {
                if let Some(rh_cur_item) = self.o_rh_cur_item {
                    match lh_cur_item.cmp(&rh_cur_item) {
                        Ordering::Less => {
                            self.o_lh_cur_item = self.lh_iter.next();
                        }
                        Ordering::Greater => {
                            self.o_rh_cur_item = self.rh_iter.next();
                        }
                        Ordering::Equal => {
                            self.o_lh_cur_item = self.lh_iter.next();
                            self.o_rh_cur_item = self.rh_iter.next();
                            return Some(lh_cur_item);
                        }
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
}

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
        //0..length.map(|_| random::<u64>()).collect()
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
        assert!(ListSet::<String>::default().len() == 0);
        assert!(ListSet::<u32>::default().len() == 0);
    }

    #[test]
    fn insert_works() {
        let mut str_set = ListSet::<String>::default();
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
        let str_set: ListSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        assert!(str_set.is_valid());
        for text in TEST_STRS.iter() {
            assert!(str_set.contains(&text.to_string()))
        }
        for string in str_set.iter() {
            assert!(TEST_STRS.contains(&string.as_str()));
        }

        let u64_seq = random_sequence(1000);
        assert_eq!(u64_seq.len(), 1000);
        let u64_set: ListSet<u64> = u64_seq.iter().map(|u| *u).collect();
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
    fn remove_works() {
        let mut str_set: ListSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        for text in TEST_STRS.iter() {
            assert!(str_set.remove(&text.to_string()));
            assert!(!str_set.remove(&text.to_string()));
        }
        assert!(str_set.is_empty());
    }

    #[test]
    fn equality_and_hash_work() {
        let str_set1: ListSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        let mut str_set2: ListSet<String> = TEST_STRS.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(str_set1, str_set2);
        assert_eq!(calculate_hash(&str_set1), calculate_hash(&str_set2));
        assert!(str_set2.remove(&TEST_STRS.first().unwrap().to_string()));
        assert!(str_set1 != str_set2);
        assert!(calculate_hash(&str_set1) != calculate_hash(&str_set2));
    }

    #[test]
    fn test_is_disjoint() {
        let str_set1: ListSet<String> =
            TEST_STRS[0..5].into_iter().map(|s| s.to_string()).collect();
        let str_set2: ListSet<String> = TEST_STRS[5..].into_iter().map(|s| s.to_string()).collect();
        assert!(str_set1.is_disjoint(&str_set2));
        let str_set1: ListSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: ListSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        assert!(!str_set1.is_disjoint(&str_set2));

        let u64_seq = random_sequence(1000);
        let set1: ListSet<u64> = u64_seq[0..500].iter().map(|u| *u).collect();
        let set2: ListSet<u64> = u64_seq[500..].iter().map(|u| *u).collect();
        assert!(set1.is_disjoint(&set2));
        let set1: ListSet<u64> = u64_seq[0..700].iter().map(|u| *u).collect();
        let set2: ListSet<u64> = u64_seq[300..].iter().map(|u| *u).collect();
        assert!(!set1.is_disjoint(&set2));
    }

    #[test]
    fn test_is_subset() {
        let set1: ListSet<String> = TEST_STRS[0..7].into_iter().map(|s| s.to_string()).collect();
        let set2: ListSet<String> = TEST_STRS[7..].into_iter().map(|s| s.to_string()).collect();
        let set3: ListSet<String> = TEST_STRS[5..].into_iter().map(|s| s.to_string()).collect();
        assert!(!set1.is_subset(&set2));
        assert!(!set1.is_subset(&set3));
        assert!(!set2.is_subset(&set3));
        assert!(set1.is_subset(&set1));
        assert!(set3.is_subset(&set2));
    }

    #[test]
    fn test_difference() {
        let str_set1: ListSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: ListSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let expected: ListSet<String> =
            TEST_STRS[0..4].into_iter().map(|s| s.to_string()).collect();
        let result = str_set1 - str_set2;
        assert!(result.is_valid());
        assert_eq!(expected, result);
    }

    #[test]
    fn test_symmetric_difference() {
        let str_set1: ListSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: ListSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let mut expected: ListSet<String> =
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
        let str_set1: ListSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: ListSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let expected: ListSet<String> = TEST_STRS[0..].into_iter().map(|s| s.to_string()).collect();
        let result = str_set1 | str_set2;
        assert!(result.is_valid());
        assert_eq!(expected, result);
    }

    #[test]
    fn test_intersection() {
        let str_set1: ListSet<String> =
            TEST_STRS[0..8].into_iter().map(|s| s.to_string()).collect();
        let str_set2: ListSet<String> = TEST_STRS[4..].into_iter().map(|s| s.to_string()).collect();
        let expected: ListSet<String> =
            TEST_STRS[4..8].into_iter().map(|s| s.to_string()).collect();
        let result = str_set1 & str_set2;
        assert!(result.is_valid());
        assert_eq!(expected, result);
    }
}
