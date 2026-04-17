/// ResizeHandle — horizontal drag-to-resize component.
///
/// Renders a narrow grab zone that calls `on_resize(delta_px)` on every
/// rAF-batched pointer movement.  During drag:
///   - `cursor: col-resize` is applied to `document.body`
///   - `user-select: none` prevents text selection
///
/// Listeners are attached to `document` (not the element) so dragging
/// outside the element still works, and removed on component cleanup.
///
/// ## Memory safety
///
/// The mousemove closure lifetime is managed with a `StoredValue<_, LocalStorage>`
/// so the live JsValue is cleaned up via `on_cleanup` when the component unmounts,
/// even if a drag is in progress.  The rAF closure is the only part that uses
/// `forget()` — one per rAF cycle which is bounded and short-lived.
use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Array, Function, Object, Reflect};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

// ── DOM helpers ───────────────────────────────────────────────────────────────

fn doc_js() -> Option<JsValue> {
    web_sys::window()
        .and_then(|w| w.document())
        .map(|d| JsValue::from(d))
}

fn add_doc_listener(event: &str, cb: &JsValue) {
    let Some(doc) = doc_js() else { return };
    let f = Reflect::get(&doc, &JsValue::from_str("addEventListener")).unwrap_or_default();
    Reflect::apply(
        f.unchecked_ref::<Function>(),
        &doc,
        &Array::of2(&JsValue::from_str(event), cb),
    )
    .ok();
}

fn remove_doc_listener(event: &str, cb: &JsValue) {
    let Some(doc) = doc_js() else { return };
    let f = Reflect::get(&doc, &JsValue::from_str("removeEventListener")).unwrap_or_default();
    Reflect::apply(
        f.unchecked_ref::<Function>(),
        &doc,
        &Array::of2(&JsValue::from_str(event), cb),
    )
    .ok();
}

/// Add a listener with `{ once: true }` so it fires at most once.
fn add_doc_listener_once(event: &str, cb: &JsValue) {
    let Some(doc) = doc_js() else { return };
    let f = Reflect::get(&doc, &JsValue::from_str("addEventListener")).unwrap_or_default();
    let opts = Object::new();
    Reflect::set(&opts, &JsValue::from_str("once"), &JsValue::from_bool(true)).ok();
    let args = Array::of3(&JsValue::from_str(event), cb, opts.as_ref());
    Reflect::apply(f.unchecked_ref::<Function>(), &doc, &args).ok();
}

fn set_body_cursor(cursor: &str) {
    let Some(win) = web_sys::window() else { return };
    let Some(doc) = win.document() else { return };
    let Some(body) = doc.body() else { return };
    let style = body.style();
    let _ = style.set_property("cursor", cursor);
    let select = if cursor.is_empty() { "" } else { "none" };
    let _ = style.set_property("user-select", select);
    let _ = style.set_property("-webkit-user-select", select);
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Horizontal drag-to-resize handle.
///
/// Place this as the **last child** of the element you want to make
/// resizable (or at the edge where you want the drag zone).
/// `on_resize(delta)` is called with CSS pixel deltas; accumulate them
/// into a width signal in the parent.
#[component]
pub fn ResizeHandle(on_resize: impl Fn(f64) + 'static) -> impl IntoView {
    // Wrap callback in Rc so we can share it across closures.
    let on_resize_rc: Rc<dyn Fn(f64)> = Rc::new(on_resize);

    // `StoredValue<_, LocalStorage>` keeps the live mousemove JsValue so
    // `on_cleanup` can remove it from the document even if a drag is in progress.
    let live_mm: StoredValue<Option<JsValue>, LocalStorage> = StoredValue::new_local(None);

    // Cleanup: remove any still-active mousemove listener when the component unmounts.
    on_cleanup(move || {
        if let Some(js) = live_mm.get_value() {
            remove_doc_listener("mousemove", &js);
            set_body_cursor("");
        }
    });

    let on_mousedown = move |e: MouseEvent| {
        e.prevent_default();

        let last_x = Rc::new(RefCell::new(e.client_x() as f64));
        let pending = Rc::new(RefCell::new(0.0_f64));
        let raf_active = Rc::new(RefCell::new(false));
        let on_resize_clone = on_resize_rc.clone();

        set_body_cursor("col-resize");

        // ── mousemove ────────────────────────────────────────────────────────

        let last_x_mm = last_x.clone();
        let pending_mm = pending.clone();
        let raf_active_mm = raf_active.clone();
        let or_mm = on_resize_clone.clone();

        let mm = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            let dx = e.client_x() as f64 - *last_x_mm.borrow();
            *last_x_mm.borrow_mut() = e.client_x() as f64;
            *pending_mm.borrow_mut() += dx;

            if !*raf_active_mm.borrow() {
                *raf_active_mm.borrow_mut() = true;
                let pd = pending_mm.clone();
                let ra = raf_active_mm.clone();
                let or = or_mm.clone();
                // The rAF closure is short-lived (fires once per frame);
                // `forget()` here is bounded — one allocation per queued frame.
                let cb = Closure::<dyn FnMut(f64)>::new(move |_: f64| {
                    *ra.borrow_mut() = false;
                    let delta = *pd.borrow();
                    *pd.borrow_mut() = 0.0;
                    if delta != 0.0 {
                        or(delta);
                    }
                });
                web_sys::window()
                    .unwrap()
                    .request_animation_frame(cb.as_ref().unchecked_ref())
                    .ok();
                cb.forget();
            }
        });

        let mm_js = mm.into_js_value(); // transferred ownership — no forget() needed
        live_mm.set_value(Some(mm_js.clone()));

        // ── mouseup (once) ───────────────────────────────────────────────────

        let mm_js_mu = mm_js.clone();
        let pending_mu = pending.clone();
        let or_mu = on_resize_clone.clone();

        let mu = Closure::<dyn FnMut(MouseEvent)>::new(move |_: MouseEvent| {
            let delta = *pending_mu.borrow();
            if delta != 0.0 {
                or_mu(delta);
            }
            set_body_cursor("");
            remove_doc_listener("mousemove", &mm_js_mu);
            // Clear the stored reference so on_cleanup knows drag ended.
            live_mm.set_value(None);
        });
        let mu_js = mu.into_js_value();

        add_doc_listener("mousemove", &mm_js);
        add_doc_listener_once("mouseup", &mu_js);
        // mu_js is dropped here — because add_doc_listener_once uses {once:true},
        // the browser holds the only reference and releases it after one call.
    };

    view! {
        <div class="va-resize-handle" on:mousedown=on_mousedown />
    }
}

