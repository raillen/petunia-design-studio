//! Brush stroke pipeline: samples, stabilization, resampling and dabs.
//!
//! The UI delivers raw pointer samples; the engine owns stroke shape.
//! Every stage is deterministic for fixed input: jitter derives from
//! an explicit per-stroke seed, never from a global RNG or clock.

use petunia_core::Point;
use serde::{Deserialize, Serialize};

/// One raw pointer sample from mouse, pen or another device.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokeSample {
    pub position: Point,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub rotation: f32,
    pub time: f64,
}

/// Stabilizer policy persisted in presets. Internal filter buffers
/// never persist.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Stabilizer {
    Off,
    OneEuro(OneEuroParams),
}

/// One Euro adaptive filter parameters: low-speed smoothing,
/// speed response and derivative smoothing. All finite.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OneEuroParams {
    pub min_cutoff: f64,
    pub beta: f64,
    pub derivative_cutoff: f64,
}

impl OneEuroParams {
    /// Cutoffs must be finite and positive; `beta` may be zero for a
    /// constant cutoff (pure low-pass behavior).
    pub fn new(min_cutoff: f64, beta: f64, derivative_cutoff: f64) -> Option<Self> {
        if [min_cutoff, derivative_cutoff]
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
            && beta.is_finite()
            && beta >= 0.0
        {
            Some(Self {
                min_cutoff,
                beta,
                derivative_cutoff,
            })
        } else {
            None
        }
    }
}

/// One Euro filter state over 2D positions.
#[derive(Debug, Clone)]
pub struct OneEuroFilter {
    params: OneEuroParams,
    last_time: Option<f64>,
    last_position: Option<Point>,
    last_velocity: Point,
    low_pass: Point,
    derivative_low_pass: Point,
}

impl OneEuroFilter {
    /// Fresh filter from validated parameters.
    #[must_use]
    pub fn new(params: OneEuroParams) -> Self {
        Self {
            params,
            last_time: None,
            last_position: None,
            last_velocity: Point::new(0.0, 0.0),
            low_pass: Point::new(0.0, 0.0),
            derivative_low_pass: Point::new(0.0, 0.0),
        }
    }

    /// Filter one sample, returning the stabilized position. Slow
    /// motion smooths more; fast motion answers quicker.
    pub fn filter(&mut self, position: Point, time: f64) -> Point {
        let Some(previous_time) = self.last_time else {
            self.last_time = Some(time);
            self.last_position = Some(position);
            self.low_pass = position;
            return position;
        };
        let dt = (time - previous_time).max(f64::EPSILON);
        let previous = self.last_position.unwrap_or(position);
        let velocity = Point::new(
            (position.x - previous.x) / dt,
            (position.y - previous.y) / dt,
        );
        self.derivative_low_pass = low_pass_point(
            self.derivative_low_pass,
            velocity,
            smoothing_factor(dt, self.params.derivative_cutoff),
        );
        let speed = self.derivative_low_pass.x.hypot(self.derivative_low_pass.y);
        let cutoff = self.params.min_cutoff + self.params.beta * speed;
        self.low_pass = low_pass_point(self.low_pass, position, smoothing_factor(dt, cutoff));
        self.last_velocity = velocity;
        self.last_time = Some(time);
        self.last_position = Some(position);
        self.low_pass
    }

    /// Last estimated velocity, for dynamics mapping.
    #[must_use]
    pub fn velocity(&self) -> Point {
        self.last_velocity
    }
}

fn smoothing_factor(dt: f64, cutoff: f64) -> f64 {
    let tau = 1.0 / (2.0 * std::f64::consts::PI * cutoff);
    1.0 / (1.0 + tau / dt)
}

fn low_pass_point(previous: Point, value: Point, factor: f64) -> Point {
    Point::new(
        previous.x + factor * (value.x - previous.x),
        previous.y + factor * (value.y - previous.y),
    )
}

