//! Progress samples, independent of rendering. ETA excludes cached work and resets on retries.
use std::time::{Duration, Instant};

pub struct Activity {
    pub current: u64,
    pub total: u64,
    pub message: String,
    pub done: bool,
    pub workers: usize,
    started: Instant,
    baseline: u64,
    updated: Instant,
}
impl Activity {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            current: 0,
            total: 0,
            message: String::new(),
            done: false,
            workers: 1,
            started: now,
            baseline: 0,
            updated: now,
        }
    }
    pub fn update(&mut self, current: u64, total: u64, message: Option<String>) {
        let now = Instant::now();
        if current < self.current || total != self.total {
            self.started = now;
            self.baseline = current;
        }
        if current != self.current {
            self.updated = now;
        }
        self.current = current;
        self.total = total;
        if let Some(message) = message {
            self.message = message;
        }
    }
    pub fn fraction(&self) -> Option<f32> {
        (self.total > 0).then(|| (self.current as f32 / self.total as f32).clamp(0., 1.))
    }
    fn rate(&self) -> Option<f64> {
        let elapsed = self.started.elapsed().as_secs_f64();
        let processed = self.current.saturating_sub(self.baseline);
        (elapsed >= 1. && processed > 0 && self.updated.elapsed() < Duration::from_secs(30))
            .then(|| processed as f64 / elapsed)
    }
    pub fn detail(&self, stage: &str, running: bool) -> String {
        if self.done {
            return "已完成".into();
        }
        if !running {
            return "已停止".into();
        }
        let quantity = quantity(stage, self.current, self.total);
        let remaining = if self.updated.elapsed() >= Duration::from_secs(30) && self.current > 0 {
            if stage.starts_with("scenes/") {
                "仍在处理"
            } else {
                "等待响应"
            }
            .into()
        } else if self.total > self.current {
            self.rate()
                .map(|rate| {
                    format!(
                        "约剩 {}",
                        duration((self.total - self.current) as f64 / rate)
                    )
                })
                .unwrap_or_default()
        } else if self.total > 0 {
            "收尾中…".into()
        } else {
            format!("已用 {}", duration(self.started.elapsed().as_secs_f64()))
        };
        let speed = if stage.starts_with("model/") && stage != "model/apple" {
            self.rate()
                .map(|rate| format!(" · {}/s", bytes(rate as u64)))
                .unwrap_or_default()
        } else {
            String::new()
        };
        if quantity.is_empty() {
            remaining
        } else if remaining.is_empty() {
            format!("{quantity}{speed}")
        } else {
            format!("{quantity}{speed} · {remaining}")
        }
    }
}

pub fn quantity(stage: &str, current: u64, total: u64) -> String {
    if stage == "scenes/scan" {
        format!("已检查 {current} 张画面")
    } else if stage == "scenes/extract" && total > 0 {
        format!("已保存 {current} / {total} 张截图")
    } else if stage.starts_with("model/") && stage != "model/apple" {
        if total > 0 {
            format!("{} / {}", bytes(current), bytes(total))
        } else {
            bytes(current)
        }
    } else if total > 0 {
        if stage == "transcribe" {
            format!("{current} / {total} 段")
        } else {
            format!(
                "{:.0}%",
                (current as f64 / total as f64).clamp(0., 1.) * 100.
            )
        }
    } else {
        String::new()
    }
}

/// Older saved outcomes combined a stage label, retention message and English
/// diagnostics. Keep these records readable without rewriting their evidence.
pub fn component_failure_message(label: &str, message: Option<&str>) -> String {
    let chinese = message
        .unwrap_or_default()
        .split(" / ")
        .next()
        .unwrap_or_default()
        .trim();
    if chinese.is_empty()
        || matches!(
            chinese,
            "摘要尚未完成，已保留正文"
                | "校对未全部完成，原文已保留"
                | "部分校对未完成，原始文字已保留"
        )
    {
        return format!("{label}尚未完成，正文已保存。可以仅补{label}。");
    }
    let prefix = format!("{label}尚未完成");
    if chinese.starts_with(&prefix) {
        chinese.into()
    } else {
        format!("{label}尚未完成：{chinese}")
    }
}

pub fn stage_order(stage: &str) -> usize {
    match stage {
        "fetch" => 0,
        "subtitle" => 1,
        "download" => 2,
        "scenes" | "scenes/scan" => 3,
        "scenes/extract" => 4,
        "audio" => 5,
        s if s.starts_with("model/") => 6,
        "model-load" => 7,
        "transcribe" => 8,
        "llm" => 9,
        "summary" | "summarize" => 10,
        "render" => 11,
        "export" | "exports" => 12,
        _ => 13,
    }
}

