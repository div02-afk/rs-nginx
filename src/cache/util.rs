use std::{ sync::{ Arc, Weak } };

use tokio::sync::RwLock;

use crate::cache::lru::{ Cache, CacheList };

pub async fn add_after_head(head: &Arc<RwLock<CacheList>>, node: &Arc<RwLock<CacheList>>) {
    let mut head = head.write().await;
    let mut mut_node = node.write().await;
    mut_node.next = None;

    //plxacing current node's prev to old first
    if let Some(old_first) = head.next.take() {
        println!("got head ptr");

        mut_node.prev = Some(Arc::downgrade(&old_first));
        drop(mut_node);
        //placing head's next to current node
        head.next = Some(node.clone());
        drop(head);

        //placing old first node's next to current node
        old_first.write().await.next = Some(node.to_owned());
        drop(old_first);
    } else {
        println!("added first node");
        head.next = Some(node.clone());
        drop(head);
    }
}

pub async fn move_node_to_head(
    head: &Arc<RwLock<CacheList>>,
    node: &Weak<RwLock<CacheList>>,
    data: Option<&Vec<u8>>
) {
    if let Some(current_node) = node.upgrade() {
        let head_lock = head.read().await;
        if let Some(head_next) = &head_lock.next {
            // Compare Arc pointers
            if Arc::ptr_eq(head_next, &current_node) {
                drop(head_lock);
                println!("node == head pointer condition");
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

        //delinking current node
        if let Some(prev_weak) = &current_mut_node.prev {
            println!("found prev node");
            // getting prev node's weak ref
            if let Some(prev_rc) = prev_weak.upgrade() {
                let mut prev_node = prev_rc.write().await;
                println!("got prev lock");
                // now `prev_node` is a mutable ref to the actual previous node
                prev_node.next = current_mut_node.next.clone();
                drop(prev_node);
            }
            if let Some(next_rc) = &current_mut_node.next {
                println!("found next node");
                let mut next_node = next_rc.write().await;
                println!("got next lock");
                // now `next_node` is a mutable ref to the actual next node
                next_node.prev = Some(prev_weak.clone());
                drop(next_node);
            }
        }
        drop(current_mut_node);

        //placing current node to head
        add_after_head(head, &current_node).await
    }
}

//TODO : implement last node purge
pub fn purge(cache: &Cache) {}
