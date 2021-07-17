/*
The MIT License (MIT)
Copyright © 2021 Incomplete <incomplete@aixon.co>
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”),
to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
use std::{
    cmp::Ordering, collections::HashMap, fmt, hash::Hash, iter::DoubleEndedIterator, ops::Index,
    vec::Vec,
};

type ExtractComparable<V, C> = fn(&V) -> C;

/// An `OrderedMap` is like a `std::collections::HashMap`,
/// but it is sorted according to the value in descending order.
/// It doesn't require the value of the map, `V`, to be comparable,
/// the comparison of the value is done on `C`,
/// which is the return value of `extract_comparable(&V)`.
#[derive(Clone)]
pub struct OrderedMap<K, V, C> {
    map: HashMap<K, V>,

    descending_pairs: Vec<(K, C)>,

    extract_comparable: ExtractComparable<V, C>,
}

impl<K: fmt::Debug, V: fmt::Debug, C: fmt::Debug> fmt::Debug for OrderedMap<K, V, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderedMap")
            .field("map", &self.map)
            .field("descending_pairs", &self.descending_pairs)
            .finish()
    }
}

pub struct DescendingKeys<'a, K: 'a, C: 'a> {
    inner: std::slice::Iter<'a, (K, C)>,
}

impl<'a, K: 'a, C: 'a> Iterator for DescendingKeys<'a, K, C> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            None => None,
            Some((k, _)) => Some(k),
        }
    }
}

impl<'a, K: 'a, C: 'a> DoubleEndedIterator for DescendingKeys<'a, K, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.inner.next_back() {
            None => None,
            Some((k, _)) => Some(k),
        }
    }
}

pub struct DescendingValues<'a, K, V, C> {
    map: &'a HashMap<K, V>,
    keys: DescendingKeys<'a, K, C>,
}

impl<'a, K, V, C> Iterator for DescendingValues<'a, K, V, C>
where
    K: Eq + Hash,
{
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        match self.keys.next() {
            None => None,
            Some(k) => Some(self.map.index(k)),
        }
    }
}

impl<'a, K, V, C> DoubleEndedIterator for DescendingValues<'a, K, V, C>
where
    K: Eq + Hash,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.keys.next_back() {
            None => None,
            Some(k) => Some(self.map.index(k)),
        }
    }
}

pub struct DescendingItems<'a, K, V, C> {
    map: &'a HashMap<K, V>,
    keys: DescendingKeys<'a, K, C>,
}

impl<'a, K, V, C> Iterator for DescendingItems<'a, K, V, C>
where
    K: Eq + Hash,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.keys.next() {
            None => None,
            Some(k) => Some((k, self.map.index(k))),
        }
    }
}

impl<'a, K, V, C> DoubleEndedIterator for DescendingItems<'a, K, V, C>
where
    K: Eq + Hash,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.keys.next_back() {
            None => None,
            Some(k) => Some((k, self.map.index(k))),
        }
    }
}

impl<'a, K: 'a, V: 'a, C: 'a> OrderedMap<K, V, C>
where
    K: Eq + Hash + Copy,
    C: Ord,
{
    /// The function `extract_comparable` is used to convert the value of type `&V`
    /// to something comparable of type `C`
    pub fn new(extract_comparable: ExtractComparable<V, C>) -> Self {
        OrderedMap {
            map: HashMap::new(),
            descending_pairs: vec![],
            extract_comparable,
        }
    }
    pub fn compare(&self, key1: K, key2: K) -> Ordering {
        match (self.map.get(&key1), self.map.get(&key2)) {
            (Some(idx1), Some(idx2)) => {
                let idx1 = (self.extract_comparable)(idx1);
                let idx2 = (self.extract_comparable)(idx2);
                return idx1.cmp(&idx2);
            }
            (None, _) => return Ordering::Greater,
            (_, None) => return Ordering::Less,
        }
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Keys of this map in descending order
    pub fn descending_keys(&'a self) -> DescendingKeys<'a, K, C> {
        DescendingKeys {
            inner: self.descending_pairs.iter(),
        }
    }

    /// Values of this map in descending order
    pub fn descending_values(&'a self) -> DescendingValues<'a, K, V, C> {
        DescendingValues {
            map: &self.map,
            keys: self.descending_keys(),
        }
    }

    /// (K, V) pairs of this map in descending order
    pub fn descending_items(&'a self) -> DescendingItems<'a, K, V, C> {
        DescendingItems {
            map: &self.map,
            keys: self.descending_keys(),
        }
    }

    fn insert_into_pairs(&mut self, k: K, c: C) {
        let mut insert_index = None;
        for (i, (_ek, ec)) in self.descending_pairs.iter().enumerate() {
            if &c >= ec {
                insert_index = Some(i);
                break;
            }
        }
        let idx = match insert_index {
            None => self.descending_pairs.len(),
            Some(i) => i,
        };
        self.descending_pairs.insert(idx, (k, c));
    }

    /// Insert a new key-value pair to the map,
    /// the old value is returned as `Option<V>`
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let new_c = (self.extract_comparable)(&v);
        match self.map.insert(k, v) {
            None => {
                self.insert_into_pairs(k, new_c);
                None
            }
            Some(v) => {
                remove_from_pairs(&mut self.descending_pairs, &k);
                self.insert_into_pairs(k, new_c);
                Some(v)
            }
        }
    }

    /// Remove a key-value pair from the map
    pub fn remove(&mut self, k: &K) -> Option<V> {
        match self.map.remove(k) {
            None => None,
            Some(v) => {
                remove_from_pairs(&mut self.descending_pairs, k);
                Some(v)
            }
        }
    }
}

fn remove_from_pairs<K, C>(pairs: &mut Vec<(K, C)>, k: &K) -> bool
where
    K: Eq,
{
    let mut removed = false;
    for i in 0..pairs.len() {
        if pairs[i].0 == *k {
            pairs.remove(i);
            removed = true;
            break;
        }
    }
    removed
}
