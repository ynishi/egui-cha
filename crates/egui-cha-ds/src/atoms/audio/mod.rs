//! Audio atoms - Audio visualization and control components
//!
//! Components for audio visualization, transport control, and beat synchronization.
//! Used by both VJ and DAW applications.

mod beat_sync;
mod bpm_display;
mod level_meter;
mod oscilloscope;
mod sample_pad;
mod spectrum;
mod step_seq;
mod transport;
mod waveform;

pub use beat_sync::{BeatDivision, BeatSync, BeatSyncEvent, SyncState};
pub use bpm_display::{BpmDisplay, DisplaySize, DisplayStyle};
pub use level_meter::{LevelMeter, MeterMode, MeterOrientation};
pub use oscilloscope::{Oscilloscope, ScopeMode, TriggerMode};
pub use sample_pad::{PadCell, PadEvent, SamplePad};
pub use spectrum::{Spectrum, SpectrumColorMode};
pub use step_seq::{StepEvent, StepSeq, StepValue};
pub use transport::{BeatIndicator, TransportBar, TransportEvent};
pub use waveform::{Waveform, WaveformStyle};
