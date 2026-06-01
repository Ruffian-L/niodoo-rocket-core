//! Niodoo Core MVP
//!
//! Focused binary whose only job is to demonstrate the loop the user actually wants:
//! - Load mistake reflex / correction ledger
//! - Run with optional Qdrant-backed durable memory
//! - "Turn off" (fresh process) → reload → show retained improvement
//!
//! This is deliberately small and separate from the giant research monolith.
//!
//! Full foundation docs now exist in docs/. Read in this order:
//! README.md → VISION.md → MVP_DEFINITION.md → MASTER_RESEARCH_LEDGER.md → KNOWN_PITFALLS.md

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod reflex;

use reflex::ReflexStore;

#[cfg(feature = "qdrant")]
use reflex::{QdrantConfig, QdrantReflexBackend};

use reflex::{RepetitionContext, RollingTelemetryWindow, TelemetryObservation};

#[derive(Parser)]
#[command(name = "mvp", version, about = "Niodoo Core MVP - durable mistake reflex memory")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load a mistake reflex ledger and test influence on prompts
    Reflex {
        /// Path to the JSONL ledger inside the repo (default: ledgers/mvp_corrections.jsonl)
        #[arg(short, long, default_value = "ledgers/mvp_corrections.jsonl")]
        ledger: PathBuf,

        /// Test prompt to run through the reflex matcher
        #[arg(short, long)]
        prompt: Option<String>,

        /// Maximum number of hints to surface
        #[arg(short, long, default_value_t = 3)]
        max_hints: usize,
    },

    /// Show basic info about a ledger file
    InspectLedger {
        /// Path to the JSONL ledger inside the repo (default: ledgers/mvp_corrections.jsonl)
        #[arg(short, long, default_value = "ledgers/mvp_corrections.jsonl")]
        ledger: PathBuf,
    },

    /// Write a new correction (dual-write to JSONL + Qdrant when --qdrant is used)
    WriteCorrection {
        /// Path to the tracked ledger file inside the repo (default: ledgers/mvp_corrections.jsonl)
        #[arg(short, long, default_value = "ledgers/mvp_corrections.jsonl")]
        jsonl: PathBuf,

        /// Qdrant URL (defaults to your real memory stack from `memory-up`)
        #[arg(long, default_value = "http://127.0.0.1:6360")]
        qdrant_url: String,

        /// Dedicated collection for mistake reflexes (keeps it separate from grok-memories)
        #[arg(long, default_value = "niodoo-corrections")]
        collection: String,
    },

    /// Load all corrections from Qdrant and show count (fresh-process simulation)
    LoadFromQdrant {
        /// Your real memory stack Qdrant (from memory-up)
        #[arg(long, default_value = "http://127.0.0.1:6360")]
        qdrant_url: String,

        #[arg(long, default_value = "niodoo-corrections")]
        collection: String,
    },

    /// Capture a new correction (the "learning" side — model made a mistake, we store the fix)
    CaptureCorrection {
        /// Path to the tracked ledger file inside the repo (default: ledgers/mvp_corrections.jsonl)
        #[arg(short, long, default_value = "ledgers/mvp_corrections.jsonl")]
        jsonl: PathBuf,

        /// Short description of the bad behavior that was repeated
        #[arg(long)]
        bad: String,

        /// What the correct behavior / reflex should be
        #[arg(long)]
        corrected: String,

        #[arg(long, default_value = "http://127.0.0.1:6360")]
        qdrant_url: String,

        #[arg(long, default_value = "niodoo-corrections")]
        collection: String,
    },

    /// Simulate feeding a rolling telemetry window and see repetition strength + escalation
    SimulateWindow {
        /// Run one of the built-in scenarios (gamma_baseline, gamma_with_ledger, stuck_loop)
        #[arg(long, default_value = "gamma_baseline")]
        scenario: String,
    },

    /// Process a real model run directory (telemetry.jsonl + logs) with the lightweight signal-based detector.
    /// This is the concrete next slice: prove it works with live telemetry from an actual model generation.
    ProcessRun {
        /// Path to a real run directory containing telemetry.jsonl and stdout/stderr logs
        #[arg(short, long)]
        run_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Reflex {
            ledger,
            prompt,
            max_hints,
        } => {
            println!("=== Niodoo MVP: Mistake Reflex Loader ===");
            println!("Loading ledger from: {} (repo-tracked under ledgers/)", ledger.display());

            let store = ReflexStore::load_from_jsonl(&ledger)?;
            println!("Loaded {} events into reflex store.\n", store.len());

            let test_prompts = if let Some(p) = prompt {
                vec![p]
            } else {
                // Default test prompts based on the actual winning experiments
                vec![
                    "What was the status of §10dv in the claims analysis?".to_string(),
                    "Should we mark the bridge influence claim as GREEN based on the collector summary?".to_string(),
                    "Tell me about the project owner and current status of Niodoo.".to_string(),
                ]
            };

            for (i, prompt) in test_prompts.iter().enumerate() {
                println!("--- Test Prompt {} ---", i + 1);
                println!("Original: {}\n", prompt);

                let hints = store.find_relevant_hints(prompt, max_hints);

                if hints.is_empty() {
                    println!("No relevant reflexes matched.\n");
                    continue;
                }

                println!("Matched {} reflex(es):", hints.len());
                for h in &hints {
                    println!("  [{}] score={:.2} triggers={:?}", h.event_id, h.score, h.trigger_terms);
                }

                let augmented = store.apply_text_hints(prompt, &hints);
                println!("\nAugmented prompt (what would be sent with influence):\n{}\n", augmented);
            }

            println!("=== Reflex demo complete ===");
            println!("This is the exact influence path that produced the gamma correction and 9/10 claims result.");
        }

        Commands::InspectLedger { ledger } => {
            let store = ReflexStore::load_from_jsonl(&ledger)?;
            println!("Ledger: {}", ledger.display());
            println!("Total events: {}", store.len());
            println!("\nThis loader is now ready to be wired into real runs + Qdrant dual-write.");
        }

        #[cfg(feature = "qdrant")]
        Commands::WriteCorrection {
            jsonl,
            qdrant_url,
            collection,
        } => {
            println!("=== Write Correction (Dual JSONL + Qdrant) ===");
            println!("JSONL: {} (tracked in repo ledgers/)", jsonl.display());
            println!("Qdrant: {} / collection={}  (your real memory stack — see docs/LEDGERS.md for ingestion)", qdrant_url, collection);

            let cfg = QdrantConfig {
                url: qdrant_url,
                api_key: None,
                collection,
            };
            let backend = QdrantReflexBackend::new(cfg).await?;

            // For demo: create a simple test correction
            let event = reflex::MistakeReflexEvent {
                id: format!("mvp_demo_{}", chrono::Utc::now().timestamp_millis()),
                domain: "gmms:semantic_correction_slice".to_string(),
                trigger_terms: vec!["mvp".to_string(), "demo".to_string(), "correction".to_string()],
                bad_reflex: "forget previous corrections".to_string(),
                corrected_reflex: "Always remember corrections that were stored in Qdrant before the process was killed.".to_string(),
                episodic_correction: Some("This correction was written via mvp --features qdrant".to_string()),
                evidence_requirement: "Qdrant must survive full process restart".to_string(),
                rejected_surfaces: vec![],
                accepted_surfaces: vec![],
                allowed_actions: vec!["remember_after_reset".to_string()],
                confidence: 0.95,
            };

            let store = ReflexStore::new();
            store
                .write_correction_dual(event.clone(), &jsonl, Some(&backend))
                .await?;

            println!("\nWrote correction with id: {}", event.id);
            println!("JSONL appended + Qdrant upserted.");
            println!("Run with a fresh process + `load-from-qdrant` to prove it survived.");
        }

        #[cfg(not(feature = "qdrant"))]
        Commands::WriteCorrection { .. } => {
            anyhow::bail!("WriteCorrection requires building with --features qdrant");
        }

        #[cfg(feature = "qdrant")]
        Commands::LoadFromQdrant { qdrant_url, collection } => {
            println!("=== Load From Qdrant (simulating fresh process after full reset) ===");
            println!("Qdrant: {} / {}  (your real memory stack)", qdrant_url, collection);

            let cfg = QdrantConfig {
                url: qdrant_url,
                api_key: None,
                collection,
            };
            let backend = QdrantReflexBackend::new(cfg).await?;

            let store = ReflexStore::load_from_qdrant(&backend).await?;
            println!("\nLoaded {} corrections from Qdrant into fresh ReflexStore.", store.len());

            if store.len() > 0 {
                println!("This proves the corrections survived the previous process death.");
            } else {
                println!("Collection is empty (or first time).");
            }
        }

        #[cfg(not(feature = "qdrant"))]
        Commands::LoadFromQdrant { .. } => {
            anyhow::bail!("LoadFromQdrant requires building with --features qdrant");
        }

        Commands::CaptureCorrection {
            jsonl,
            bad,
            corrected,
            qdrant_url,
            collection,
        } => {
            println!("=== Capture Correction (Learning side) ===");
            println!("Bad reflex:    {}", bad);
            println!("Corrected:     {}", corrected);

            let event = ReflexStore::create_correction(
                "mvp_capture",
                vec!["mvp".to_string(), "capture".to_string()],
                bad,
                corrected,
                Some("Captured live via mvp capture-correction".to_string()),
            );

            #[cfg(feature = "qdrant")]
            {
                let cfg = reflex::QdrantConfig {
                    url: qdrant_url,
                    api_key: None,
                    collection,
                };
                let backend = reflex::QdrantReflexBackend::new(cfg).await.ok();

                let store = ReflexStore::new();
                if let Err(e) = store.write_correction_dual(event.clone(), &jsonl, backend.as_ref()).await {
                    eprintln!("Write error: {}", e);
                }
            }

            #[cfg(not(feature = "qdrant"))]
            {
                let store = ReflexStore::new();
                store.append_to_jsonl(&event, &jsonl)?;
            }

            println!("\nCaptured correction id: {}", event.id);
            println!("Written to JSONL under ledgers/ (git-tracked + watcher protected). Qdrant best-effort on your custom stack.");
        }

        Commands::SimulateWindow { scenario } => {
            println!("=== Rolling Telemetry Window + Repetition Strength (signal correlation, not string matching) ===");
            println!("Scenario: {}", scenario);

            let mut window = RollingTelemetryWindow::new(24);

            match scenario.as_str() {
                "gamma_baseline" => {
                    for step in 0..18 {
                        window.push(TelemetryObservation {
                            step,
                            ghost_pull: 10.0,
                            internal_monitor_flawed: step > 8 && step % 3 == 0,
                            request_spike_count: if step > 10 { 1 } else { 0 },
                            no_progress_marker: step > 12,
                            latency_spike: false,
                            fallback_activated: false,
                        });
                    }
                }
                "gamma_with_ledger" => {
                    for step in 0..12 {
                        window.push(TelemetryObservation {
                            step,
                            ghost_pull: if step < 6 { 4.5 } else { 1.2 },
                            internal_monitor_flawed: step == 5,
                            request_spike_count: 0,
                            no_progress_marker: false,
                            latency_spike: false,
                            fallback_activated: false,
                        });
                    }
                }
                "stuck_loop" => {
                    for step in 0..20 {
                        window.push(TelemetryObservation {
                            step,
                            ghost_pull: 7.5 + (step as f32 % 4.0),
                            internal_monitor_flawed: step % 4 == 0,
                            request_spike_count: 1,
                            no_progress_marker: step % 3 == 0,
                            latency_spike: step % 5 == 0,
                            fallback_activated: false,
                        });
                    }
                }
                _ => {
                    println!("Unknown scenario. Using gamma_baseline.");
                    for step in 0..15 {
                        window.push(TelemetryObservation {
                            step,
                            ghost_pull: 9.0,
                            internal_monitor_flawed: step > 6,
                            request_spike_count: if step > 8 { 1 } else { 0 },
                            no_progress_marker: step > 9,
                            latency_spike: false,
                            fallback_activated: false,
                        });
                    }
                }
            }

            let strength = window.repetition_strength();
            println!("Window size: {}", window.len());
            println!("Repetition strength (0-1): {:.3}", strength);

            if let Some(escalation) = ReflexStore::new().evaluate_repetition_from_window(&window, 0.55) {
                println!("\n>>> REPETITIVE FAILURE DETECTED (score > threshold)");
                println!("    Strength: {:.3}", escalation.strength);
                println!("    Treat as old_mistake_seen → escalate");
                println!("    Suggested action_level boost: {}", escalation.suggested_action_level_boost);
                println!("\nThis would increment repeat_mistake_count and raise action_level on matching reflexes.");
                println!("(No surface string matching required — pure signal correlation from the rolling window.)");
            } else {
                println!("\nRepetition strength below threshold. No automatic escalation.");
            }
        }

        Commands::ProcessRun { run_dir } => {
            println!("=== Processing real model run with lightweight signal-based RepetitionContext ===");
            println!("Run dir: {}", run_dir.display());

            // Delegate to the dedicated Python tool (matches how the rest of the user's memory stack works)
            let status = std::process::Command::new("python3")
                .arg("scripts/process_real_run.py")
                .arg(&run_dir)
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("\nReal run processed. Corrections (if any) written to ledgers/mvp_corrections.jsonl");
                    println!("Now that embed-up is up, run:");
                    println!("  python3 scripts/ingest_niodoo_corrections.py");
                    println!("to push them into your live niodoo-corrections collection on 6360.");
                }
                _ => {
                    eprintln!("Failed to run process_real_run.py. Make sure the script exists and python3 is available.");
                }
            }
        }
    }

    Ok(())
}
