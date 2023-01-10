//! A common abstraction of a 'flat' tree which tracks the nodes with an ordered map and the set of
//! root nodes.
//!
//! This pattern is common with implementors of [`crate::Consume`] so is presented here, but is by
//! no means a feature complete implementation.
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
};

/// A common abstraction of a 'flat' tree which tracks the nodes with an ordered map and the set of
/// root nodes.
///
/// Note that the data structures use ordered maps and sets which work well with the incrementing
/// [`crate::Id`] key.
pub struct FlatTree<K, V> {
    /// The nodes.
    pub nodes: BTreeMap<K, V>,
    /// The set of root nodes.
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
    /// Get a node's value using the key.
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.nodes.get(k)
    }

    /// Get a node's value (mutable reference) using the key.
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.nodes.get_mut(k)
    }

    /// Insert a root node.
    pub fn insert_root(&mut self, k: K, v: V) -> Option<V> {
        let x = self.insert(k.clone(), v);
        self.roots.insert(k);

        x
    }

    /// Insert a node.
    ///
    /// If the node is a root node, [`insert_root`] should be used.
    ///
    /// [`insert_root`]: Self::insert_root
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.nodes.insert(k, v)
    }

    /// Remove a node with the key.
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        let x = self.nodes.remove(k);
        self.roots.remove(k);

        x
    }

    /// Does the structure contain a node with the key.
    pub fn contains_node<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.nodes.contains_key(k)
    }

    /// Check if a key is a root node.
    pub fn is_root<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Ord,
    {
        self.roots.contains(k)
    }

    /// An iterator over the roots, fetching the nodes' values.
    pub fn roots(&self) -> impl Iterator<Item = (&K, &V)> {
        self.roots
            .iter()
            .filter_map(|k| self.get(k).map(|v| (k, v)))
    }
}
