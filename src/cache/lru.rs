use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Weak},
};

use tokio::sync::RwLock;

use crate::cache::util::{add_after_head, move_node_to_head, purge};

#[derive(Default, Debug)]
pub struct CacheEntry {
    pub data: Vec<u8>,
}

#[derive(Default, Debug)]
pub struct CacheList {
    pub key: PathBuf,
    pub cache_entry: CacheEntry,
    pub next: Option<Arc<RwLock<CacheList>>>,
    pub prev: Option<Weak<RwLock<CacheList>>>,
}

pub struct Cache {
    pub cache_map: RwLock<HashMap<PathBuf, Weak<RwLock<CacheList>>>>,
    pub cache_ll_head: Arc<RwLock<CacheList>>,
    pub cache_ll_tail: Arc<RwLock<CacheList>>,
    pub capacity: usize,          //in b
    pub data_size: RwLock<usize>, // in b
}

impl CacheEntry {
    pub fn new(data: Vec<u8>) -> CacheEntry {
        CacheEntry { data }
    }
}

impl CacheList {
    pub fn new(key: PathBuf, cache_entry: CacheEntry) -> CacheList {
        CacheList {
            key,
            cache_entry,
            next: None,
            prev: None,
        }
    }
}

impl Cache {
    pub fn new(capacity: usize) -> Cache {
        Cache {
            cache_map: RwLock::new(HashMap::new()),
            cache_ll_head: Arc::new(RwLock::new(CacheList::default())),
            cache_ll_tail: Arc::new(RwLock::new(CacheList::default())),
            capacity: capacity * 1024,
            data_size: RwLock::new(0),
        }
    }
    pub async fn get(&self, key: &PathBuf) -> Option<Vec<u8>> {
        if self.capacity == 0 {
            return None;
        }
        let cache_map = self.cache_map.read().await;
        if let Some(weak_cache_ll_entry) = cache_map.get(key).cloned() {
            drop(cache_map);
            if let Some(cache_ll_entry) = weak_cache_ll_entry.upgrade() {
                move_node_to_head(
                    &self.cache_ll_head,
                    &self.cache_ll_tail,
                    &weak_cache_ll_entry,
                    None,
                )
                .await;
                let node = cache_ll_entry.read().await;
                return Some(node.cache_entry.data.clone());
            }
        }

        None
    }
    pub async fn add(&self, key: &PathBuf, data: &Vec<u8>) {
        if self.capacity == 0 {
            return;
        }

        let mut data_size_lock = self.data_size.write().await;
        *data_size_lock += data.len();
        drop(data_size_lock);

        let mut cache_map = self.cache_map.write().await;

        if let Some(node) = cache_map.get(key) {
            move_node_to_head(&self.cache_ll_head, &self.cache_ll_tail, node, Some(data)).await;
        } else {
            let node = CacheList::new(key.to_path_buf(), CacheEntry::new(data.to_vec()));

            let arc_node = Arc::new(RwLock::new(node));

            cache_map.insert(key.to_path_buf(), Arc::downgrade(&arc_node));
            drop(cache_map);

            add_after_head(&self.cache_ll_head, &arc_node).await;
            purge(self).await;
        }
    }
}
