use std::sync::{Arc, Weak};

use tokio::sync::RwLock;

use crate::cache::lru::{Cache, CacheList};

pub async fn add_after_head(head: &Arc<RwLock<CacheList>>, node: &Arc<RwLock<CacheList>>) {
    let mut head = head.write().await;
    let mut current_node_mut = node.write().await;
    current_node_mut.next = None;

    //placing current node's prev to old first
    if let Some(old_first) = head.next.take() {
        current_node_mut.prev = Some(Arc::downgrade(&old_first));
        drop(current_node_mut);
        //placing head's next to current node
        head.next = Some(node.clone());
        drop(head);

        //placing old first node's next to current node
        old_first.write().await.next = Some(node.to_owned());
        drop(old_first);
    } else {
        head.next = Some(node.clone());
        drop(head);
    }
}

pub async fn move_node_to_head(
    head: &Arc<RwLock<CacheList>>,
    tail: &Arc<RwLock<CacheList>>,
    node: &Weak<RwLock<CacheList>>,
    data: Option<&Vec<u8>>,
) {
    if let Some(current_node) = node.upgrade() {
        let head_lock = head.read().await;
        if let Some(head_next) = &head_lock.next {
            // Compare Arc pointers
            if Arc::ptr_eq(head_next, &current_node) {
                drop(head_lock);
                // Already at head, just update data if needed
                if let Some(data) = data {
                    current_node.write().await.cache_entry.data = data.clone();
                }
                return; // Early exit!
            }
        }
        drop(head_lock);

        let mut current_mut_node = current_node.write().await; //got the current node

        //update old data
        if let Some(data) = data {
            current_mut_node.cache_entry.data = data.clone();
        }

        //delinking current node from list
        if let Some(prev_weak) = &current_mut_node.prev {
            // getting prev node's weak ref
            if let Some(prev_rc) = prev_weak.upgrade() {
                let mut prev_node = prev_rc.write().await;

                // now `prev_node` is a mutable ref to the actual previous node
                prev_node.next = current_mut_node.next.clone();
                drop(prev_node);
            }
            if let Some(next_rc) = &current_mut_node.next {
                let mut next_node = next_rc.write().await;

                // now `next_node` is a mutable ref to the actual next node
                next_node.prev = Some(prev_weak.clone());
                drop(next_node);
            }
            drop(current_mut_node);
        } else {
            // first node -> add tail's next to current node
            drop(current_mut_node);
            let mut tail_lock = tail.write().await;
            tail_lock.next = Some(current_node.clone());
            drop(tail_lock);
        }

        //placing current node to head
        add_after_head(head, &current_node).await
    }
}

//TODO : implement last node purge
pub async fn purge(cache: &Cache) {
    let data_size_lock = cache.data_size.read().await;
    if *data_size_lock < cache.capacity {
        return;
    }
    drop(data_size_lock);

    let mut tail_lock = cache.cache_ll_tail.write().await;

    if let Some(last_node) = tail_lock.next.take() {
        let mut last_node_lock = last_node.write().await;
        tail_lock.next = last_node_lock.next.take(); // link tail to last node's next
        let mut cache_map = cache.cache_map.write().await;
        cache_map.remove(&last_node_lock.key); // remove from map
        drop(cache_map);

        //remove all links
        last_node_lock.next = None;
        last_node_lock.prev = None;
        let mut data_size_lock = cache.data_size.write().await;
        *data_size_lock -= last_node_lock.cache_entry.data.len();
        drop(last_node_lock);
    }
}
