/// Fully standalone demo of the lightweight RepetitionContext
/// (no crate dependencies — just the logic the user asked for).

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

        let mut ghost_sum = 0.0f32;
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
        s += ((hgr + mr + npr) / 3.0) * 0.15;
        s.min(1.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepetitionContext {
    pub max_ghost_pull: Option<f32>,
    pub internal_monitor_flawed_count: u32,
    pub repeated_request_tags_without_progress: u32,
    pub high_struggle_without_evidence: bool,
    pub repetition_strength: f32,
}

impl RepetitionContext {
    pub fn from_window(window: &RollingTelemetryWindow) -> Self {
        let mut ctx = Self::default();
        if window.window.is_empty() { return ctx; }

        let mut max_g = 0.0f32;
        let mut flawed = 0u32;
        let mut spikes = 0u32;
        let mut struggle = false;

        for o in &window.window {
            if o.ghost_pull > max_g { max_g = o.ghost_pull; }
            if o.internal_monitor_flawed { flawed += 1; }
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

    pub fn should_escalate(&self, threshold: f32) -> bool {
        self.repetition_strength > threshold
    }
}

fn main() {
    println!("=== Lightweight RepetitionContext Demo ===\n");

    // Simulate the gamma_baseline repetitive failure case
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

    let ctx = RepetitionContext::from_window(&w);

    println!("RepetitionContext built from rolling telemetry window:");
    println!("  max_ghost_pull: {:?}", ctx.max_ghost_pull);
    println!("  internal_monitor_flawed_count: {}", ctx.internal_monitor_flawed_count);
    println!("  repeated_request_tags_without_progress: {}", ctx.repeated_request_tags_without_progress);
    println!("  high_struggle_without_evidence: {}", ctx.high_struggle_without_evidence);
    println!("  repetition_strength: {:.3}", ctx.repetition_strength);

    if ctx.should_escalate(0.55) {
        println!("\n>>> REPETITIVE FAILURE DETECTED VIA SIGNALS");
        println!("    → Treat as old_mistake_seen");
        println!("    → increment repeat_mistake_count");
        println!("    → raise action_level");
    }

    println!("\n(Pure signal correlation — zero surface text matching used.)");
}
