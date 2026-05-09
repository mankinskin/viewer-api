use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

pub(super) type DragState = Rc<RefCell<Option<(JsValue, JsValue)>>>;

pub(super) fn new_drag_state() -> DragState {
    Rc::new(RefCell::new(None))
}

pub(super) fn cleanup_drag_states(mouse_state: &DragState, touch_state: &DragState) {
    let document = current_document();
    cleanup_state(mouse_state, document.as_ref(), "mousemove", "mouseup");
    cleanup_state(touch_state, document.as_ref(), "touchmove", "touchend");

    if let Some(document) = document.as_ref() {
        clear_body_cursor(document);
    }
}

pub(super) fn start_mouse_drag(
    mouse_state: DragState,
    on_resize: EventHandler<f64>,
    is_horizontal: bool,
    evt: Event<MouseData>,
) {
    evt.prevent_default();
    let Some(document) = current_document() else {
        return;
    };

    set_body_cursor(&document, is_horizontal);

    let coords = evt.client_coordinates();
    let initial = if is_horizontal { coords.x } else { coords.y };
    let start_pos = Rc::new(RefCell::new(initial));
    let raf_pending = Rc::new(RefCell::new(false));

    let (move_handle, end_handle) = build_mouse_handlers(
        document.clone(),
        mouse_state.clone(),
        on_resize,
        is_horizontal,
        start_pos,
        raf_pending,
    );

    add_listener_pair(&document, "mousemove", &move_handle, "mouseup", &end_handle);
    *mouse_state.borrow_mut() = Some((move_handle, end_handle));
}

pub(super) fn start_touch_drag(
    touch_state: DragState,
    on_resize: EventHandler<f64>,
    is_horizontal: bool,
    evt: Event<TouchData>,
) {
    use dioxus::web::WebEventExt as _;

    evt.prevent_default();
    let Some(document) = current_document() else {
        return;
    };

    let initial = evt
        .data()
        .try_as_web_event()
        .and_then(|event: web_sys::TouchEvent| touch_position_from_event(&event, is_horizontal))
        .unwrap_or(0.0);

    let start_pos = Rc::new(RefCell::new(initial));
    let raf_pending = Rc::new(RefCell::new(false));

    let (move_handle, end_handle) = build_touch_handlers(
        document.clone(),
        touch_state.clone(),
        on_resize,
        is_horizontal,
        start_pos,
        raf_pending,
    );

    add_listener_pair(&document, "touchmove", &move_handle, "touchend", &end_handle);
    *touch_state.borrow_mut() = Some((move_handle, end_handle));
}

fn current_document() -> Option<web_sys::Document> {
    web_sys::window().and_then(|window| window.document())
}

fn cleanup_state(
    drag_state: &DragState,
    document: Option<&web_sys::Document>,
    move_event: &str,
    end_event: &str,
) {
    let Some((move_handle, end_handle)) = drag_state.borrow_mut().take() else {
        return;
    };

    if let Some(document) = document {
        remove_listener_pair(document, move_event, &move_handle, end_event, &end_handle);
    }

    drop((move_handle, end_handle));
}

fn build_mouse_handlers(
    document: web_sys::Document,
    mouse_state: DragState,
    on_resize: EventHandler<f64>,
    is_horizontal: bool,
    start_pos: Rc<RefCell<f64>>,
    raf_pending: Rc<RefCell<bool>>,
) -> (JsValue, JsValue) {
    let on_resize_move = on_resize.clone();
    let start_pos_move = start_pos.clone();
    let raf_pending_move = raf_pending.clone();

    let move_callback: Closure<dyn FnMut(web_sys::MouseEvent)> = Closure::new(move |event| {
        queue_resize(
            on_resize_move.clone(),
            start_pos_move.clone(),
            raf_pending_move.clone(),
            axis_position(event.client_x() as f64, event.client_y() as f64, is_horizontal),
        );
    });
    let move_handle = move_callback.into_js_value();

    let end_document = document.clone();
    let end_mouse_state = mouse_state.clone();
    let end_callback: Closure<dyn FnMut(web_sys::MouseEvent)> = Closure::new(move |_| {
        cleanup_state(&end_mouse_state, Some(&end_document), "mousemove", "mouseup");
        clear_body_cursor(&end_document);
    });
    let end_handle = end_callback.into_js_value();

    (move_handle, end_handle)
}

