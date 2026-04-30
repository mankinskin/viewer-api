//! [`Prefetcher`] — capacity-bounded LRU cache with async fetch-on-miss.
//!
//! Designed for viewer-side fetch deduplication: small static datasets
//! (markdown bodies, parsed module trees) are cached per key with simple
//! LRU eviction so navigating away and back is instant.
//!
//! ## Single-flight (best-effort)
//!
//! [`Prefetcher::get_or_fetch`] does not implement true single-flight
//! sharing of in-progress futures (which would require a shared executor
//! and futures-util's `Shared`).  Concurrent calls for the same key while
//! the cache is cold may each invoke the supplied fetcher; both results
//! end up cached but only the second insert is retained.  Callers that
//! need strict de-duplication should serialise their own fetches.

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::future::Future;
use std::hash::Hash;
use std::rc::Rc;

/// LRU cache with `O(1)` `get` / `insert` and capacity-bounded eviction.
///
/// Cloning a `Prefetcher` is cheap — it shares the underlying state via
/// reference counting (`Rc<RefCell<…>>`).  This makes it ergonomic to
/// pass into closures and async blocks.
///
/// `K` must be `Hash + Eq + Clone` (clones are cheap when keys are
/// `String`/`Rc<str>` etc).  `V` must be `Clone` so the cache can hand
/// out copies on hits without invalidating the stored entry.
pub struct Prefetcher<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    inner: Rc<RefCell<PrefetcherInner<K, V>>>,
}

struct PrefetcherInner<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    /// LRU ordering: front = oldest, back = most-recently-used.
    order: VecDeque<K>,
}

impl<K, V> Clone for Prefetcher<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<K, V> Prefetcher<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Creates a new cache that retains at most `capacity` entries.
    ///
    /// A `capacity` of `0` disables caching: every `get_or_fetch` call
    /// will invoke the fetcher and never store a result.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Rc::new(RefCell::new(PrefetcherInner {
                capacity,
                map: HashMap::with_capacity(capacity),
                order: VecDeque::with_capacity(capacity),
            })),
        }
    }

    /// Returns the cached value for `key`, marking it as most-recently-used.
    pub fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.inner.borrow_mut();
        let value = inner.map.get(key).cloned()?;
        // Bump LRU order.
        if let Some(pos) = inner.order.iter().position(|k| k == key) {
            inner.order.remove(pos);
        }
        inner.order.push_back(key.clone());
        Some(value)
    }

    /// Inserts `value` for `key`, evicting the least-recently-used entry
    /// if capacity is exceeded.
    pub fn insert(&self, key: K, value: V) {
        let mut inner = self.inner.borrow_mut();
        if inner.capacity == 0 {
            return;
        }
        // Refresh LRU position.
        if let Some(pos) = inner.order.iter().position(|k| k == &key) {
            inner.order.remove(pos);
        }
        inner.map.insert(key.clone(), value);
        inner.order.push_back(key);
        while inner.order.len() > inner.capacity {
            if let Some(old) = inner.order.pop_front() {
                inner.map.remove(&old);
            }
        }
    }

    /// Number of entries currently cached.
    pub fn len(&self) -> usize {
        self.inner.borrow().map.len()
    }

    /// `true` if the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().map.is_empty()
    }

    /// Returns the cached value for `key` if present, otherwise awaits
    /// `fetcher(key)`, caches the result, and returns it.
    ///
    /// See the module-level docs for single-flight caveats.
    pub async fn get_or_fetch<E, Fut, F>(&self, key: K, fetcher: F) -> Result<V, E>
    where
        F: FnOnce(K) -> Fut,
        Fut: Future<Output = Result<V, E>>,
    {
        if let Some(v) = self.get(&key) {
            return Ok(v);
        }
        let value = fetcher(key.clone()).await?;
        self.insert(key, value.clone());
        Ok(value)
    }
}

impl<K, V> Default for Prefetcher<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Default capacity of 32 entries.
    fn default() -> Self {
        Self::with_capacity(32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let p: Prefetcher<String, u32> = Prefetcher::with_capacity(3);
        p.insert("a".into(), 1);
        p.insert("b".into(), 2);
        assert_eq!(p.get(&"a".into()), Some(1));
        assert_eq!(p.get(&"b".into()), Some(2));
        assert_eq!(p.get(&"missing".into()), None);
    }

    #[test]
    fn lru_eviction_order() {
        let p: Prefetcher<String, u32> = Prefetcher::with_capacity(2);
        p.insert("a".into(), 1);
        p.insert("b".into(), 2);
        // Touch "a" so "b" becomes oldest.
        let _ = p.get(&"a".into());
        p.insert("c".into(), 3);
        assert_eq!(p.get(&"a".into()), Some(1));
        assert_eq!(p.get(&"b".into()), None, "b should have been evicted");
        assert_eq!(p.get(&"c".into()), Some(3));
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn re_inserting_updates_value_and_lru() {
        let p: Prefetcher<String, u32> = Prefetcher::with_capacity(2);
        p.insert("a".into(), 1);
        p.insert("b".into(), 2);
        p.insert("a".into(), 10);
        p.insert("c".into(), 3);
        assert_eq!(p.get(&"a".into()), Some(10));
        assert_eq!(p.get(&"b".into()), None);
        assert_eq!(p.get(&"c".into()), Some(3));
    }

    #[test]
    fn capacity_zero_never_caches() {
        let p: Prefetcher<String, u32> = Prefetcher::with_capacity(0);
        p.insert("a".into(), 1);
        assert_eq!(p.get(&"a".into()), None);
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn cloning_shares_state() {
        let p: Prefetcher<String, u32> = Prefetcher::with_capacity(4);
        let q = p.clone();
        p.insert("a".into(), 1);
        assert_eq!(q.get(&"a".into()), Some(1));
    }

    #[test]
    fn get_or_fetch_caches_miss() {
        // Use a tiny ad-hoc executor: futures with no `await` resolve in poll().
        let p: Prefetcher<String, u32> = Prefetcher::with_capacity(4);
        let fut = p.get_or_fetch::<(), _, _>("k".to_string(), |_| async { Ok(42u32) });
        let v = futures_test_util::block_on(fut).unwrap();
        assert_eq!(v, 42);
        assert_eq!(p.get(&"k".into()), Some(42));
    }

    #[test]
    fn get_or_fetch_hit_skips_fetcher() {
        let p: Prefetcher<String, u32> = Prefetcher::with_capacity(4);
        p.insert("k".into(), 7);
        let fut = p.get_or_fetch::<(), _, _>("k".to_string(), |_| async {
            panic!("fetcher must not run on cache hit")
        });
        assert_eq!(futures_test_util::block_on(fut).unwrap(), 7);
    }

    /// Minimal poll-once executor sufficient for futures that never yield.
    mod futures_test_util {
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        pub fn block_on<F: Future>(mut fut: F) -> F::Output {
            // SAFETY: we never poll past one yield point; create a no-op waker.
            fn raw() -> RawWaker {
                fn no_op(_: *const ()) {}
                fn clone(_: *const ()) -> RawWaker {
                    raw()
                }
                static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
                RawWaker::new(std::ptr::null(), &VTABLE)
            }
            let waker = unsafe { Waker::from_raw(raw()) };
            let mut cx = Context::from_waker(&waker);
            // SAFETY: fut is on the stack and not moved while pinned.
            let fut = unsafe { Pin::new_unchecked(&mut fut) };
            match fut.poll(&mut cx) {
                Poll::Ready(v) => v,
                Poll::Pending => panic!("test future yielded; expected immediate resolution"),
            }
        }
    }
}
