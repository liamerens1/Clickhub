/// Collects latency samples (microseconds) and reports percentiles.
#[derive(Debug, Default, Clone)]
pub struct LatencyStats {
    samples: Vec<u64>,
}

/// A snapshot of [`LatencyStats`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencySummary {
    pub count: usize,
    pub p50_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
}

impl LatencyStats {
    pub fn record(&mut self, micros: u64) {
        self.samples.push(micros);
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Nearest-rank percentiles. `None` when there are no samples.
    pub fn summary(&self) -> Option<LatencySummary> {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let max_us = *sorted.last()?;
        Some(LatencySummary {
            count: sorted.len(),
            p50_us: nearest_rank(&sorted, 50),
            p99_us: nearest_rank(&sorted, 99),
            max_us,
        })
    }
}

/// `sorted` must be non-empty and ascending.
fn nearest_rank(sorted: &[u64], percentile: usize) -> u64 {
    let rank = (percentile * sorted.len()).div_ceil(100).max(1);
    sorted[rank.min(sorted.len()) - 1]
}
