use std::{
    collections::HashMap,
    hash::Hash,
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::world_generation::{
    chunk_generation::country::country_cache::CountryCache,
    generation_options::GenerationOptions,
};

pub trait GenerationCacheItem<K: Copy + Eq + Hash> {
    fn generate(
        key: K,
        generation_options: &GenerationOptions,
        country_cache: &mut CountryCache,
    ) -> Self;
}

pub struct GenerationCache<K: Copy + Eq + Hash, T: GenerationCacheItem<K>> {
    cache_lock: RwLock<HashMap<K, Arc<RwLock<Option<Arc<T>>>>>>,
}

impl<K: Copy + Eq + Hash, T: GenerationCacheItem<K>> GenerationCache<K, T> {
    pub fn new() -> Self {
        Self {
            cache_lock: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_cache_entry(
        &self,
        key: K,
        generation_options: &GenerationOptions,
        country_cache: &mut CountryCache,
    ) -> Arc<T> {
        self.get_generated_cache_entry(
            self.get_hash_lock_entry(key),
            key,
            generation_options,
            country_cache,
        )
    }

    pub fn try_get_entry_no_lock(&self, key: K) -> Option<Arc<T>> {
        match self.cache_lock.try_read() {
            Ok(read) => {
                let entry = read.get(&key)?;
                match entry.try_read() {
                    Ok(read) => match read.deref() {
                        None => None,
                        Some(t) => Some(t.clone()),
                    },
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    fn get_hash_lock_entry(&self, key: K) -> Arc<RwLock<Option<Arc<T>>>> {
        let read = self.cache_lock.read().unwrap();
        match read.get(&key) {
            None => {
                drop(read);
                let mut write = self.cache_lock.write().unwrap();
                let result = match write.get(&key) {
                    None => {
                        let lock = Arc::new(RwLock::new(None));
                        write.insert(key, lock);
                        write.get(&key).unwrap().clone()
                    }
                    Some(cache) => cache.clone(),
                };
                drop(write);
                result
            }
            Some(cache) => cache.clone(),
        }
    }

    fn get_generated_cache_entry(
        &self,
        hash_lock_entry: Arc<RwLock<Option<Arc<T>>>>,
        key: K,
        generation_options: &GenerationOptions,
        country_cache: &mut CountryCache,
    ) -> Arc<T> {
        let read = hash_lock_entry.read().unwrap();
        match read.deref() {
            None => {
                drop(read);
                let mut write = hash_lock_entry.write().unwrap();
                match write.deref() {
                    None => write
                        .insert(Arc::new(T::generate(
                            key,
                            generation_options,
                            country_cache,
                        )))
                        .clone(),
                    Some(country_cache) => country_cache.clone(),
                }
            }
            Some(country_cache) => country_cache.clone(),
        }
    }
}
