//! Drag & Drop support for TEA pattern
//!
//! Provides type-safe drag and drop with message emission.

use std::sync::Arc;

use crate::ViewCtx;

/// Response from a drag source
pub struct DragSourceResponse<R> {
    /// The return value from the content closure
    pub inner: R,
    /// The egui response
    pub response: egui::Response,
    /// Whether a drag was started this frame
    pub drag_started: bool,
}

impl<R> DragSourceResponse<R> {
    /// Emit a message when drag starts
    pub fn on_drag_start<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, msg: Msg) -> Self {
        if self.drag_started {
            ctx.emit(msg);
        }
        self
    }

    /// Map the inner value
    pub fn map<U>(self, f: impl FnOnce(R) -> U) -> DragSourceResponse<U> {
        DragSourceResponse {
            inner: f(self.inner),
            response: self.response,
            drag_started: self.drag_started,
        }
    }
}

/// Response from a drop zone
pub struct DropZoneResponse<P, R> {
    /// The return value from the content closure
    pub inner: R,
    /// The egui response
    pub response: egui::Response,
    /// The payload if one was dropped
    pub payload: Option<Arc<P>>,
    /// Whether a compatible payload is being dragged over this zone
    pub is_being_dragged_over: bool,
}

impl<P, R> DropZoneResponse<P, R> {
    /// Emit a message when a payload is dropped
    pub fn on_drop<Msg, F>(self, ctx: &mut ViewCtx<'_, Msg>, f: F) -> Self
    where
        F: FnOnce(Arc<P>) -> Msg,
    {
        if let Some(ref payload) = self.payload {
            ctx.emit(f(Arc::clone(payload)));
        }
        self
    }

    /// Emit a message when hovering with a compatible payload
    pub fn on_hover<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, msg: Msg) -> Self {
        if self.is_being_dragged_over {
            ctx.emit(msg);
        }
        self
    }

    /// Map the inner value
    pub fn map<U>(self, f: impl FnOnce(R) -> U) -> DropZoneResponse<P, U> {
        DropZoneResponse {
            inner: f(self.inner),
            response: self.response,
            payload: self.payload,
            is_being_dragged_over: self.is_being_dragged_over,
        }
    }
}
