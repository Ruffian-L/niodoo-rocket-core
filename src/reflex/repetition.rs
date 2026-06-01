//! Lightweight RepetitionContext for the Niodoo MVP.
//!
//! This is the clean, signal-based replacement for the old brittle surface/text
//! matching that was always the bane of our existence for detecting repetitive
//! failures.
//!
//! Design:
//! - Maintains a rolling telemetry window (last N steps)
//! - Key signals: ghost pulls, latency spikes, fallback rate, "no progress" markers,
//!   internal monitors, repeated request tags
//! - Scores repetition strength purely from **signal correlation**, not string matching
//! - If score > threshold → treat as old_mistake_seen and escalate
//!   (caller should increment repeat_mistake_count + raise action_level)

use std::collections::VecDeque;

/// One observation / sample from the running system's telemetry.
#[derive(Debug, Clone, Default)]
pub struct TelemetryObservation {
    pub step: usize,

    // Core signals from the physics / steering layer
    pub ghost_pull: f32,                    // strength of steering intervention needed

    // Self-diagnostic signals emitted by the model
    pub internal_monitor_flawed: bool,      // saw "LOGICALLY FLAWED" style monitor
    pub request_spike_count: u32,           // how many [REQUEST: SPIKE/EXPLORE] etc. this step

    // Explicit failure / stuck markers
    pub no_progress_marker: bool,           // repeated bad surface, lock veto, etc.
    pub latency_spike: bool,                // sudden jump in generation latency
    pub fallback_activated: bool,           // system fell back to simpler / safer path
}

/// Lightweight rolling window of recent telemetry.
/// This is the heart of the new approach.
#[derive(Debug, Clone)]
pub struct RollingTelemetryWindow {
    window: VecDeque<TelemetryObservation>,
    capacity: usize,
}

impl RollingTelemetryWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, obs: TelemetryObservation) {
        if self.window.len() == self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(obs);
    }

    /// The key function: scores how strongly the recent signals indicate
    /// a repetitive failure, using correlation between the signals rather
    /// than any string matching on generated output.
    pub fn repetition_strength(&self) -> f32 {
        if self.window.is_empty() {
            return 0.0;
        }

        let n = self.window.len() as f32;

        let mut ghost_sum = 0.0f32;
        let mut high_ghost_steps = 0u32;
        let mut monitor_hits = 0u32;
        let mut spike_total = 0u32;
        let mut no_progress_steps = 0u32;
        let mut struggle_steps = 0u32;

        for o in &self.window {
            ghost_sum += o.ghost_pull;
            if o.ghost_pull > 4.0 {
                high_ghost_steps += 1;
            }
            if o.internal_monitor_flawed {
                monitor_hits += 1;
            }
            spike_total += o.request_spike_count;
            if o.no_progress_marker {
                no_progress_steps += 1;
            }
            if o.latency_spike || o.fallback_activated {
                struggle_steps += 1;
            }
        }

        let avg_ghost = (ghost_sum / n).min(12.0) / 12.0;
        let high_ghost_ratio = high_ghost_steps as f32 / n;
        let monitor_ratio = monitor_hits as f32 / n;
        let spike_intensity = (spike_total as f32 / n).min(5.0) / 5.0;
        let no_progress_ratio = no_progress_steps as f32 / n;
        let struggle_ratio = struggle_steps as f32 / n;

        // Weighted correlation score.
        // The power comes from multiple signals firing together.
        let mut score = 0.0;
        score += avg_ghost * 0.20;
        score += high_ghost_ratio * 0.25;
        score += monitor_ratio * 0.20;
        score += spike_intensity * 0.15;
        score += no_progress_ratio * 0.10;
        score += struggle_ratio * 0.10;

        // Strong bonus when ghost + monitor + no-progress co-occur
        let co_occurrence = (high_ghost_ratio + monitor_ratio + no_progress_ratio) / 3.0;
        score += co_occurrence * 0.20;

        score.min(1.0)
    }
}

/// The lightweight RepetitionContext the user asked for.
///
/// This is the main public type. It can be constructed from a rolling window
/// of telemetry and then used to decide whether to escalate a reflex.
#[derive(Debug, Clone, Default)]
pub struct RepetitionContext {
    pub max_ghost_pull: Option<f32>,
    pub internal_monitor_flawed_count: u32,
    pub repeated_request_tags_without_progress: u32,
    pub high_struggle_without_evidence: bool,

    /// The computed repetition strength from the window (0.0–1.0)
    pub repetition_strength: f32,
}

impl RepetitionContext {
    pub fn from_window(window: &RollingTelemetryWindow) -> Self {
        let mut ctx = Self::default();

        if window.window.is_empty() {
            return ctx;
        }

        let mut max_g = 0.0f32;
        let mut flawed = 0u32;
        let mut spikes = 0u32;
        let mut struggle = false;

        for o in &window.window {
            if o.ghost_pull > max_g {
                max_g = o.ghost_pull;
            }
            if o.internal_monitor_flawed {
                flawed += 1;
            }
            spikes += o.request_spike_count;
            if o.no_progress_marker || o.latency_spike || o.fallback_activated {
                struggle = true;
            }
        }

        ctx.max_ghost_pull = Some(max_g);
        ctx.internal_monitor_flawed_count = flawed;
        ctx.repeated_request_tags_without_progress = spikes;
        ctx.high_struggle_without_evidence = struggle;
        ctx.repetition_strength = window.repetition_strength();

        ctx
    }

    /// The decision the whole system cares about.
    /// If this returns true, the caller should treat the current situation
    /// as a repetitive failure (equivalent to old_mistake_seen) and escalate
    /// the relevant reflex (bump repeat_mistake_count, raise action_level).
    pub fn should_escalate(&self, threshold: f32) -> bool {
        self.repetition_strength > threshold
    }
}

/// Convenience result type when you want the full escalation details.
#[derive(Debug, Clone)]
pub struct RepetitionEscalation {
    pub strength: f32,
    pub treat_as_old_mistake: bool,
    pub suggested_action_level_boost: u8,
}

pub fn evaluate_for_escalation(
    window: &RollingTelemetryWindow,
    threshold: f32,
) -> Option<RepetitionEscalation> {
    let strength = window.repetition_strength();
    if strength > threshold {
        Some(RepetitionEscalation {
            strength,
            treat_as_old_mistake: true,
            suggested_action_level_boost: if strength > 0.75 { 2 } else { 1 },
        })
    } else {
        None
    }
}
