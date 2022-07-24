use core::borrow::Borrow;
use core::hash::Hash;

use hashbrown::{HashMap, HashSet};

pub struct HashMap2<K1, K2, V> {
    map: HashMap<K1, HashMap<K2, V>>,
}

impl<K1, K2, V> HashMap2<K1, K2, V>
where
    K1: Hash + Eq,
    K2: Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key1: K1, key2: K2, value: V) {
        self.map
            .raw_entry_mut()
            .from_key(&key1)
            .or_insert_with(|| (key1, HashMap::new()))
            .1
            .insert(key2, value);
    }

    pub fn get<Q1: ?Sized, Q2: ?Sized>(&self, key1: &Q1, key2: &Q2) -> Option<&V>
    where
        K1: Borrow<Q1>,
        K2: Borrow<Q2>,
        Q1: Hash + Eq,
        Q2: Hash + Eq,
    {
        self.map.get(key1).and_then(|map| map.get(key2))
    }

    pub fn contains_key<Q1: ?Sized, Q2: ?Sized>(&self, key1: &Q1, key2: &Q2) -> bool
    where
        K1: Borrow<Q1>,
        K2: Borrow<Q2>,
        Q1: Hash + Eq,
        Q2: Hash + Eq,
    {
        self.map
            .get(key1)
            .map_or(false, |map| map.contains_key(key2))
    }

    pub fn get_mut<Q1: ?Sized, Q2: ?Sized>(&mut self, key1: &Q1, key2: &Q2) -> Option<&mut V>
    where
        K1: Borrow<Q1>,
        K2: Borrow<Q2>,
        Q1: Hash + Eq,
        Q2: Hash + Eq,
    {
        self.map.get_mut(key1).and_then(|map| map.get_mut(key2))
    }

    pub fn for_each<'a, F>(&'a self, mut f: F)
    where
        F: FnMut((&'a K1, &'a K2, &'a V)),
    {
        for (k1, map) in &self.map {
            for (k2, v) in map {
                f((k1, k2, v));
            }
        }
    }

    pub fn for_each_mut<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut((&'a K1, &'a K2, &'a mut V)),
    {
        for (k1, map) in &mut self.map {
            for (k2, v) in map {
                f((k1, k2, v));
            }
        }
    }
}

impl<K1, K2, V> Default for HashMap2<K1, K2, V>
where
    K1: Hash + Eq,
    K2: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct HashSet4<K1, K2, K3, K4> {
    map: HashMap<K1, HashMap<K2, HashMap<K3, HashSet<K4>>>>,
}

impl<K1, K2, K3, K4> HashSet4<K1, K2, K3, K4>
where
    K1: Hash + Eq,
    K2: Hash + Eq,
    K3: Hash + Eq,
    K4: Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key1: K1, key2: K2, key3: K3, key4: K4) {
        self.map
            .raw_entry_mut()
            .from_key(&key1)
            .or_insert_with(|| (key1, HashMap::new()))
            .1
            .raw_entry_mut()
            .from_key(&key2)
            .or_insert_with(|| (key2, HashMap::new()))
            .1
            .raw_entry_mut()
            .from_key(&key3)
            .or_insert_with(|| (key3, HashSet::new()))
            .1
            .insert(key4);
    }

    pub fn contains<Q1: ?Sized, Q2: ?Sized, Q3: ?Sized, Q4: ?Sized>(
        &self,
        key1: &Q1,
        key2: &Q2,
        key3: &Q3,
        key4: &Q4,
    ) -> bool
    where
        K1: Borrow<Q1>,
        K2: Borrow<Q2>,
        K3: Borrow<Q3>,
        K4: Borrow<Q4>,
        Q1: Hash + Eq,
        Q2: Hash + Eq,
        Q3: Hash + Eq,
        Q4: Hash + Eq,
    {
        self.map.get(key1).map_or(false, |map| {
            map.get(key2).map_or(false, |map| {
                map.get(key3).map_or(false, |map| map.contains(key4))
            })
        })
    }
}

impl<K1, K2, K3, K4> Default for HashSet4<K1, K2, K3, K4>
where
    K1: Hash + Eq,
    K2: Hash + Eq,
    K3: Hash + Eq,
    K4: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}
