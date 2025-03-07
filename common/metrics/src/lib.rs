#![allow(clippy::needless_doctest_main)]
//! A wrapper around the `prometheus` crate that provides a global metrics registry
//! and functions to add and use the following components (more info at
//! [Prometheus docs](https://prometheus.io/docs/concepts/metric_types/)):
//!
//! - `Histogram`: used with `start_timer(..)` and `stop_timer(..)` to record durations (e.g.,
//!   block processing time).
//! - `IncCounter`: used to represent an ideally ever-growing, never-shrinking integer (e.g.,
//!   number of block processing requests).
//! - `IntGauge`: used to represent an varying integer (e.g., number of attestations per block).
//!
//! ## Important
//!
//! Metrics will fail if two items have the same `name`. All metrics must have a unique `name`.
//! Because we use a global registry there is no namespace per crate, it's one big global space.
//!
//! See the [Prometheus naming best practices](https://prometheus.io/docs/practices/naming/) when
//! choosing metric names.
//!
//! ## Example
//!
//! ```rust
//! use metrics::*;
//! use std::sync::LazyLock;
//!
//! // These metrics are "magically" linked to the global registry defined in `metrics`.
//! pub static RUN_COUNT: LazyLock<Result<IntCounter>> = LazyLock::new(|| try_create_int_counter(
//!     "runs_total",
//!     "Total number of runs"
//! ));
//! pub static CURRENT_VALUE: LazyLock<Result<IntGauge>> = LazyLock::new(|| try_create_int_gauge(
//!     "current_value",
//!     "The current value"
//! ));
//! pub static RUN_TIME: LazyLock<Result<Histogram>> =
//!     LazyLock::new(|| try_create_histogram("run_seconds", "Time taken (measured to high precision)"));
//!
//! fn main() {
//!     for i in 0..100 {
//!         inc_counter(&RUN_COUNT);
//!         let timer = start_timer(&RUN_TIME);
//!
//!         for j in 0..10 {
//!             set_gauge(&CURRENT_VALUE, j);
//!             println!("Howdy partner");
//!         }
//!
//!         stop_timer(timer);
//!     }
//! }
//! ```

use prometheus::{Error, HistogramOpts, Opts};
use std::time::Duration;

use prometheus::core::{Atomic, GenericGauge, GenericGaugeVec};
pub use prometheus::{
    exponential_buckets, linear_buckets,
    proto::{Metric, MetricFamily, MetricType},
    Encoder, Gauge, GaugeVec, Histogram, HistogramTimer, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec, Result, TextEncoder, DEFAULT_BUCKETS,
};

/// Collect all the metrics for reporting.
pub fn gather() -> Vec<prometheus::proto::MetricFamily> {
    prometheus::gather()
}

/// Attempts to create an `IntCounter`, returning `Err` if the registry does not accept the counter
/// (potentially due to naming conflict).
pub fn try_create_int_counter(name: &str, help: &str) -> Result<IntCounter> {
    let opts = Opts::new(name, help);
    let counter = IntCounter::with_opts(opts)?;
    prometheus::register(Box::new(counter.clone()))?;
    Ok(counter)
}

/// Attempts to create an `IntGauge`, returning `Err` if the registry does not accept the counter
/// (potentially due to naming conflict).
pub fn try_create_int_gauge(name: &str, help: &str) -> Result<IntGauge> {
    let opts = Opts::new(name, help);
    let gauge = IntGauge::with_opts(opts)?;
    prometheus::register(Box::new(gauge.clone()))?;
    Ok(gauge)
}

/// Attempts to create a `Gauge`, returning `Err` if the registry does not accept the counter
/// (potentially due to naming conflict).
pub fn try_create_float_gauge(name: &str, help: &str) -> Result<Gauge> {
    let opts = Opts::new(name, help);
    let gauge = Gauge::with_opts(opts)?;
    prometheus::register(Box::new(gauge.clone()))?;
    Ok(gauge)
}

