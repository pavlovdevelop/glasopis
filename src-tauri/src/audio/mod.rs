//! Microphone discovery and recording.

pub mod devices;
pub mod recorder;

pub use devices::{list_input_devices, InputDevice};
pub use recorder::Recorder;
