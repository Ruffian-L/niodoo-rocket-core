//! Minimal standalone demonstration of the new repetition detection approach.
//!
//! Rolling telemetry window + signal correlation scoring (no string/surface matching).
//! If strength > threshold → treat as old_mistake_seen and escalate.

use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct TelemetryObservation {
    pub step: usize,
    pub ghost_pull: f32,
    pub internal_monitor_flawed: bool,
    pub request_spike_count: u32,
    pub no_progress_marker: bool,
    pub latency_spike: bool,
    pub fallback_activated: bool,
}

#[derive(Debug, Clone)]
pub struct RollingTelemetryWindow {
    window: VecDeque<TelemetryObservation>,
    capacity: usize,
}

impl RollingTelemetryWindow {
    pub fn new(capacity: usize) -> Self {
        Self { window: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, obs: TelemetryObservation) {
        if self.window.len() == self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(obs);
    }

    pub fn repetition_strength(&self) -> f32 {
        if self.window.is_empty() { return 0.0; }
        let n = self.window.len() as f32;

        let mut ghost_sum = 0.0;
        let mut high_ghost = 0u32;
        let mut monitors = 0u32;
        let mut spikes = 0u32;
        let mut no_prog = 0u32;

        for o in &self.window {
            ghost_sum += o.ghost_pull;
            if o.ghost_pull > 4.0 { high_ghost += 1; }
            if o.internal_monitor_flawed { monitors += 1; }
            spikes += o.request_spike_count;
            if o.no_progress_marker { no_prog += 1; }
        }

        let avg_g = (ghost_sum / n).min(12.0) / 12.0;
        let hgr = high_ghost as f32 / n;
        let mr = monitors as f32 / n;
        let si = (spikes as f32 / n).min(4.0) / 4.0;
        let npr = no_prog as f32 / n;

        let mut s = avg_g * 0.25 + hgr * 0.30 + mr * 0.20 + si * 0.15 + npr * 0.10;
        s += ((hgr + mr + npr) / 3.0) * 0.15; // co-occurrence bonus
        s.min(1.0)
    }
}

#[derive(Debug)]
pub struct RepetitionEscalation {
    pub strength: f32,
    pub treat_as_old_mistake: bool,
    pub suggested_action_level_boost: u8,
}

pub fn evaluate_repetition(
    window: &RollingTelemetryWindow,
    threshold: f32,
) -> Option<RepetitionEscalation> {
    let s = window.repetition_strength();
    if s > threshold {
        Some(RepetitionEscalation {
            strength: s,
            treat_as_old_mistake: true,
            suggested_action_level_boost: if s > 0.75 { 2 } else { 1 },
        })
    } else {
        None
    }
}

fn main() {
    // Simulate the classic gamma_baseline case (sustained high ghost pull + monitors)
    let mut w = RollingTelemetryWindow::new(24);
    for step in 0..18 {
        w.push(TelemetryObservation {
            step,
            ghost_pull: 10.0,
            internal_monitor_flawed: step > 8 && step % 3 == 0,
            request_spike_count: if step > 10 { 1 } else { 0 },
            no_progress_marker: step > 12,
            latency_spike: false,
            fallback_activated: false,
        });
    }

    println!("Gamma baseline simulation");
    println!("Repetition strength: {:.3}", w.repetition_strength());

    if let Some(e) = evaluate_repetition(&w, 0.55) {
        println!(">>> REPETITIVE FAILURE — escalate");
        println!("    strength: {:.3}", e.strength);
        println!("    boost action_level by: {}", e.suggested_action_level_boost);
        println!("    (would increment repeat_mistake_count)");
    }
}