/// Attempts to create a `Histogram`, returning `Err` if the registry does not accept the counter
/// (potentially due to naming conflict).
pub fn try_create_histogram(name: &str, help: &str) -> Result<Histogram> {
    try_create_histogram_with_buckets(name, help, Ok(DEFAULT_BUCKETS.to_vec()))
}

/// Attempts to create a `Histogram` with specified buckets, returning `Err` if the registry does not accept the counter
/// (potentially due to naming conflict) or no valid buckets are provided.
pub fn try_create_histogram_with_buckets(
    name: &str,
    help: &str,
    buckets: Result<Vec<f64>>,
) -> Result<Histogram> {
    let opts = HistogramOpts::new(name, help).buckets(buckets?);
    let histogram = Histogram::with_opts(opts)?;
    prometheus::register(Box::new(histogram.clone()))?;
    Ok(histogram)
}

/// Attempts to create a `HistogramVec`, returning `Err` if the registry does not accept the counter
/// (potentially due to naming conflict).
pub fn try_create_histogram_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> Result<HistogramVec> {
    try_create_histogram_vec_with_buckets(name, help, Ok(DEFAULT_BUCKETS.to_vec()), label_names)
}

/// Attempts to create a `HistogramVec` with specified buckets, returning `Err` if the registry does not accept the counter
/// (potentially due to naming conflict) or no valid buckets are provided.
pub fn try_create_histogram_vec_with_buckets(
    name: &str,
    help: &str,
    buckets: Result<Vec<f64>>,
    label_names: &[&str],
) -> Result<HistogramVec> {
    let opts = HistogramOpts::new(name, help).buckets(buckets?);
    let histogram_vec = HistogramVec::new(opts, label_names)?;
    prometheus::register(Box::new(histogram_vec.clone()))?;
    Ok(histogram_vec)
}

/// Attempts to create a `IntGaugeVec`, returning `Err` if the registry does not accept the gauge
/// (potentially due to naming conflict).
pub fn try_create_int_gauge_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> Result<IntGaugeVec> {
    let opts = Opts::new(name, help);
    let counter_vec = IntGaugeVec::new(opts, label_names)?;
    prometheus::register(Box::new(counter_vec.clone()))?;
    Ok(counter_vec)
}

/// Attempts to create a `GaugeVec`, returning `Err` if the registry does not accept the gauge
/// (potentially due to naming conflict).
pub fn try_create_float_gauge_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> Result<GaugeVec> {
    let opts = Opts::new(name, help);
    let counter_vec = GaugeVec::new(opts, label_names)?;
    prometheus::register(Box::new(counter_vec.clone()))?;
    Ok(counter_vec)
}

/// Attempts to create a `IntCounterVec`, returning `Err` if the registry does not accept the gauge
/// (potentially due to naming conflict).
pub fn try_create_int_counter_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> Result<IntCounterVec> {
    let opts = Opts::new(name, help);
    let counter_vec = IntCounterVec::new(opts, label_names)?;
    prometheus::register(Box::new(counter_vec.clone()))?;
    Ok(counter_vec)
}

/// If `int_gauge_vec.is_ok()`, returns a gauge with the given `name`.
pub fn get_int_gauge(int_gauge_vec: &Result<IntGaugeVec>, name: &[&str]) -> Option<IntGauge> {
    if let Ok(int_gauge_vec) = int_gauge_vec {
        Some(int_gauge_vec.get_metric_with_label_values(name).ok()?)
    } else {
        None
    }
}

pub fn get_gauge<P: Atomic>(
    gauge_vec: &Result<GenericGaugeVec<P>>,
    name: &[&str],
) -> Option<GenericGauge<P>> {
    if let Ok(gauge_vec) = gauge_vec {
        Some(gauge_vec.get_metric_with_label_values(name).ok()?)
    } else {
        None
    }
}

pub fn set_gauge_entry<P: Atomic>(
    gauge_vec: &Result<GenericGaugeVec<P>>,
    name: &[&str],
    value: P::T,
) {
    if let Some(v) = get_gauge(gauge_vec, name) {
        v.set(value)
    };
}

