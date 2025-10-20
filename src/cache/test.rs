#[cfg(test)]
mod tests {
    use std::{ path::{ Path, PathBuf }, sync::Arc, thread::spawn };

    use crate::cache::lru::Cache;

    use super::*; // import symbols from parent module
    use tokio::{ self, test };
    #[test]
    async fn test_single_threaded_lru_cache() {
        let mut cache = Cache::new(1024);
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


}
