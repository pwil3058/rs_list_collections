use crate::OrderedMap;

pub struct OccupiedEntry<'a, K: 'a + Ord, V: 'a> {
    key: K,
    index: usize,
    map: &'a mut OrderedMap<K, V>,
}

impl<'a, K: 'a + Ord, V: 'a> OccupiedEntry<'a, K, V> {
    fn into_mut(self) -> &'a mut V {
        &mut self.map.values[self.index]
    }

    fn key(&self) -> &K {
        &self.key
    }

    fn get_mut(&mut self) -> &mut V {
        &mut self.map.values[self.index]
    }
}

pub struct VacantEntry<'a, K: 'a + Ord, V: 'a> {
    key: K,
    index: usize,
    map: &'a mut OrderedMap<K, V>,
}

impl<'a, K: 'a + Ord, V: 'a> VacantEntry<'a, K, V> {
    fn insert(self, value: V) -> &'a mut V {
        self.map.keys.insert(self.index, self.key);
        self.map.values.insert(self.index, value);
        &mut self.map.values[self.index]
    }

    fn key(&self) -> &K {
        &self.key
    }
}

pub enum Entry<'a, K: 'a + Ord, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K: Ord, V> Entry<'a, K, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    pub fn and_modify<F>(mut self, modify: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Entry::Occupied(entry) = &mut self {
            modify(entry.get_mut());
        }
        self
    }
}

impl<'a, K: Ord, V: Default> Entry<'a, K, V> {
    pub fn or_default(self) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        }
    }
}

impl<K: Ord, V> OrderedMap<K, V> {
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        match self.keys.binary_search(&key) {
            Ok(index) => {
                let entry = OccupiedEntry {
                    key,
                    index,
                    map: self,
                };
                Entry::Occupied(entry)
            }
            Err(index) => {
                let entry = VacantEntry {
                    key,
                    index,
                    map: self,
                };
                Entry::Vacant(entry)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_entry_or_insert() {
        let mut map: OrderedMap<&str, u32> = OrderedMap::new();
        map.entry("whatever").or_insert(3);
        assert_eq!(map.get(&"whatever"), Some(&3_u32));
        *map.entry("whatever").or_insert(10) *= 2;
        assert_eq!(map.get(&"whatever"), Some(&6_u32));
    }

    #[test]
    fn map_entry_or_insert_with() {
        let mut map: OrderedMap<&str, u32> = OrderedMap::new();
        map.entry("whatever").or_insert_with(|| 3);
        assert_eq!(map.get(&"whatever"), Some(&3_u32));
        *map.entry("whatever").or_insert_with(|| 10) *= 2;
        assert_eq!(map.get(&"whatever"), Some(&6_u32));
    }

    #[test]
    fn map_entry_key() {
        let mut map: OrderedMap<&str, u32> = OrderedMap::new();
        assert_eq!(map.entry("whatever").key(), &"whatever");
    }

    #[test]
    fn map_entry_and_modify() {
        let mut map: OrderedMap<&str, u32> = OrderedMap::new();
        map.entry("whatever").and_modify(|e| *e += 1).or_insert(3);
        assert_eq!(map.get(&"whatever"), Some(&3_u32));
        map.entry("whatever").and_modify(|e| *e += 1).or_insert(3);
        assert_eq!(map.get(&"whatever"), Some(&4_u32));
    }

    #[test]
    fn map_entry_or_default() {
        let mut map: OrderedMap<&str, u32> = OrderedMap::new();
        map.entry("whatever").or_default();
        assert_eq!(map.get(&"whatever"), Some(&0_u32));
        *map.entry("whatever").or_default() += 2;
        assert_eq!(map.get(&"whatever"), Some(&2_u32));
        *map.entry("whatever").or_default() += 2;
        assert_eq!(map.get(&"whatever"), Some(&4_u32));
    }
}