/// If `int_gauge_vec.is_ok()`, sets the gauge with the given `name` to the given `value`
/// otherwise returns false.
pub fn set_int_gauge(int_gauge_vec: &Result<IntGaugeVec>, name: &[&str], value: i64) -> bool {
    if let Ok(int_gauge_vec) = int_gauge_vec {
        int_gauge_vec
            .get_metric_with_label_values(name)
            .map(|v| {
                v.set(value);
                true
            })
            .unwrap_or_else(|_| false)
    } else {
        false
    }
}

/// If `int_counter_vec.is_ok()`, returns a counter with the given `name`.
pub fn get_int_counter(
    int_counter_vec: &Result<IntCounterVec>,
    name: &[&str],
) -> Option<IntCounter> {
    if let Ok(int_counter_vec) = int_counter_vec {
        Some(int_counter_vec.get_metric_with_label_values(name).ok()?)
    } else {
        None
    }
}

/// Increments the `int_counter_vec` with the given `name`.
pub fn inc_counter_vec(int_counter_vec: &Result<IntCounterVec>, name: &[&str]) {
    if let Some(counter) = get_int_counter(int_counter_vec, name) {
        counter.inc()
    }
}

pub fn inc_counter_vec_by(int_counter_vec: &Result<IntCounterVec>, name: &[&str], amount: u64) {
    if let Some(counter) = get_int_counter(int_counter_vec, name) {
        counter.inc_by(amount);
    }
}

/// If `histogram_vec.is_ok()`, returns a histogram with the given `name`.
pub fn get_histogram(histogram_vec: &Result<HistogramVec>, name: &[&str]) -> Option<Histogram> {
    if let Ok(histogram_vec) = histogram_vec {
        Some(histogram_vec.get_metric_with_label_values(name).ok()?)
    } else {
        None
    }
}

/// Starts a timer on `vec` with the given `name`.
pub fn start_timer_vec(vec: &Result<HistogramVec>, name: &[&str]) -> Option<HistogramTimer> {
    get_histogram(vec, name).map(|h| h.start_timer())
}

/// Starts a timer for the given `Histogram`, stopping when it gets dropped or given to `stop_timer(..)`.
pub fn start_timer(histogram: &Result<Histogram>) -> Option<HistogramTimer> {
    if let Ok(histogram) = histogram {
        Some(histogram.start_timer())
    } else {
        None
    }
}

/// Starts a timer on `vec` with the given `name`.
pub fn observe_timer_vec(vec: &Result<HistogramVec>, name: &[&str], duration: Duration) {
    if let Some(h) = get_histogram(vec, name) {
        h.observe(duration_to_f64(duration))
    }
}

/// Stops a timer created with `start_timer(..)`.
pub fn stop_timer(timer: Option<HistogramTimer>) {
    if let Some(t) = timer {
        t.observe_duration()
    }
}

/// Stops a timer created with `start_timer(..)`.
///
/// Return the duration that the timer was running for, or 0.0 if it was `None` due to incorrect
/// initialisation.
pub fn stop_timer_with_duration(timer: Option<HistogramTimer>) -> Duration {
    Duration::from_secs_f64(timer.map_or(0.0, |t| t.stop_and_record()))
}

pub fn observe_vec(vec: &Result<HistogramVec>, name: &[&str], value: f64) {
    if let Some(h) = get_histogram(vec, name) {
        h.observe(value)
    }
}

pub fn inc_counter(counter: &Result<IntCounter>) {
    if let Ok(counter) = counter {
        counter.inc();
    }
}

pub fn inc_counter_by(counter: &Result<IntCounter>, value: u64) {
    if let Ok(counter) = counter {
        counter.inc_by(value);
    }
}

pub fn set_gauge_vec(int_gauge_vec: &Result<IntGaugeVec>, name: &[&str], value: i64) {
    if let Some(gauge) = get_int_gauge(int_gauge_vec, name) {
        gauge.set(value);
    }
}

pub fn inc_gauge_vec(int_gauge_vec: &Result<IntGaugeVec>, name: &[&str]) {
    if let Some(gauge) = get_int_gauge(int_gauge_vec, name) {
        gauge.inc();
    }
}

