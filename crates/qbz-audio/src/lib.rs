//! QBZ Audio - Audio backend system for bit-perfect playback
//!
//! This crate provides the audio backend abstraction layer:
//! - Backend trait and implementations (PipeWire, ALSA, PulseAudio)
//! - Audio device enumeration and selection
//! - Loudness analysis and normalization
//! - Diagnostic tools
//!
//! # CRITICAL: This code is IMMUTABLE
//!
//! The audio backend system was carefully designed for bit-perfect playback.
//! Do NOT modify the logic in these files without understanding the full
//! architecture. See `qbz-nix-docs/AUDIO_BACKENDS.md` for details.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     qbz-audio (Tier 1)                      │
//! │  Audio backends, device management, loudness analysis       │
//! └─────────────────────────────────────────────────────────────┘
//!                              ↑
//!                      ┌───────┴───────┐
//!                      │  qbz-models   │
//!                      │   (Tier 0)    │
//!                      └───────────────┘
//! ```

pub mod alsa_backend;
pub mod alsa_direct;
pub mod analysis;
pub mod analyzer_tap;
pub mod backend;
pub mod diagnostic;
pub mod dynamic_amplify;
pub mod loudness;
pub mod loudness_analyzer;
pub mod loudness_cache;
pub mod pipewire_backend;
pub mod pulse_backend;
pub mod settings;
pub mod visualizer;

// Re-export commonly used types
pub use alsa_backend::{
    device_supports_sample_rate, get_device_supported_rates, normalize_device_id_to_stable,
    resolve_stable_to_current_hw,
};
pub use alsa_direct::AlsaDirectStream;
pub use analysis::SpectralAnalyzer;
pub use analyzer_tap::{AnalyzerMessage, AnalyzerTap};
pub use backend::{
    AlsaDirectError, AlsaPlugin, AudioBackend, AudioBackendType, AudioDevice, BackendConfig,
    BackendManager, BackendResult, BitPerfectMode,
};
pub use diagnostic::{AudioDiagnostic, BitDepthResult, DiagnosticSource};
pub use dynamic_amplify::DynamicAmplify;
pub use loudness::{calculate_gain_factor, db_to_linear, extract_replaygain, ReplayGainData};
pub use loudness_analyzer::LoudnessAnalyzer;
pub use loudness_cache::LoudnessCache;
pub use settings::AudioSettings;
pub use visualizer::{RingBuffer, TappedSource, VisualizerTap};
