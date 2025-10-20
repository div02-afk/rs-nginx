use std::{ sync::{ Arc, Weak } };

use tokio::sync::RwLock;

use crate::cache::lru::{ Cache, CacheList };

pub async  fn add_after_head(head: &Arc<RwLock<CacheList>>, node: &Arc<RwLock<CacheList>>) {
    let mut head = head.write().await;
    let mut mut_node = node.write().await;
    mut_node.next = None;

    //plxacing current node's prev to old first
    if let Some(old_first) = head.next.take() {
        mut_node.prev = Some(Arc::downgrade(&old_first));
        drop(mut_node);
        //placing head's next to current node
        head.next = Some(node.clone());
        drop(head);

        //placing old first node's next to current node
        old_first.write().await.next = Some(node.to_owned());
        drop(old_first);
    } else {
        head.next = Some(node.clone());
    }
}

pub async  fn move_node_to_head(
    head: &Arc<RwLock<CacheList>>,
    node: &Weak<RwLock<CacheList>>,
    data: Option<&Vec<u8>>
) {
    if let Some(current_node) = node.upgrade() {
        let mut current_mut_node = current_node.write().await; //got the current node

        //update old data
        if let Some(data) = data {
            current_mut_node.cache_entry.data = data.clone();
        }

        //delinking current node
        if let Some(prev_weak) = &current_mut_node.prev {
            // getting prev node's weak ref
            if let Some(prev_rc) = prev_weak.upgrade() {
                let mut prev_node = prev_rc.write().await;
                // now `prev_node` is a mutable ref to the actual previous node
                prev_node.next = current_mut_node.next.clone();
                drop(prev_node);
                if let Some(next_rc) = &current_mut_node.next {
                    let mut next_node = next_rc.write().await;
                    // now `next_node` is a mutable ref to the actual next node
                    next_node.prev = Some(prev_weak.clone());
                    drop(next_node);
                }
            }
        }
        drop(current_mut_node);

        //placing current node to head
        add_after_head(head, &current_node).await
    }
}

//TODO : implement last node purge
pub fn purge(cache: &Cache) {}