pub fn dec_gauge_vec(int_gauge_vec: &Result<IntGaugeVec>, name: &[&str]) {
    if let Some(gauge) = get_int_gauge(int_gauge_vec, name) {
        gauge.dec();
    }
}

pub fn set_gauge(gauge: &Result<IntGauge>, value: i64) {
    if let Ok(gauge) = gauge {
        gauge.set(value);
    }
}

pub fn set_float_gauge(gauge: &Result<Gauge>, value: f64) {
    if let Ok(gauge) = gauge {
        gauge.set(value);
    }
}

pub fn set_float_gauge_vec(gauge_vec: &Result<GaugeVec>, name: &[&str], value: f64) {
    if let Some(gauge) = get_gauge(gauge_vec, name) {
        gauge.set(value);
    }
}

pub fn inc_gauge(gauge: &Result<IntGauge>) {
    if let Ok(gauge) = gauge {
        gauge.inc();
    }
}

pub fn dec_gauge(gauge: &Result<IntGauge>) {
    if let Ok(gauge) = gauge {
        gauge.dec();
    }
}

pub fn maybe_set_gauge(gauge: &Result<IntGauge>, value_opt: Option<i64>) {
    if let Some(value) = value_opt {
        set_gauge(gauge, value)
    }
}

pub fn maybe_set_float_gauge(gauge: &Result<Gauge>, value_opt: Option<f64>) {
    if let Some(value) = value_opt {
        set_float_gauge(gauge, value)
    }
}

/// Sets the value of a `Histogram` manually.
pub fn observe(histogram: &Result<Histogram>, value: f64) {
    if let Ok(histogram) = histogram {
        histogram.observe(value);
    }
}

pub fn observe_duration(histogram: &Result<Histogram>, duration: Duration) {
    if let Ok(histogram) = histogram {
        histogram.observe(duration_to_f64(duration))
    }
}

fn duration_to_f64(duration: Duration) -> f64 {
    // This conversion was taken from here:
    //
    // https://docs.rs/prometheus/0.5.0/src/prometheus/histogram.rs.html#550-555
    let nanos = f64::from(duration.subsec_nanos()) / 1e9;
    duration.as_secs() as f64 + nanos
}

/// Create buckets using divisors of 10 multiplied by powers of 10, e.g.,
/// […, 0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, …]
///
/// The buckets go from `10^min_power` to `5 × 10^max_power`, inclusively.
/// The total number of buckets is `3 * (max_power - min_power + 1)`.
///
/// assert_eq!(vec![0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0], decimal_buckets(-1, 1));
/// assert_eq!(vec![1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0], decimal_buckets(0, 2));
pub fn decimal_buckets(min_power: i32, max_power: i32) -> Result<Vec<f64>> {
    if max_power < min_power {
        return Err(Error::Msg(format!(
            "decimal_buckets min_power needs to be <= max_power, given {} and {}",
            min_power, max_power
        )));
    }

    let mut buckets = Vec::with_capacity(3 * (max_power - min_power + 1) as usize);
    for n in min_power..=max_power {
        for m in &[1f64, 2f64, 5f64] {
            buckets.push(m * 10f64.powi(n))
        }
    }
    Ok(buckets)
}

/// Would be nice to use the `Try` trait bound and have a single implementation, but try_trait_v2
/// is not a stable feature yet.
pub trait TryExt {
    fn discard_timer_on_break(self, timer: &mut Option<HistogramTimer>) -> Self;
}

impl<T, E> TryExt for std::result::Result<T, E> {
    fn discard_timer_on_break(self, timer_opt: &mut Option<HistogramTimer>) -> Self {
        if self.is_err() {
            if let Some(timer) = timer_opt.take() {
                timer.stop_and_discard();
            }
        }
        self
    }
}

impl<T> TryExt for Option<T> {
    fn discard_timer_on_break(self, timer_opt: &mut Option<HistogramTimer>) -> Self {
        if self.is_none() {
            if let Some(timer) = timer_opt.take() {
                timer.stop_and_discard();
            }
        }
        self
    }
}
