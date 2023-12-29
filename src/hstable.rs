use std::{borrow::Borrow, collections::HashMap, marker::PhantomData};

use serde::{ser::SerializeSeq, Deserialize, Serialize};

pub trait HashSortable {
    type HashKey: ?Sized;
    type SortKey: ?Sized;
    fn key(&self) -> (&Self::HashKey, &Self::SortKey);
}

#[derive(Debug)]
pub struct HSTable<T>
where
    T: HashSortable,
    T::HashKey: std::hash::Hash + Eq + Clone,
    T::SortKey: Ord,
{
    elements: HashMap<T::HashKey, Vec<T>>,
    phantom: PhantomData<T::SortKey>,
}

impl<T> HSTable<T>
where
    T: HashSortable,
    T::HashKey: std::hash::Hash + Eq + Clone,
    T::SortKey: Ord,
{
    pub fn new() -> Self {
        Self {
            elements: Default::default(),
            phantom: PhantomData,
        }
    }

    pub fn from_iter(xs: impl IntoIterator<Item = T>) -> Self {
        let mut ret = Self::new();
        xs.into_iter().for_each(|x| ret.insert(x));
        ret
    }

    pub fn insert(&mut self, x: T) {
        let (pk, sk) = x.key();
        let Some(existing) = self.elements.get_mut(pk) else {
            self.elements.insert(pk.to_owned(), vec![x]);
            return;
        };

        match existing.binary_search_by(|a| a.key().1.cmp(sk)) {
            Ok(i) => {
                existing[i] = x;
            }
            Err(i) => {
                existing.insert(i, x);
            }
        }
    }

    pub fn get<Q: ?Sized, R: ?Sized>(&self, pk: &Q, sk: &R) -> Option<&T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
        R: Borrow<T::SortKey>,
    {
        self.elements.get(pk).and_then(|existing| {
            match existing.binary_search_by(|a| a.key().1.cmp(sk.borrow())) {
                Ok(i) => existing.get(i),
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

    pub fn get_mut<Q: ?Sized, R: ?Sized>(&mut self, pk: &Q, sk: &R) -> Option<&mut T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
        T::SortKey: Borrow<R>,
        R: Ord,
    {
        self.elements.get_mut(pk).and_then(|existing| {
            match existing.binary_search_by(|a| a.key().1.borrow().cmp(sk)) {
                Ok(i) => existing.get_mut(i),
                Err(_) => None,
            }
        })
    }

    pub fn remove<Q: ?Sized, R: ?Sized>(&mut self, pk: &Q, sk: &R) -> Option<T>
    where
        T::HashKey: Borrow<Q>,
        Q: std::hash::Hash + Eq,
        T::SortKey: Borrow<R>,
        R: Ord,
    {
        self.elements.get_mut(pk).and_then(|existing| {
            match existing.binary_search_by(|a| a.key().1.borrow().cmp(sk)) {
                Ok(i) => Some(existing.remove(i)),
                Err(_) => None,
            }
        })
    }

    pub fn iter_all(&self) -> impl Iterator<Item = &T> + '_ {
        self.elements.values().flat_map(|xs| xs.iter())
    }
}

impl<T: Serialize> Serialize for HSTable<T>
where
    T: HashSortable,
    T::HashKey: std::hash::Hash + Eq + Clone,
    T::SortKey: Ord,
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
    T: HashSortable,
    T::HashKey: std::hash::Hash + Eq + Clone,
    T::SortKey: Ord,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let x: Vec<T> = Deserialize::deserialize(deserializer)?;
        Ok(HSTable::from_iter(x))
    }
}

impl<T: HashSortable> Default for HSTable<T>
where
    T: HashSortable,
    T::HashKey: std::hash::Hash + Eq + Clone,
    T::SortKey: Ord,
{
    fn default() -> Self {
        Self::new()
    }
}