fn build_touch_handlers(
    document: web_sys::Document,
    touch_state: DragState,
    on_resize: EventHandler<f64>,
    is_horizontal: bool,
    start_pos: Rc<RefCell<f64>>,
    raf_pending: Rc<RefCell<bool>>,
) -> (JsValue, JsValue) {
    let on_resize_move = on_resize.clone();
    let start_pos_move = start_pos.clone();
    let raf_pending_move = raf_pending.clone();

    let move_callback: Closure<dyn FnMut(web_sys::TouchEvent)> = Closure::new(move |event| {
        let Some(position) = touch_position_from_event(&event, is_horizontal) else {
            return;
        };

        queue_resize(
            on_resize_move.clone(),
            start_pos_move.clone(),
            raf_pending_move.clone(),
            position,
        );
    });
    let move_handle = move_callback.into_js_value();

    let end_document = document.clone();
    let end_touch_state = touch_state.clone();
    let end_callback: Closure<dyn FnMut(web_sys::TouchEvent)> = Closure::new(move |_| {
        cleanup_state(&end_touch_state, Some(&end_document), "touchmove", "touchend");
    });
    let end_handle = end_callback.into_js_value();

    (move_handle, end_handle)
}

fn queue_resize(
    on_resize: EventHandler<f64>,
    start_pos: Rc<RefCell<f64>>,
    raf_pending: Rc<RefCell<bool>>,
    position: f64,
) {
    if *raf_pending.borrow() {
        return;
    }

    *raf_pending.borrow_mut() = true;
    let delta = position - *start_pos.borrow();
    *start_pos.borrow_mut() = position;

    if let Some(window) = web_sys::window() {
        let pending_flag = raf_pending.clone();
        let frame_callback = Closure::once_into_js(move |_: f64| {
            *pending_flag.borrow_mut() = false;
            on_resize.call(delta);
        });
        let _ = window.request_animation_frame(frame_callback.unchecked_ref::<js_sys::Function>());
    }
}

fn add_listener_pair(
    document: &web_sys::Document,
    move_event: &str,
    move_handle: &JsValue,
    end_event: &str,
    end_handle: &JsValue,
) {
    let _ = document.add_event_listener_with_callback(
        move_event,
        move_handle.unchecked_ref::<js_sys::Function>(),
    );
    let _ = document.add_event_listener_with_callback(
        end_event,
        end_handle.unchecked_ref::<js_sys::Function>(),
    );
}

fn remove_listener_pair(
    document: &web_sys::Document,
    move_event: &str,
    move_handle: &JsValue,
    end_event: &str,
    end_handle: &JsValue,
) {
    let _ = document.remove_event_listener_with_callback(
        move_event,
        move_handle.unchecked_ref::<js_sys::Function>(),
    );
    let _ = document.remove_event_listener_with_callback(
        end_event,
        end_handle.unchecked_ref::<js_sys::Function>(),
    );
}

fn set_body_cursor(document: &web_sys::Document, is_horizontal: bool) {
    let cursor = if is_horizontal { "col-resize" } else { "row-resize" };
    if let Some(body) = document.body() {
        let _ = body.style().set_property("cursor", cursor);
    }
}

fn clear_body_cursor(document: &web_sys::Document) {
    if let Some(body) = document.body() {
        let _ = body.style().remove_property("cursor");
    }
}

fn touch_position_from_event(event: &web_sys::TouchEvent, is_horizontal: bool) -> Option<f64> {
    let touch = event.touches().get(0)?;
    Some(axis_position(
        touch.client_x() as f64,
        touch.client_y() as f64,
        is_horizontal,
    ))
}

fn axis_position(x: f64, y: f64, is_horizontal: bool) -> f64 {
    if is_horizontal { x } else { y }
}