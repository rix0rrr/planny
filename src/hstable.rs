#![allow(dead_code)]
use std::{borrow::Borrow, collections::HashMap, marker::PhantomData};

use serde::{ser::SerializeSeq, Deserialize, Serialize};

pub trait Hashable {
    type Coll: Coll<Self>
    where
        Self: Sized;
    type HashKey: ?Sized + std::hash::Hash + Eq;

    fn hash_key(&self) -> &Self::HashKey;
}

pub trait Sortable {
    type SortKey: ?Sized + Ord;
    fn sort_key(&self) -> &Self::SortKey;
}

#[derive(Debug)]
pub struct HSTable<T>
where
    T: Hashable,
    T::HashKey: std::hash::Hash + Eq + Clone,
{
    elements: HashMap<T::HashKey, Vec<T>>,
}

impl<T: Hashable> HSTable<T>
where
    T: Hashable,
    T::HashKey: std::hash::Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            elements: Default::default(),
        }
    }

    pub fn from_iter(xs: impl IntoIterator<Item = T>) -> Self {
        let mut ret = Self::new();
        xs.into_iter().for_each(|x| ret.insert(x));
        ret
    }

    pub fn insert(&mut self, x: T) {
        let pk = x.hash_key();
        let Some(existing) = self.elements.get_mut(pk) else {
            self.elements.insert(pk.to_owned(), vec![x]);
            return;
        };

        T::Coll::insert(existing, x)
    }

    pub fn iter_all(&self) -> impl Iterator<Item = &T> + '_ {
        self.elements.values().flat_map(|xs| xs.iter())
    }
}

impl<T> HSTable<T>
where
    T: Hashable,
    T::HashKey: std::hash::Hash + Eq + Clone,
{
    pub fn get1<Q: ?Sized>(&self, pk: &Q) -> Option<&T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        self.elements
            .get(pk)
            .map(|existing| {
                assert!(existing.len() <= 1);
                existing
            })
            .and_then(|existing| existing.get(0))
    }

    pub fn get1_mut<Q: ?Sized>(&mut self, pk: &Q) -> Option<&mut T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        self.elements
            .get_mut(pk)
            .map(|existing| {
                assert!(existing.len() <= 1);
                existing
            })
            .and_then(|existing| existing.get_mut(0))
    }

    pub fn remove1<Q: ?Sized>(&mut self, pk: &Q) -> Option<T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        self.elements.get_mut(pk).and_then(|existing| {
            if !existing.is_empty() {
                Some(existing.remove(0))
            } else {
                None
            }
        })
    }
}

impl<T> HSTable<T>
where
    T: Hashable + Sortable,
    T::HashKey: std::hash::Hash + Eq + Clone,
{
    pub fn get2<Q: ?Sized, R: ?Sized>(&self, pk: &Q, sk: &R) -> Option<&T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
        T::SortKey: Borrow<R>,
        R: Ord,
    {
        self.elements.get(pk).and_then(|existing| {
            match existing.binary_search_by(|a| a.sort_key().borrow().cmp(sk)) {
                Ok(i) => existing.get(i),
                Err(_) => None,
            }
        })
    }

    pub fn get2_mut<Q: ?Sized, R: ?Sized>(&mut self, pk: &Q, sk: &R) -> Option<&mut T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
        T::SortKey: Borrow<R>,
        R: Ord,
    {
        self.elements.get_mut(pk).and_then(|existing| {
            match existing.binary_search_by(|a| a.sort_key().borrow().cmp(sk)) {
                Ok(i) => existing.get_mut(i),
                Err(_) => None,
            }
        })
    }

    pub fn remove2<Q: ?Sized, R: ?Sized>(&mut self, pk: &Q, sk: &R) -> Option<T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
        T::SortKey: Borrow<R>,
        R: Ord,
    {
        self.elements.get_mut(pk).and_then(|existing| {
            match existing.binary_search_by(|a| a.sort_key().borrow().cmp(sk)) {
                Ok(i) => Some(existing.remove(i)),
                Err(_) => None,
            }
        })
    }

    pub fn get_many<Q: ?Sized>(&self, pk: &Q) -> impl Iterator<Item = &T> + '_
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        self.elements.get(pk).into_iter().flat_map(|xs| xs.iter())
    }

    pub fn into_many<Q: ?Sized>(mut self, pk: &Q) -> impl Iterator<Item = T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        self.elements
            .remove_entry(pk)
            .into_iter()
            .flat_map(|(_, xs)| xs.into_iter())
    }
}

impl<T: Serialize> Serialize for HSTable<T>
where
    T: Hashable,
    T::HashKey: std::hash::Hash + Eq + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        for e in self.iter_all() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for HSTable<T>
where
    T: Hashable,
    T::HashKey: std::hash::Hash + Eq + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let x: Vec<T> = Deserialize::deserialize(deserializer)?;
        Ok(HSTable::from_iter(x))
    }
}

impl<T> Default for HSTable<T>
where
    T: Hashable,
    T::HashKey: std::hash::Hash + Eq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait Coll<T> {
    fn insert(vec: &mut Vec<T>, x: T);
}

pub struct HashColl<T> {
    phantom: PhantomData<T>,
}

impl<T> Coll<T> for HashColl<T> {
    fn insert(vec: &mut Vec<T>, x: T) {
        if vec.is_empty() {
            vec.push(x)
        } else {
            vec[0] = x;
        }
    }
}

pub struct SortColl<T> {
    phantom: PhantomData<T>,
}

impl<T> Coll<T> for SortColl<T>
where
    T: Sortable,
{
    fn insert(vec: &mut Vec<T>, x: T) {
        let sk = x.sort_key();
        match vec.binary_search_by(|a| a.sort_key().borrow().cmp(sk)) {
            Ok(i) => vec[i] = x,
            Err(i) => vec.insert(i, x),
        }
    }
}
