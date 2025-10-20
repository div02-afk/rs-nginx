use std::{ cell::RefCell, collections::HashMap, path::PathBuf, rc::{ Rc, Weak }, sync::{Arc} };

use tokio::sync::{RwLock};


use crate::cache::util::{ add_after_head, move_node_to_head, purge };

#[derive(Default, Debug)]
pub struct CacheEntry {
    pub data: Vec<u8>,
}

#[derive(Default, Debug)]
pub struct CacheList {
    pub key: PathBuf,
    pub cache_entry: CacheEntry,
    pub next: Option<Rc<RefCell<CacheList>>>,
    pub prev: Option<Weak<RefCell<CacheList>>>,
}

pub struct Cache {
    cache_map: RwLock<HashMap<PathBuf, Weak<RefCell<CacheList>>>>,
    cache_ll_head: Rc<RefCell<CacheList>>,
    cache_ll_tail: Rc<RefCell<CacheList>>,
    capacity: usize, //in b
    data_size: usize, // in b
}

impl Cache {
    pub fn new(capacity: usize) -> Cache {
        Cache {
            cache_map: RwLock::new(HashMap::new()),
            cache_ll_head: Rc::new(RefCell::new(CacheList::default())),
            cache_ll_tail: Rc::new(RefCell::new(CacheList::default())),
            capacity :capacity * 1024,
            data_size: 0,
        }
    }
    pub async fn get(&mut self, key: &PathBuf) -> Option<Vec<u8>> {
        let cache_map = self.cache_map.read().await;

        if let Some(weak_cache_ll_entry) = cache_map.get(key) {
            if let Some(cache_ll_entry) = weak_cache_ll_entry.upgrade() {
                move_node_to_head(&self.cache_ll_head, weak_cache_ll_entry, None);
                let node = cache_ll_entry.borrow();
                return Some(node.cache_entry.data.clone());
            }
        }

        None
    }
    pub async fn add(&mut self, key: &PathBuf, data: &Vec<u8>) {
        let mut cache_map = self.cache_map.write().await;

        if let Some(node) = cache_map.get(key) {
            move_node_to_head(&self.cache_ll_head, node, Some(data));
        } else {
            let mut node = CacheList::default();
            node.cache_entry.data = data.to_vec();
            node.key = key.to_path_buf();
            let rc_node = Rc::new(RefCell::new(node));

            cache_map.insert(key.to_path_buf(), Rc::downgrade(&rc_node));
            drop(cache_map);

            add_after_head(&self.cache_ll_head, &rc_node);
            self.data_size += data.len();
            purge(self);
        }
    }
}
