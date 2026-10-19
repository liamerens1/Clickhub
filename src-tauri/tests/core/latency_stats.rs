use clickhub_lib::core::{LatencyStats, LatencySummary};

#[test]
fn empty_stats_have_no_summary() {
    assert_eq!(LatencyStats::default().summary(), None);
}

#[test]
fn reports_nearest_rank_percentiles() {
    let mut stats = LatencyStats::default();
    for micros in 1..=1_000 {
        stats.record(micros);
    }
    assert_eq!(
        stats.summary(),
        Some(LatencySummary {
            count: 1_000,
            p50_us: 500,
            p99_us: 990,
            max_us: 1_000,
        })
    );
}

#[test]
fn single_sample_is_every_percentile() {
    let mut stats = LatencyStats::default();
    stats.record(42);
    let summary = stats.summary().expect("one sample");
    assert_eq!(
        (summary.p50_us, summary.p99_us, summary.max_us),
        (42, 42, 42)
    );
}

#[test]
fn order_of_recording_does_not_matter() {
    let mut stats = LatencyStats::default();
    for micros in [9, 1, 5, 3, 7] {
        stats.record(micros);
    }
    let summary = stats.summary().expect("samples");
    assert_eq!((summary.p50_us, summary.max_us), (5, 9));
}
