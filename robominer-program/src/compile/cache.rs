//! Process-local LRU cache for compiled robot programs.
//!
//! Rally/pool fills often recompile the same AI and player sources within one
//! process; caching by source text avoids repeated parse work without changing
//! simulation behavior.

use std::collections::{HashMap, VecDeque};
#[cfg(not(test))]
use std::sync::{LazyLock, Mutex};
#[cfg(test)]
use std::cell::RefCell;

use crate::types::{CompileError, ExecutableProgram};

const CAPACITY: usize = 256;

type CacheValue = Result<(usize, ExecutableProgram), CompileError>;

struct CompileCache {
    map: HashMap<String, CacheValue>,
    order: VecDeque<String>,
    hits: u64,
    misses: u64,
}

impl CompileCache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            hits: 0,
            misses: 0,
        }
    }

    fn get_or_insert_with(
        &mut self,
        source: &str,
        compile: impl FnOnce() -> CacheValue,
    ) -> CacheValue {
        if let Some(value) = self.map.get(source).cloned() {
            self.touch(source);
            self.hits += 1;
            return value;
        }

        self.misses += 1;
        let value = compile();
        self.insert(source.to_owned(), value.clone());
        value
    }

    fn touch(&mut self, source: &str) {
        if let Some(index) = self.order.iter().position(|entry| entry == source) {
            if let Some(key) = self.order.remove(index) {
                self.order.push_back(key);
            }
        }
    }

    fn insert(&mut self, source: String, value: CacheValue) {
        if self.map.contains_key(&source) {
            self.map.insert(source.clone(), value);
            self.touch(&source);
            return;
        }

        while self.map.len() >= CAPACITY {
            if let Some(evicted) = self.order.pop_front() {
                self.map.remove(&evicted);
            } else {
                break;
            }
        }

        self.order.push_back(source.clone());
        self.map.insert(source, value);
    }

    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

#[cfg(not(test))]
static COMPILE_CACHE: LazyLock<Mutex<CompileCache>> =
    LazyLock::new(|| Mutex::new(CompileCache::new()));

#[cfg(test)]
thread_local! {
    static COMPILE_CACHE: RefCell<CompileCache> = RefCell::new(CompileCache::new());
}

pub(super) fn get_or_insert_with(
    source: &str,
    compile: impl FnOnce() -> CacheValue,
) -> CacheValue {
    #[cfg(not(test))]
    {
        COMPILE_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get_or_insert_with(source, compile)
    }

    #[cfg(test)]
    {
        COMPILE_CACHE.with(|cache| cache.borrow_mut().get_or_insert_with(source, compile))
    }
}

/// Clears the process-local compile cache (intended for tests).
pub fn clear_compile_cache() {
    #[cfg(not(test))]
    {
        COMPILE_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    #[cfg(test)]
    {
        COMPILE_CACHE.with(|cache| cache.borrow_mut().clear());
    }
}

/// Returns `(hits, misses)` since the last clear (intended for tests).
pub fn compile_cache_stats() -> (u64, u64) {
    #[cfg(not(test))]
    {
        let cache = COMPILE_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (cache.hits, cache.misses)
    }

    #[cfg(test)]
    {
        COMPILE_CACHE.with(|cache| {
            let cache = cache.borrow();
            (cache.hits, cache.misses)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::{compile_executable_source, verify_source};
    use crate::types::CompileError;

    #[test]
    fn second_compile_of_same_source_is_a_cache_hit() {
        clear_compile_cache();

        let source = "mine(); move(1);";
        let first = compile_executable_source(source).expect("source should compile");
        let (hits_after_miss, misses_after_miss) = compile_cache_stats();
        assert_eq!(misses_after_miss, 1);
        assert_eq!(hits_after_miss, 0);

        let second = compile_executable_source(source).expect("cached compile should succeed");
        let (hits, misses) = compile_cache_stats();
        assert_eq!(misses, 1);
        assert_eq!(hits, 1);
        assert_eq!(first, second);
    }

    #[test]
    fn compile_errors_are_cached() {
        clear_compile_cache();

        let source = "this is not valid robot code !!!";
        let first = compile_executable_source(source).expect_err("invalid source");
        let second = compile_executable_source(source).expect_err("cached error");
        assert_eq!(first.to_string(), second.to_string());

        let (hits, misses) = compile_cache_stats();
        assert_eq!(misses, 1);
        assert_eq!(hits, 1);
    }

    #[test]
    fn verify_source_shares_cache_with_compile() {
        clear_compile_cache();

        let source = "scan(); mine();";
        assert!(verify_source(source).verified);
        let (hits_after_miss, misses) = compile_cache_stats();
        assert_eq!(misses, 1);
        assert_eq!(hits_after_miss, 0);

        compile_executable_source(source).expect("should hit cache");
        let (hits, misses_after_hit) = compile_cache_stats();
        assert_eq!(misses_after_hit, 1);
        assert_eq!(hits, 1);
    }

    #[test]
    fn eviction_recompiles_oldest_entry() {
        clear_compile_cache();

        for index in 0..CAPACITY {
            let source = format!("int x{index} = {index}; mine();");
            compile_executable_source(&source).expect("unique sources should compile");
        }
        let (_, misses_full) = compile_cache_stats();
        assert_eq!(misses_full, CAPACITY as u64);

        // Touch a mid entry so the first inserted source is oldest.
        compile_executable_source("int x1 = 1; mine();").expect("should hit");
        let (hits_before_evict, _) = compile_cache_stats();

        let overflow = format!("int x{} = 0; mine();", CAPACITY);
        compile_executable_source(&overflow).expect("overflow source should compile");

        // Oldest (index 0) should have been evicted and miss again.
        compile_executable_source("int x0 = 0; mine();").expect("evicted source recompiles");
        let (hits, misses) = compile_cache_stats();
        assert!(misses > misses_full);
        assert!(hits >= hits_before_evict);
    }

    #[test]
    fn insert_overwrites_existing_without_growing() {
        let mut cache = CompileCache::new();
        let value = Err(CompileError::new("boom"));
        cache.insert("a".to_owned(), value.clone());
        cache.insert("a".to_owned(), value);
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.order.len(), 1);
    }
}
