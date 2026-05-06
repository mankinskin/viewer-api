//! DOM scanning — collect screen-space rects of UI elements matching
//! [`UI_SELECTORS`] and pack them into a flat `Vec<f32>` ready for upload
//! to the element storage buffer.
//!
//! Layout per element: `[x, y, w, h, hue, kind, depth=0, _pad=0]`.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use web_sys::{Document, Element, NodeList};

use super::element_types::*;
use super::webgpu::{get_fn, prop_f32};

/// Scan `#ui-root` (and the rest of the document) for elements matching
/// [`UI_SELECTORS`] and return `(packed_data, count)`.
pub(super) fn scan_ui_rects(doc: &Document) -> (Vec<f32>, usize) {
    let total = UI_SELECTORS.len() as f32;
    let mut data: Vec<f32> = Vec::with_capacity(64 * ELEM_FLOATS);

    for (idx, &(selector, kind)) in UI_SELECTORS.iter().enumerate() {
        let hue: f32 = idx as f32 / total;

        let node_list: NodeList = match doc.query_selector_all(selector) {
            Ok(nl) => nl,
            Err(_) => continue,
        };

        for j in 0..node_list.length() {
            let Some(node) = node_list.get(j) else { continue; };
            let Ok(el) = node.dyn_into::<Element>() else { continue; };

            // Reflect into el.getBoundingClientRect() — avoids the DomRect
            // web-sys feature.
            let Some(rect_val) = get_fn(&el, "getBoundingClientRect")
                .and_then(|f| f.call0(&el).ok())
            else { continue; };

            let x = prop_f32(&rect_val, "x");
            let y = prop_f32(&rect_val, "y");
            let w = prop_f32(&rect_val, "width");
            let h = prop_f32(&rect_val, "height");

            // Skip zero-size elements.
            if w < 1.0 || h < 1.0 { continue; }

            data.push(x);
            data.push(y);
            data.push(w);
            data.push(h);
            data.push(hue);
            data.push(kind as f32);
            data.push(0.0); // depth — flat screen-space
            data.push(0.0); // _pad
        }
    }

    let count = data.len() / ELEM_FLOATS;
    (data, count)
}
