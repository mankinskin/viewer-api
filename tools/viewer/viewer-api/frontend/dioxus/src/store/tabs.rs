//! [`TabsStore`] — generic tab list with active-tab tracking.
//!
//! Two layers:
//!
//! 1. [`TabsStateInner`] — pure logic (no Dioxus). Easily unit-testable.
//! 2. [`TabsStore`] — Dioxus-friendly wrapper holding the inner state in
//!    `Signal`s so changes trigger re-renders.
//!
//! Both layers are generic over a per-tab payload `T` (typically a small
//! `Clone` struct holding state needed to render the tab body).
use std::collections::HashSet;

use dioxus::prelude::*;

/// One tab entry — id-keyed, with an arbitrary `payload`.
///
/// `id` must be unique within a store; it is used for selection,
/// closing, and URL syncing.
#[derive(Clone, PartialEq, Debug)]
pub struct Tab<T> {
    pub id: String,
    pub payload: T,
}

impl<T> Tab<T> {
    pub fn new(id: impl Into<String>, payload: T) -> Self {
        Self {
            id: id.into(),
            payload,
        }
    }
}

/// Pure-logic tab list (no Dioxus signals).  Used internally by
/// [`TabsStore`] and exposed for unit tests / non-WASM consumers.
#[derive(Clone, Debug)]
pub struct TabsStateInner<T> {
    pub tabs: Vec<Tab<T>>,
    pub active: Option<String>,
}

impl<T> Default for TabsStateInner<T> {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active: None,
        }
    }
}

impl<T: Clone + PartialEq> TabsStateInner<T> {
    /// Open a new tab (or focus an existing one with the same id).
    ///
    /// On opening a fresh tab, it becomes the active tab.  Re-opening an
    /// existing id replaces the payload and activates it.
    pub fn open(&mut self, id: impl Into<String>, payload: T) {
        let id = id.into();
        if let Some(existing) = self.tabs.iter_mut().find(|t| t.id == id) {
            existing.payload = payload;
        } else {
            self.tabs.push(Tab::new(id.clone(), payload));
        }
        self.active = Some(id);
    }

    /// Close the tab with the given `id`.
    ///
    /// If the closed tab was active, the next-best neighbour (the tab
    /// that previously sat to its right, falling back to its left) is
    /// activated.  When the last tab closes, `active` becomes `None`.
    pub fn close(&mut self, id: &str) {
        let Some(idx) = self.tabs.iter().position(|t| t.id == id) else {
            return;
        };
        self.tabs.remove(idx);
        if self.active.as_deref() == Some(id) {
            self.active = if self.tabs.is_empty() {
                None
            } else {
                let next_idx = idx.min(self.tabs.len() - 1);
                Some(self.tabs[next_idx].id.clone())
            };
        }
    }

    /// Activate `id` if it exists.  No-op if the id is unknown.
    pub fn activate(&mut self, id: &str) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active = Some(id.to_owned());
        }
    }

    /// Replace the entire tab set, optionally specifying which is active.
    ///
    /// If `active` is `Some` but doesn't match any tab id, the first tab
    /// (if any) is activated instead.
    pub fn set_tabs(&mut self, tabs: Vec<Tab<T>>, active: Option<String>) {
        let resolved_active = match active {
            Some(id) if tabs.iter().any(|t| t.id == id) => Some(id),
            _ => tabs.first().map(|t| t.id.clone()),
        };
        self.tabs = tabs;
        self.active = resolved_active;
    }

    /// The currently active tab, if any.
    pub fn active_tab(&self) -> Option<&Tab<T>> {
        let id = self.active.as_deref()?;
        self.tabs.iter().find(|t| t.id == id)
    }

    /// Set of all tab ids — useful when syncing with URL state or a
    /// tree-view's expanded set.
    pub fn ids(&self) -> HashSet<String> {
        self.tabs.iter().map(|t| t.id.clone()).collect()
    }
}

/// Dioxus-bound tabs store.  Holds [`TabsStateInner`] inside `Signal`s so
/// mutations trigger re-renders.
///
/// Construct with [`TabsStore::new`] inside a Dioxus component (typically
/// at the application root) and pass clones into child components.
///
/// All mutating methods take `&mut self` because Dioxus `Signal::write`
/// requires mutable access.
pub struct TabsStore<T: Clone + PartialEq + 'static> {
    pub tabs: Signal<Vec<Tab<T>>>,
    pub active: Signal<Option<String>>,
}

