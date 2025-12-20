//! Mixer atoms - Audio mixing and effects components
//!
//! Components for channel mixing, effects processing, and automation.
//! Primarily used in DAW applications.

mod automation_lane;
mod channel_strip;
mod crossfader;
mod effect_rack;
mod envelope_editor;

pub use automation_lane::{AutomationCurve, AutomationEvent, AutomationLane, AutomationPoint};
pub use channel_strip::{ChannelEvent, ChannelStrip};
pub use crossfader::{CrossFader, CrossfaderCurve, CrossfaderOrientation};
pub use effect_rack::{Effect, EffectCategory, EffectParam, EffectRack, RackEvent, RackOrientation};
pub use envelope_editor::{CurveType, EnvelopeEditor, EnvelopeEvent, EnvelopePoint};
