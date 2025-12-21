//! Visual atoms - Video/graphics editing components
//!
//! Components for layer management, color editing, masking, and media browsing.
//! Primarily used in VJ applications.

mod clip_grid;
mod color_wheel;
mod gradient_editor;
mod layer_stack;
mod mask_editor;
mod media_browser;
mod output_router;
mod preview;
mod timeline;
mod transform_gizmo;

pub use clip_grid::{ClipCell, ClipGrid, ClipState};
pub use color_wheel::{ColorWheel, Hsva, WheelStyle};
pub use gradient_editor::{
    Gradient, GradientDirection, GradientEditor, GradientEvent, GradientStop,
};
pub use layer_stack::{BlendMode, Layer, LayerEvent, LayerStack};
pub use mask_editor::{Mask, MaskEditor, MaskEvent, MaskPoint, MaskShape};
pub use media_browser::{BrowserViewMode, MediaBrowser, MediaBrowserEvent, MediaItem, MediaType};
pub use output_router::{
    OutputRouter, OutputType, RouteConnection, RouteOutput, RouteSource, RouterEvent, SourceType,
};
pub use preview::{AspectRatio, Preview, PreviewEvent, PreviewState};
pub use timeline::{TimeFormat, Timeline, TimelineEvent, TimelineMarker, TimelineRegion};
pub use transform_gizmo::{
    GizmoHandle, Transform2D, TransformEvent, TransformGizmo, TransformMode,
};
