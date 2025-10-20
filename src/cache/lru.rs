use std::{ collections::HashMap, path::PathBuf, sync::{ Arc, Weak } };

use tokio::sync::{ RwLock };

use crate::cache::util::{ add_after_head, move_node_to_head, purge };

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
    cache_map: RwLock<HashMap<PathBuf, Weak<RwLock<CacheList>>>>,
    cache_ll_head: Arc<RwLock<CacheList>>,
    cache_ll_tail: Arc<RwLock<CacheList>>,
    capacity: usize, //in b
    data_size: usize, // in b
}

impl Cache {
    pub fn new(capacity: usize) -> Cache {
        Cache {
            cache_map: RwLock::new(HashMap::new()),
            cache_ll_head: Arc::new(RwLock::new(CacheList::default())),
            cache_ll_tail: Arc::new(RwLock::new(CacheList::default())),
            capacity: capacity * 1024,
            data_size: 0,
        }
    }
    pub async fn get(&self, key: &PathBuf) -> Option<Vec<u8>> {
         println!("getting cached data for {:?}",key);
        let cache_map = self.cache_map.read().await;
        println!("acquired cachemap log");
        if let Some(weak_cache_ll_entry) = cache_map.get(key).cloned() {
            drop(cache_map); 
            if let Some(cache_ll_entry) = weak_cache_ll_entry.upgrade() {
                move_node_to_head(&self.cache_ll_head, &weak_cache_ll_entry, None).await;
                let node = cache_ll_entry.read().await;
                return Some(node.cache_entry.data.clone());
            }
        }

        None
    }
    pub async fn add(&self, key: &PathBuf, data: &Vec<u8>) {
        let mut cache_map = self.cache_map.write().await;

        if let Some(node) = cache_map.get(key).clone() {
            move_node_to_head(&self.cache_ll_head, node, Some(data)).await;
        } else {
            let mut node = CacheList::default();
            node.cache_entry.data = data.to_vec();
            node.key = key.to_path_buf();
            let rc_node = Arc::new(RwLock::new(node));

            cache_map.insert(key.to_path_buf(), Arc::downgrade(&rc_node));
            drop(cache_map);

            add_after_head(&self.cache_ll_head, &rc_node).await;
            // purge(self);
        }
    }
}