/// Resample a stroke by arc length so dab density stops depending on
/// device event rate. Positions interpolate linearly; pressure, tilt
/// and rotation interpolate continuously (no stepwise copies), and
/// timestamps scale proportionally.
#[must_use]
pub fn resample_arc_length(samples: &[StrokeSample], spacing: f64) -> Vec<StrokeSample> {
    if samples.len() < 2 || !(spacing.is_finite() && spacing > 0.0) {
        return samples.to_vec();
    }
    let mut lengths = vec![0.0];
    for pair in samples.windows(2) {
        let step = (pair[1].position.x - pair[0].position.x)
            .hypot(pair[1].position.y - pair[0].position.y);
        lengths.push(lengths.last().expect("non-empty") + step);
    }
    let total = *lengths.last().expect("non-empty");
    if total <= 0.0 {
        return vec![samples[0]];
    }
    let mut out = vec![samples[0]];
    let mut target = spacing;
    let mut segment = 1;
    while target < total {
        while segment < lengths.len() - 1 && lengths[segment] < target {
            segment += 1;
        }
        let before = &samples[segment - 1];
        let after = &samples[segment];
        let span = lengths[segment] - lengths[segment - 1];
        let t = if span > 0.0 {
            ((target - lengths[segment - 1]) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(StrokeSample {
            position: Point::new(
                before.position.x + t * (after.position.x - before.position.x),
                before.position.y + t * (after.position.y - before.position.y),
            ),
            pressure: before.pressure + t as f32 * (after.pressure - before.pressure),
            tilt_x: before.tilt_x + t as f32 * (after.tilt_x - before.tilt_x),
            tilt_y: before.tilt_y + t as f32 * (after.tilt_y - before.tilt_y),
            rotation: before.rotation + t as f32 * (after.rotation - before.rotation),
            time: before.time + t * (after.time - before.time),
        });
        target += spacing;
    }
    out.push(*samples.last().expect("non-empty"));
    out
}

/// Deterministic per-dab jitter from an explicit stroke seed. Same
/// seed plus dab index plus channel always yields the same value, so
/// preview and commit agree without rerunning random state.
#[must_use]
pub fn dab_jitter(seed: u64, dab_index: u64, channel: u64) -> f64 {
    let mut state = seed
        .wrapping_add(dab_index.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(channel.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    state ^= state >> 30;
    state = state.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^= state >> 31;
    // Unit interval from the top 53 bits.
    ((state >> 11) as f64) / ((1u64 << 53) as f64)
}

/// One deposited dab: evaluated position, size, opacity and color
/// multiplier after dynamics and spacing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DabSpec {
    pub position: Point,
    pub diameter: f64,
    pub opacity: f32,
    pub jitter: f64,
}

/// Lay dabs along resampled points: spacing derives from the
/// effective diameter times `spacing_ratio`, keeping density stable
/// across brush sizes.
#[must_use]
pub fn layout_dabs(
    samples: &[StrokeSample],
    diameter: f64,
    spacing_ratio: f64,
    base_opacity: f32,
    seed: u64,
) -> Vec<DabSpec> {
    if samples.is_empty() || !(diameter.is_finite() && diameter > 0.0) {
        return Vec::new();
    }
    let ratio = if spacing_ratio.is_finite() && spacing_ratio > 0.0 {
        spacing_ratio
    } else {
        0.1
    };
    let points = resample_arc_length(samples, diameter * ratio);
    points
        .into_iter()
        .enumerate()
        .map(|(index, sample)| DabSpec {
            position: sample.position,
            diameter,
            opacity: (base_opacity * sample.pressure).clamp(0.0, 1.0),
            jitter: dab_jitter(seed, index as u64, 0),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(x: f64, time: f64) -> StrokeSample {
        StrokeSample {
            position: Point::new(x, 0.0),
            pressure: 0.8,
            tilt_x: 0.0,
            tilt_y: 0.0,
            rotation: 0.0,
            time,
        }
    }

    #[test]
    fn one_euro_params_validate() {
        assert!(OneEuroParams::new(1.0, 0.5, 1.0).is_some());
        assert!(OneEuroParams::new(0.0, 0.5, 1.0).is_none());
        assert!(OneEuroParams::new(1.0, f64::NAN, 1.0).is_none());
    }

    #[test]
    fn filter_converges_on_steady_input() {
        let params = OneEuroParams::new(1.0, 0.0, 1.0).expect("valid");
        let mut filter = OneEuroFilter::new(params);
        let mut last = Point::new(0.0, 0.0);
        for step in 0..200 {
            last = filter.filter(Point::new(10.0, 0.0), step as f64 * 0.008);
        }
        assert!((last.x - 10.0).abs() < 0.5, "settled at {last:?}");
    }

    #[test]
    fn resampling_spaces_by_distance_not_time() {
        // Clustered-then-sparse input still yields uniform dabs.
        let samples = vec![
            sample(0.0, 0.0),
            sample(1.0, 0.001),
            sample(2.0, 0.002),
            sample(20.0, 1.0),
        ];
        let out = resample_arc_length(&samples, 5.0);
        assert!(out.len() >= 5, "got {}", out.len());
        for pair in out.windows(2) {
            let gap = (pair[1].position.x - pair[0].position.x)
                .hypot(pair[1].position.y - pair[0].position.y);
            assert!(gap <= 5.0 + 1e-9, "gap {gap}");
        }
        // Pressure interpolates instead of stepping.
        assert!(out[1].pressure > 0.0 && out[1].pressure <= 0.8);
    }

    #[test]
    fn jitter_is_deterministic_per_seed_and_index() {
        assert_eq!(dab_jitter(42, 7, 0), dab_jitter(42, 7, 0));
        assert_ne!(dab_jitter(42, 7, 0), dab_jitter(43, 7, 0));
        assert_ne!(dab_jitter(42, 7, 0), dab_jitter(42, 8, 0));
        let value = dab_jitter(42, 7, 0);
        assert!((0.0..1.0).contains(&value));
    }

    #[test]
    fn dab_layout_derives_spacing_from_diameter() {
        let samples: Vec<StrokeSample> = (0..=10)
            .map(|i| sample(i as f64 * 2.0, i as f64 * 0.016))
            .collect();
        let dabs = layout_dabs(&samples, 10.0, 0.5, 1.0, 7);
        // 20 units long at 5-unit spacing plus endpoints.
        assert!((4..=6).contains(&dabs.len()), "got {}", dabs.len());
        assert!(dabs.iter().all(|dab| dab.diameter == 10.0));
    }
}
