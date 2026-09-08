//! Opt-in native measurements. No collection task or file I/O in ordinary builds.
use gpui::{App, Window};
use serde_json::json;
use std::{io::Write, path::PathBuf, time::Duration};

/// Record GPUI's measured draw and input-to-present histograms once per second.
/// File creation, encoding and writes run on the background executor. This does
/// not request frames, navigate the app, or change the animation scheduler.
pub fn start(window: &mut Window, cx: &App) {
    let Some(path) = std::env::var_os("COURSE2MD_PERFORMANCE_LOG").map(PathBuf::from) else {
        return;
    };
    let executor = cx.background_executor().clone();
    window
        .spawn(cx, async move |cx| {
            let started = std::time::Instant::now();
            loop {
                smol::Timer::after(Duration::from_secs(1)).await;
                let Ok((frames, inputs)) = cx.update(|window, _| {
                    (
                        window.frame_duration_snapshot(),
                        window.input_latency_snapshot(),
                    )
                }) else {
                    break;
                };
                let elapsed_ms = started.elapsed().as_millis();
                let path = path.clone();
                let result = executor
                    .spawn(async move {
                        macro_rules! histogram {
                        ($hist:expr) => {{
                            let h = &$hist;
                            json!({
                                "count": h.len(),
                                "p50_ms": h.value_at_quantile(0.50) as f64 / 1e6,
                                "p95_ms": h.value_at_quantile(0.95) as f64 / 1e6,
                                "p99_ms": h.value_at_quantile(0.99) as f64 / 1e6,
                                "max_ms": h.max() as f64 / 1e6,
                                "over_16_7_ms": h.count_between(16_700_000, u64::MAX),
                                "over_100_ms": h.count_between(100_000_000, u64::MAX),
                            })
                        }};
                    }
                        let sample = json!({
                            "elapsed_ms": elapsed_ms,
                            "draw": histogram!(frames.draw_duration_histogram),
                            "dirty_to_present": histogram!(frames.dirty_to_present_histogram),
                            "animation_interval": histogram!(frames.present_interval_histogram),
                            "input_to_present": histogram!(inputs.latency_histogram),
                            "mid_draw_events_dropped": inputs.mid_draw_events_dropped,
                        });
                        let mut file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(path)?;
                        serde_json::to_writer(&mut file, &sample)?;
                        file.write_all(b"\n")
                    })
                    .await;
                if let Err(error) = result {
                    eprintln!("Performance recording stopped: {error}");
                    break;
                }
            }
        })
        .detach();
}