impl<T: Clone + PartialEq + 'static> Clone for TabsStore<T> {
    fn clone(&self) -> Self {
        Self {
            tabs: self.tabs,
            active: self.active,
        }
    }
}

impl<T: Clone + PartialEq + 'static> Copy for TabsStore<T> {}

impl<T: Clone + PartialEq + 'static> TabsStore<T> {
    /// Construct a fresh, empty store.  Must be called inside a
    /// Dioxus scope (e.g. a `#[component]` body or `use_*` callback).
    pub fn new() -> Self {
        Self {
            tabs: Signal::new(Vec::new()),
            active: Signal::new(None),
        }
    }

    /// Snapshot the current state into a [`TabsStateInner`] (cheap clone).
    pub fn snapshot(&self) -> TabsStateInner<T> {
        TabsStateInner {
            tabs: self.tabs.read().clone(),
            active: self.active.read().clone(),
        }
    }

    /// Open or replace the tab with the given `id`.  See
    /// [`TabsStateInner::open`].
    pub fn open(&mut self, id: impl Into<String>, payload: T) {
        let mut inner = self.snapshot();
        inner.open(id, payload);
        self.commit(inner);
    }

    /// Close the tab with the given `id`.
    pub fn close(&mut self, id: &str) {
        let mut inner = self.snapshot();
        inner.close(id);
        self.commit(inner);
    }

    /// Activate the tab with the given `id`.
    pub fn activate(&mut self, id: &str) {
        let mut inner = self.snapshot();
        inner.activate(id);
        self.commit(inner);
    }

    /// Replace the entire tab set.
    pub fn set_tabs(&mut self, tabs: Vec<Tab<T>>, active: Option<String>) {
        let mut inner = self.snapshot();
        inner.set_tabs(tabs, active);
        self.commit(inner);
    }

    /// Read the currently active tab.
    pub fn active_tab(&self) -> Option<Tab<T>> {
        let id = self.active.read().clone()?;
        self.tabs.read().iter().find(|t| t.id == id).cloned()
    }

    fn commit(&mut self, inner: TabsStateInner<T>) {
        self.tabs.set(inner.tabs);
        self.active.set(inner.active);
    }
}

impl<T: Clone + PartialEq + 'static> Default for TabsStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_appends_and_activates() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("a", 1);
        s.open("b", 2);
        assert_eq!(s.tabs.len(), 2);
        assert_eq!(s.active.as_deref(), Some("b"));
        assert_eq!(s.active_tab().unwrap().payload, 2);
    }

    #[test]
    fn open_existing_replaces_payload_and_activates() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("a", 1);
        s.open("b", 2);
        s.open("a", 99);
        assert_eq!(s.tabs.len(), 2);
        assert_eq!(s.active.as_deref(), Some("a"));
        assert_eq!(s.active_tab().unwrap().payload, 99);
    }

    #[test]
    fn close_picks_neighbour_to_right_then_left() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("a", 1);
        s.open("b", 2);
        s.open("c", 3);
        s.activate("b");
        s.close("b");
        assert_eq!(s.active.as_deref(), Some("c"));

        s.close("c");
        assert_eq!(s.active.as_deref(), Some("a"));
    }

    #[test]
    fn close_last_clears_active() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("only", 1);
        s.close("only");
        assert!(s.tabs.is_empty());
        assert_eq!(s.active, None);
    }

    #[test]
    fn close_inactive_keeps_active() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("a", 1);
        s.open("b", 2);
        s.activate("a");
        s.close("b");
        assert_eq!(s.active.as_deref(), Some("a"));
    }

    #[test]
    fn activate_unknown_is_noop() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("a", 1);
        s.activate("nope");
        assert_eq!(s.active.as_deref(), Some("a"));
    }

    #[test]
    fn set_tabs_resolves_invalid_active() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.set_tabs(
            vec![Tab::new("x", 10), Tab::new("y", 20)],
            Some("missing".to_string()),
        );
        assert_eq!(s.active.as_deref(), Some("x"));
    }

    #[test]
    fn set_tabs_empty_clears_active() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("a", 1);
        s.set_tabs(vec![], None);
        assert_eq!(s.active, None);
    }

    #[test]
    fn ids_collects_all() {
        let mut s: TabsStateInner<u32> = TabsStateInner::default();
        s.open("a", 1);
        s.open("b", 2);
        let ids = s.ids();
        assert!(ids.contains("a"));
        assert!(ids.contains("b"));
        assert_eq!(ids.len(), 2);
    }
}