pub fn title(stage: &str) -> String {
    if let Some(file) = stage.strip_prefix("model/") {
        return if file == "apple" {
            "准备 Apple 识别模型".into()
        } else if file.starts_with("mmproj") {
            "下载音频编码器".into()
        } else {
            "下载语音模型".into()
        };
    }
    match stage {
        "model-load" => "加载识别模型",
        "fetch" => "读取课程",
        "download" => "下载视频",
        "scenes" => "提取画面",
        "scenes/scan" => "扫描画面",
        "scenes/extract" => "生成截图",
        "audio" => "提取音频",
        "transcribe" => "语音转写",
        "subtitle" => "读取字幕",
        "llm" => "AI 校对",
        "summary" | "summarize" => "生成摘要",
        "export" | "exports" => "导出文件",
        "render" => "生成笔记",
        _ => stage,
    }
    .into()
}
fn bytes(value: u64) -> String {
    if value >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", value as f64 / (1024. * 1024. * 1024.))
    } else {
        format!("{:.1} MB", value as f64 / (1024. * 1024.))
    }
}
fn duration(seconds: f64) -> String {
    let seconds = seconds.ceil() as u64;
    if seconds >= 3600 {
        format!("{} 小时 {} 分", seconds / 3600, seconds % 3600 / 60)
    } else if seconds >= 60 {
        format!("{} 分 {} 秒", seconds / 60, seconds % 60)
    } else {
        format!("{seconds} 秒")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_scan_cannot_supply_the_new_extraction_counter_or_eta() {
        let mut scan = Activity::new();
        scan.update(18, 0, Some("已找到 1 张候选截图".into()));
        assert!(scan.fraction().is_none());
        assert!(
            scan.detail("scenes/scan", true)
                .contains("已检查 18 张画面")
        );
        scan.done = true;
        let mut extract = Activity::new();
        extract.update(0, 1, None);
        extract.started = Instant::now() - Duration::from_secs(25);
        let detail = extract.detail("scenes/extract", true);
        assert!(detail.contains("已保存 0 / 1 张截图"), "{detail}");
        assert!(!detail.contains("100%") && !detail.contains("收尾") && !detail.contains("约剩"));
        assert_eq!(extract.fraction(), Some(0.));
        extract.update(1, 1, None);
        extract.done = true;
        assert_eq!(extract.detail("scenes/extract", true), "已完成");
        let mut persisted = crate::workspace::Stage {
            status: "done".into(),
            current: 18,
            total: 18,
            detail: Some("old sample".into()),
        };
        persisted.begin();
        assert_eq!((persisted.current, persisted.total), (0, 0));
        assert!(persisted.detail.is_none());
    }
    #[test]
    fn saved_component_failures_have_one_label_and_no_english_diagnostics() {
        assert_eq!(
            component_failure_message(
                "摘要",
                Some("摘要尚未完成，已保留正文 / Summary incomplete; body retained")
            ),
            "摘要尚未完成，正文已保存。可以仅补摘要。"
        );
        assert_eq!(
            component_failure_message("摘要", Some("服务拒绝了请求（HTTP 400）。")),
            "摘要尚未完成：服务拒绝了请求（HTTP 400）。"
        );
    }
    #[test]
    fn retry_and_cached_work_do_not_inflate_eta() {
        let mut item = Activity::new();
        item.update(60, 100, None); // Resumed work must not count as throughput.
        assert!(item.rate().is_none());
        item.started = Instant::now() - Duration::from_secs(10);
        item.update(80, 100, None);
        assert!((1.9..2.1).contains(&item.rate().unwrap()));
        item.update(5, 100, None); // A retry restarts the sample window.
        assert!(item.rate().is_none());
        item.started = Instant::now() - Duration::from_secs(60);
        item.updated = Instant::now() - Duration::from_secs(31);
        assert!(item.detail("model/test", true).contains("等待响应"));
    }
    #[test]
    fn unknown_total_never_invents_a_percentage_or_eta() {
        let mut item = Activity::new();
        item.update(8192, 0, None);
        assert!(item.fraction().is_none());
        assert!(!item.detail("model/test", true).contains("约剩"));
        assert_eq!(item.detail("model/test", false), "已停止");
    }
}
