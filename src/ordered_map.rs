//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::convert::From;
use std::default::Default;

use crate::iter_ops::*;
use crate::ordered_iterators::*;

use crate::OrderedMap;

impl<K: Ord, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self {
            keys: vec![],
            values: vec![],
        }
    }
}

impl<K: Ord, V> OrderedMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return true if keys is sorted and contains no duplicate keys
    /// and the same length as values.
    pub fn is_valid(&self) -> bool {
        for i in 1..self.keys.len() {
            if self.keys[i - 1] >= self.keys[i] {
                return false;
            }
        }
        self.keys.len() == self.values.len()
    }

    /// Return the number of items in this set.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.keys.capacity().min(self.values.capacity())
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.keys.binary_search(key).is_ok()
    }

    // TODO: implement a useful drain for OrderedMap
    //pub fn drain(&mut self) -> Drain<(K, V)> {
    //    self.keys.drain(..)
    //}

    pub fn iter(&self) -> MapIter<K, V> {
        MapIter::new(&self.keys, &self.values)
    }

    pub fn merge<'a>(
        &'a self,
        other: &'a Self,
    ) -> MapMergeIter<'a, K, V, MapIter<K, V>, MapIter<K, V>> {
        MapMergeIter::new(self.iter(), other.iter())
    }

    pub fn iter_mut(&mut self) -> MapIterMut<K, V> {
        MapIterMut::new(&self.keys, &mut self.values)
    }

    pub fn iter_after(&self, key: &K) -> MapIter<K, V> {
        let start = after_index![self.keys, key];
        MapIter::new(&self.keys[start..], &self.values[start..])
    }

    pub fn keys(&self) -> SetIter<K> {
        SetIter::new(&self.keys)
    }

    pub fn values(&self) -> ValueIter<K, V> {
        ValueIter::new(&self.keys, &self.values)
    }

    pub fn values_mut<'a>(&'a mut self) -> ValueIterMut<'a, K, V> {
        ValueIterMut::new(&self.keys, &mut self.values)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Ok(index) = self.keys.binary_search(key) {
            Some(&self.values[index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Ok(index) = self.keys.binary_search(key) {
            Some(&mut self.values[index])
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.keys.binary_search(&key) {
            Ok(index) => {
                let old = self.values.remove(index);
                self.values.insert(index, value);
                Some(old)
            }
            Err(index) => {
                self.keys.insert(index, key);
                self.values.insert(index, value);
                None
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.keys.binary_search(&key) {
            Ok(index) => {
                self.keys.remove(index);
                Some(self.values.remove(index))
            }
            Err(_) => None,
        }
    }
}

/// Convert to OrderedMap<K, V> from a Vec<(K, V)>
impl<K: Ord, V> From<Vec<(K, V)>> for OrderedMap<K, V> {
    fn from(mut list: Vec<(K, V)>) -> Self {
        let mut map = Self::default();
        for (key, value) in list.drain(..) {
            map.insert(key, value);
        }
        assert!(map.is_valid());
        map
    }
}

/// Convert to OrderedMap<K, V> from a Vec<(K, V)>
impl<K: Ord + Clone, V: Clone> From<&[(K, V)]> for OrderedMap<K, V> {
    fn from(list: &[(K, V)]) -> Self {
        let mut map = Self::default();
        for (key, value) in list.iter() {
            map.insert(key.clone(), value.clone());
        }
        assert!(map.is_valid());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    static TEST_ITEMS_0: &[(&str, (&str, u32))] = &[
        ("hhh", ("HHH", 0)),
        ("aaa", ("AAA", 0)),
        ("ggg", ("GGG", 0)),
        ("sss", ("SSS", 0)),
        ("zzz", ("ZZZ", 0)),
        ("bbb", ("BBB", 0)),
        ("fff", ("FFF", 0)),
        ("iii", ("III", 0)),
        ("qqq", ("QQQ", 0)),
        ("jjj", ("JJJ", 0)),
        ("ddd", ("DDD", 0)),
        ("eee", ("EEE", 0)),
        ("ccc", ("CCC", 0)),
        ("mmm", ("MMM", 0)),
        ("lll", ("LLL", 0)),
        ("nnn", ("NNN", 0)),
        ("ppp", ("PPP", 0)),
        ("rrr", ("RRR", 0)),
    ];

    static TEST_ITEMS_1: &[(&str, (&str, u32))] = &[
        ("hhh", ("HHH", 1)),
        ("aaa", ("AAA", 1)),
        ("ggg", ("GGG", 1)),
        ("sss", ("SSS", 1)),
        ("zzz", ("ZZZ", 1)),
        ("bbb", ("BBB", 1)),
        ("fff", ("FFF", 1)),
        ("iii", ("III", 1)),
        ("qqq", ("QQQ", 1)),
        ("jjj", ("JJJ", 1)),
        ("ddd", ("DDD", 1)),
        ("eee", ("EEE", 1)),
        ("ccc", ("CCC", 1)),
        ("mmm", ("MMM", 1)),
        ("lll", ("LLL", 1)),
        ("nnn", ("NNN", 1)),
        ("ppp", ("PPP", 1)),
        ("rrr", ("RRR", 1)),
    ];

    #[test]
    fn map_default_works() {
        let map = OrderedMap::<u32, u32>::default();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn map_basic_functionality() {
        let mut map = OrderedMap::<&str, (&str, u32)>::default();
        for (key, value) in TEST_ITEMS_0.iter() {
            println!("{:?} => {:?}", key, value);
            assert!(map.insert(key, *value).is_none());
            assert!(map.is_valid());
            assert_eq!(map.get(key), Some(value));
            assert_eq!(map.insert(key, *value), Some(*value));
            assert!(map.is_valid());
        }
        let mut hash_map = HashMap::<&str, (&str, u32)>::new();
        for (key, value) in TEST_ITEMS_0.iter() {
            assert!(hash_map.insert(key, *value).is_none());
        }
        for (key, value) in TEST_ITEMS_1.iter() {
            if let Some(old_value) = hash_map.get(key) {
                assert_eq!(map.insert(key, *value), Some(*old_value));
                assert!(map.is_valid());
                assert_eq!(map.get(key), Some(value));
            } else {
                assert!(false);
            }
        }
    }
}
