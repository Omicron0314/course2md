//! Pure reader identities, source links, search and proportional reading anchors.
use std::{
    ops::Range,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceTarget {
    Web(String),
    Local(PathBuf),
}

pub fn source_target(raw: &str, local: bool) -> Option<SourceTarget> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(url) = url::Url::parse(raw) {
        if url.scheme() == "file" {
            return url.to_file_path().ok().map(SourceTarget::Local);
        }
        if !local && matches!(url.scheme(), "https" | "http") && url.host_str().is_some() {
            return Some(SourceTarget::Web(url.into()));
        }
    }
    let path = PathBuf::from(raw);
    path.is_absolute().then_some(SourceTarget::Local(path))
}

/// Only these source platforms promise a seek URL; arbitrary sites and local players do not.
pub fn seek_url(target: &SourceTarget, seconds: f64) -> Option<String> {
    if !seconds.is_finite() || seconds < 0. {
        return None;
    }
    let SourceTarget::Web(raw) = target else {
        return None;
    };
    let mut url = url::Url::parse(raw).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    let supported = ["youtube.com", "youtu.be", "bilibili.com"]
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")));
    if !supported {
        return None;
    }
    let pairs: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key != "t" && key != "start")
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    url.set_query(None);
    url.query_pairs_mut()
        .extend_pairs(pairs)
        .append_pair("t", &(seconds.floor() as u64).to_string());
    url.set_fragment(None);
    Some(url.into())
}

/// Prefer the registered new location even while the old recovery copy still exists.
pub fn relocated_source(path: &Path, locations: &[(PathBuf, Vec<PathBuf>)]) -> PathBuf {
    let mut candidates = Vec::new();
    for (current, previous) in locations {
        for old in previous {
            if let Ok(relative) = path.strip_prefix(old) {
                candidates.push((old.components().count(), current.join(relative)));
            } else if let (Ok(path), Ok(old)) = (path.canonicalize(), old.canonicalize())
                && let Ok(relative) = path.strip_prefix(&old)
            {
                candidates.push((old.components().count(), current.join(relative)));
            }
        }
    }
    candidates
        .into_iter()
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, path)| path)
        .unwrap_or_else(|| path.to_path_buf())
}

pub fn within_fraction(within: f32, item_height: f32) -> f32 {
    if !within.is_finite() || !item_height.is_finite() || item_height <= 0. {
        return 0.;
    }
    (-within / item_height).clamp(0., 0.999)
}

pub fn restore_within(fraction: Option<f32>, within: f32, item_height: f32) -> f32 {
    if let Some(fraction) = fraction.filter(|value| value.is_finite()) {
        -(fraction.clamp(0., 0.999) * item_height.max(0.))
    } else if within.is_finite() {
        within.min(0.).max(-item_height.max(0.))
    } else {
        0.
    }
}

pub fn nearest_time(
    times: impl IntoIterator<Item = (usize, Option<f64>)>,
    target: f64,
) -> Option<(usize, bool)> {
    if !target.is_finite() {
        return None;
    }
    times
        .into_iter()
        .filter_map(|(index, seconds)| {
            seconds
                .filter(|s| s.is_finite())
                .map(|seconds| (index, (seconds - target).abs()))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
        .map(|(index, distance)| (index, distance < 0.05))
}

/// Map case-folded matches back to valid UTF-8 ranges, including expanding lowercase chars.
pub fn text_matches(text: &str, query: &str) -> Vec<Range<usize>> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let mut lower = String::new();
    let mut mapping = Vec::new();
    for (start, character) in text.char_indices() {
        for lower_char in character.to_lowercase() {
            let count = lower_char.len_utf8();
            lower.push(lower_char);
            mapping.extend(std::iter::repeat_n(
                (start, start + character.len_utf8()),
                count,
            ));
        }
    }
    lower
        .match_indices(&query)
        .filter_map(|(start, value)| {
            Some(mapping.get(start)?.0..mapping.get(start + value.len() - 1)?.1)
        })
        .collect()
}

pub fn timestamp_utc(milliseconds: u64) -> String {
    // Gregorian civil date from days since Unix epoch; constant time for untrusted dates.
    let seconds = (milliseconds / 1000).min(253402300799);
    let days = (seconds / 86400) as i64 + 719468;
    let era = days.div_euclid(146097);
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        seconds % 86400 / 3600,
        seconds % 3600 / 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_links_do_not_invent_local_or_unknown_seek_support() {
        assert!(source_target("", false).is_none());
        assert!(source_target("javascript:alert(1)", false).is_none());
        let local = source_target("/tmp/video.mp4", true).unwrap();
        assert!(seek_url(&local, 12.).is_none());
        let unknown = source_target("https://video.example/watch", false).unwrap();
        assert!(seek_url(&unknown, 12.).is_none());
        let web = source_target("https://www.bilibili.com/video/BV123?p=2&t=4#old", false).unwrap();
        let url = seek_url(&web, 90.).unwrap();
        assert!(url.contains("p=2") && url.contains("t=90") && !url.contains("old"));
    }
    #[test]
    fn move_uses_new_root_even_when_old_copy_is_kept() {
        let locations = vec![(PathBuf::from("/new"), vec![PathBuf::from("/old")])];
        assert_eq!(
            relocated_source(Path::new("/old/video.mp4"), &locations),
            PathBuf::from("/new/video.mp4")
        );
        assert_eq!(
            relocated_source(Path::new("/outside/video.mp4"), &locations),
            PathBuf::from("/outside/video.mp4")
        );
    }
    #[test]
    fn anchor_restores_inside_paragraph_after_font_reflow() {
        let fraction = within_fraction(-75., 300.);
        assert_eq!(fraction, 0.25);
        assert_eq!(restore_within(Some(fraction), -75., 600.), -150.);
        assert_eq!(restore_within(Some(f32::NAN), -75., 600.), -75.);
        assert_eq!(within_fraction(f32::NAN, 10.), 0.);
    }
    #[test]
    fn nearest_time_uses_known_data_and_reports_approximate_matches() {
        assert_eq!(
            nearest_time([(0, None), (1, Some(10.)), (2, Some(20.))], 19.),
            Some((2, false))
        );
        assert_eq!(
            nearest_time([(0, None), (1, Some(0.))], 0.),
            Some((1, true))
        );
        assert_eq!(nearest_time([(0, None)], 0.), None);
    }
    #[test]
    fn search_preserves_utf8_and_finds_multiple_real_matches() {
        let value = "中文 AI 与 ai，İstanbul";
        let ranges = text_matches(value, "ai");
        assert_eq!(
            ranges.iter().map(|r| &value[r.clone()]).collect::<Vec<_>>(),
            ["AI", "ai"]
        );
        let range = text_matches(value, "i̇").remove(0);
        assert_eq!(&value[range], "İ");
        assert!(text_matches(value, "").is_empty());
    }
    #[test]
    fn creation_time_has_a_real_calendar_date_and_explicit_timezone() {
        assert_eq!(timestamp_utc(0), "1970-01-01 00:00 UTC");
        assert_eq!(timestamp_utc(1709164800000), "2024-02-29 00:00 UTC");
    }
}
