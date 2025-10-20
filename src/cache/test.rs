#[cfg(test)]
mod tests {
    use test_log::test;
    use std::{ path::{ Path, PathBuf }, sync::Arc, thread::spawn };
    use crate::cache::lru::Cache;

    use super::*; // import symbols from parent module
    use tokio::{ self, task::JoinHandle };

    #[test(tokio::test)]
    async fn test_single_threaded_lru_cache() {
        let cache = Cache::new(1024);
        let path = "/";
        let key = Path::new(path).to_path_buf();
        let data = b"Hello, world!".to_vec();
        cache.add(&key, &data).await;

        println!("Added to cache");

        let result = cache.get(&key).await;

        if result.is_some() {
            println!("Getting from cache: {:?}", Some(result));
        } else {
            println!("Failed to get results");
        }
    }

    #[test(tokio::test)]
    async fn test_multithreaded_lru_cache() {
        let cache = Arc::new(Cache::new(1024));
        let paths = vec!["/", "/a", "/aa"];
        let mut handles = vec![];
        for path in paths {
            let cloned_cache = cache.clone();

            let handle = tokio::spawn(async move {
                let key = Path::new(path).to_path_buf();
                let data = b"Hello, world!".to_vec();
                cloned_cache.add(&key, &data).await;

                println!("Added {:?} to cloned_cache", key.clone());
                let result = cloned_cache.get(&key).await;

                if result.is_some() {
                    println!("Getting from cache: {:?}", Some(result));
                } else {
                    println!("Failed to get results");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
