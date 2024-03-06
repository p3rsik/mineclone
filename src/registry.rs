use std::hash::Hash;

use bevy::{prelude::*, utils::HashMap};

use crate::block::{Block, BlockId};

#[derive(Resource)]
struct Registry<K, V> {
    registry: HashMap<K, V>,
}

pub type BlockRegistry = Registry<BlockId, Handle<Block>>;

impl<K: PartialEq + Eq + Hash, V> Registry<K, V> {
    pub fn new() -> Registry<K, V> {
        Registry {
            registry: HashMap::new(),
        }
    }

    pub fn register(&mut self, k: K, v: V) {
        self.registry.insert(k, v);
    }

    pub fn remove(&mut self, k: &K) {
        self.registry.remove(k);
    }
}

impl<K: PartialEq + Eq + Hash, V> Default for Registry<K, V> {
    fn default() -> Self {
        Registry::new()
    }
}
