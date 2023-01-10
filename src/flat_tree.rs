use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
};

pub struct FlatTree<K, V> {
    pub nodes: BTreeMap<K, V>,
    pub roots: BTreeSet<K>,
}

impl<K, V> Default for FlatTree<K, V> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            roots: Default::default(),
        }
    }
}

impl<K, V> FlatTree<K, V>
where
    K: Clone + Eq + Ord,
{
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.nodes.get(k)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.nodes.get_mut(k)
    }

    pub fn insert_root(&mut self, k: K, v: V) -> Option<V> {
        let x = self.insert(k.clone(), v);
        self.roots.insert(k);

        x
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.nodes.insert(k, v)
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        let x = self.nodes.remove(k);
        self.roots.remove(k);

        x
    }

    pub fn contains_node<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.nodes.contains_key(k)
    }

    pub fn is_root<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.roots.contains(k)
    }

    pub fn roots(&self) -> impl Iterator<Item = (&K, &V)> {
        self.roots
            .iter()
            .filter_map(|k| self.get(k).map(|v| (k, v)))
    }
}
