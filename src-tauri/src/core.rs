use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    pub key: String,
    pub source: String,
    pub start: i64,
    pub end: i64,
}

pub fn close_interval(open: Option<Interval>, end: i64) -> Option<Interval> {
    open.filter(|i| end > i.start).map(|mut i| {
        i.end = end;
        i
    })
}

/// Clips intervals to the requested range and merges overlaps per source/key.
/// This is the single accounting path used by app and browser usage summaries.
pub fn aggregate(intervals: &[Interval], from: i64, to: i64) -> HashMap<(String, String), i64> {
    let mut grouped: HashMap<(String, String), Vec<(i64, i64)>> = HashMap::new();
    for item in intervals {
        let start = item.start.max(from);
        let end = item.end.min(to);
        if end > start {
            grouped
                .entry((item.source.clone(), item.key.clone()))
                .or_default()
                .push((start, end));
        }
    }
    let mut out = HashMap::new();
    for (key, mut spans) in grouped {
        spans.sort_by_key(|s| s.0);
        let mut total = 0;
        let mut current: Option<(i64, i64)> = None;
        for span in spans {
            match current {
                Some((start, end)) if span.0 <= end => current = Some((start, end.max(span.1))),
                Some((start, end)) => {
                    total += end - start;
                    current = Some(span);
                }
                None => current = Some(span),
            }
        }
        if let Some((start, end)) = current {
            total += end - start;
        }
        out.insert(key, total);
    }
    out
}

pub fn is_idle(last_input: i64, now: i64, threshold: i64) -> bool {
    now.saturating_sub(last_input) >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregates_overlaps_once() {
        let xs = vec![
            Interval {
                key: "Code".into(),
                source: "app".into(),
                start: 0,
                end: 100,
            },
            Interval {
                key: "Code".into(),
                source: "app".into(),
                start: 50,
                end: 160,
            },
        ];
        assert_eq!(
            aggregate(&xs, 0, 200)[&(String::from("app"), String::from("Code"))],
            160
        );
    }
    #[test]
    fn clips_to_day() {
        let xs = vec![Interval {
            key: "x".into(),
            source: "website".into(),
            start: -20,
            end: 80,
        }];
        assert_eq!(
            aggregate(&xs, 0, 60)[&(String::from("website"), String::from("x"))],
            60
        );
    }
    #[test]
    fn idle_threshold_is_inclusive() {
        assert!(is_idle(10, 70, 60));
        assert!(!is_idle(11, 70, 60));
    }
}
