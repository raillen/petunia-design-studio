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

/// Round brush parameters in linear working RGB. Flow deposits each dab;
/// opacity limits the complete stroke's coverage against the original surface.
#[derive(Debug, Clone, Copy)]
pub struct RoundBrush {
    pub color: [f32; 4],
    pub hardness: f32,
    pub flow: f32,
    pub opacity: f32,
    pub blend: petunia_core::BlendMode,
}

/// An isolated, cancellable COW stroke. The source remains unchanged until
/// the resulting tile transaction is explicitly applied or PNG edit committed.
#[derive(Debug, Clone)]
pub struct RasterStroke {
    before: crate::tiles::TileStore,
    preview: crate::tiles::TileStore,
    coverage: std::collections::HashMap<(u32, u32), f32>,
    width: u32,
    height: u32,
    brush: RoundBrush,
}

impl RasterStroke {
    pub fn begin(
        source: &crate::tiles::TileStore,
        width: u32,
        height: u32,
        brush: RoundBrush,
    ) -> crate::Result<Self> {
        if width == 0
            || height == 0
            || u64::from(width) * u64::from(height) > 64 << 20
            || source.tile_size() == 0
        {
            return Err(crate::EngineError::Execution(
                "raster dimensions exceed allocation guard".into(),
            ));
        }
        if !brush
            .color
            .iter()
            .chain([&brush.hardness, &brush.flow, &brush.opacity])
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        {
            return Err(crate::EngineError::Execution(
                "invalid round brush parameters".into(),
            ));
        }
        let mut preview = source.clone();
        preview.clear_dirty();
        Ok(Self {
            before: source.clone(),
            preview,
            coverage: std::collections::HashMap::new(),
            width,
            height,
            brush,
        })
    }
    #[must_use]
    pub fn preview(&self) -> &crate::tiles::TileStore {
        &self.preview
    }

    /// Deposit one dab, multiplying its coverage by the session selection.
    /// All allocations and edits happen on a temporary version; failure leaves
    /// both the source and this preview unchanged.
    pub fn dab(
        &mut self,
        dab: DabSpec,
        selection: Option<&crate::filter::Surface>,
    ) -> crate::Result<()> {
        if !dab.position.x.is_finite()
            || !dab.position.y.is_finite()
            || !dab.diameter.is_finite()
            || dab.diameter <= 0.0
            || !dab.opacity.is_finite()
            || !(0.0..=1.0).contains(&dab.opacity)
        {
            return Err(crate::EngineError::Execution("invalid dab".into()));
        }
        if selection.is_some_and(|mask| {
            mask.width != self.width
                || mask.height != self.height
                || mask.pixels.len() != self.width as usize * self.height as usize * 4
        }) {
            return Err(crate::EngineError::Execution(
                "selection dimensions mismatch".into(),
            ));
        }
        let radius = dab.diameter / 2.0;
        let left = (dab.position.x - radius)
            .floor()
            .max(0.0)
            .min(f64::from(self.width)) as u32;
        let top = (dab.position.y - radius)
            .floor()
            .max(0.0)
            .min(f64::from(self.height)) as u32;
        let right = (dab.position.x + radius)
            .ceil()
            .max(0.0)
            .min(f64::from(self.width)) as u32;
        let bottom = (dab.position.y + radius)
            .ceil()
            .max(0.0)
            .min(f64::from(self.height)) as u32;
        if u64::from(right - left) * u64::from(bottom - top) > 16 << 20 {
            return Err(crate::EngineError::Execution(
                "dab temporary ROI exceeds guard".into(),
            ));
        }
        let mut preview = self.preview.clone();
        let mut coverage = self.coverage.clone();
        for y in top..bottom {
            for x in left..right {
                let distance = (f64::from(x) + 0.5 - dab.position.x)
                    .hypot(f64::from(y) + 0.5 - dab.position.y)
                    / radius;
                if distance >= 1.0 {
                    continue;
                }
                let hardness = f64::from(self.brush.hardness);
                let falloff = if distance <= hardness || hardness == 1.0 {
                    1.0
                } else {
                    (1.0 - distance) / (1.0 - hardness)
                } as f32;
                let mask = selection.map_or(1.0, |mask| {
                    mask.sample(
                        i64::from(x),
                        i64::from(y),
                        3,
                        crate::filter::EdgeMode::Transparent,
                    )
                    .clamp(0.0, 1.0)
                });
                let deposit = (falloff * mask * self.brush.flow * dab.opacity).clamp(0.0, 1.0);
                if deposit == 0.0 {
                    continue;
                }
                let amount = coverage.entry((x, y)).or_default();
                *amount = 1.0 - (1.0 - *amount) * (1.0 - deposit);
                let alpha = *amount * self.brush.opacity * self.brush.color[3];
                let size = preview.tile_size();
                let coord = petunia_core::TileCoord {
                    x: x / size,
                    y: y / size,
                };
                let tile_width = size.min(self.width - coord.x * size);
                let tile_height = size.min(self.height - coord.y * size);
                let offset = ((y % size) * tile_width + x % size) as usize * 4;
                let mut backdrop = [0.0; 4];
                if let Some(tile) = self.before.get(coord) {
                    if tile.format != petunia_core::PixelFormat::Rgba8Unorm
                        || tile.width != tile_width
                        || tile.height != tile_height
                        || offset + 4 > tile.bytes.len()
                    {
                        return Err(crate::EngineError::Execution(
                            "brush requires consistent RGBA8 surface tiles".into(),
                        ));
                    }
                    for (channel, component) in backdrop.iter_mut().enumerate().take(3) {
                        *component = decode_srgb(f32::from(tile.bytes[offset + channel]) / 255.0);
                    }
                    backdrop[3] = f32::from(tile.bytes[offset + 3]) / 255.0;
                }
                let rgb = petunia_render_model::apply_blend(
                    [
                        self.brush.color[0],
                        self.brush.color[1],
                        self.brush.color[2],
                    ],
                    [backdrop[0], backdrop[1], backdrop[2]],
                    self.brush.blend,
                );
                let output_alpha = alpha + backdrop[3] * (1.0 - alpha);
                let bytes = preview.write_tile(
                    coord,
                    tile_width,
                    tile_height,
                    petunia_core::PixelFormat::Rgba8Unorm,
                )?;
                for channel in 0..3 {
                    let premul = alpha * (1.0 - backdrop[3]) * self.brush.color[channel]
                        + alpha * backdrop[3] * rgb[channel]
                        + (1.0 - alpha) * backdrop[3] * backdrop[channel];
                    let linear = if output_alpha > 0.0 {
                        premul / output_alpha
                    } else {
                        0.0
                    };
                    bytes[offset + channel] =
                        (encode_srgb(linear).clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                bytes[offset + 3] = (output_alpha * 255.0).round() as u8;
            }
        }
        self.preview = preview;
        self.coverage = coverage;
        Ok(())
    }
    #[must_use]
    pub fn finish(self) -> crate::tiles::TileTransaction {
        crate::tiles::TileTransaction::new(self.before, self.preview)
    }

    /// Create one authorial resource-reference transaction. Bytes go to the
    /// caller's BlobStore before the commit; failed/undone refs keep immutable
    /// bytes for history/recovery. No pixel data enters a scene node.
    pub fn materialize(&self, object: petunia_core::ObjectId) -> crate::Result<RasterEdit> {
        use image::ImageEncoder;
        let count = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|value| value.checked_mul(4))
            .filter(|bytes| *bytes <= 256 << 20)
            .ok_or_else(|| {
                crate::EngineError::Execution("PNG materialization allocation guard".into())
            })?;
        let mut pixels = vec![0u8; count];
        let size = self.preview.tile_size();
        for y in 0..self.height {
            for x in 0..self.width {
                let coord = petunia_core::TileCoord {
                    x: x / size,
                    y: y / size,
                };
                if let Some(tile) = self.preview.get(coord) {
                    if tile.format != petunia_core::PixelFormat::Rgba8Unorm {
                        return Err(crate::EngineError::Execution(
                            "PNG requires RGBA8 tiles".into(),
                        ));
                    }
                    let source = ((y % size) * tile.width + x % size) as usize * 4;
                    let destination = (y * self.width + x) as usize * 4;
                    let bytes = tile.bytes.get(source..source + 4).ok_or_else(|| {
                        crate::EngineError::Execution("truncated raster tile".into())
                    })?;
                    pixels[destination..destination + 4].copy_from_slice(bytes);
                }
            }
        }
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(
                &pixels,
                self.width,
                self.height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|error| crate::EngineError::Execution(format!("PNG encode: {error}")))?;
        let resource = petunia_core::ResourceId::new_v4();
        let record = petunia_core::ResourceRecord {
            id: resource,
            kind: petunia_core::ResourceKind::Image,
            source: petunia_core::ResourceSource::Embedded {
                entry: format!("{resource}.png"),
            },
            content_hash: Some(petunia_core::ContentHash::new(&png)),
            metadata: petunia_core::ResourceMetadata {
                byte_size: Some(png.len() as u64),
                font_policy: None,
            },
        };
        let request = crate::TransactionRequest {
            command_id: crate::CommandId::new_v4(),
            operations: vec![
                crate::DocumentOp::Registry(crate::transaction::RegistryOp::Resource {
                    id: resource,
                    value: Some(Box::new(record)),
                }),
                crate::DocumentOp::SetPixelSurface {
                    object,
                    surface: petunia_core::PixelSurfaceRef { resource },
                },
            ],
            merge_key: None,
        };
        Ok(RasterEdit {
            png,
            resource,
            request,
        })
    }
}
#[derive(Debug)]
pub struct RasterEdit {
    pub png: Vec<u8>,
    pub resource: petunia_core::ResourceId,
    pub request: crate::TransactionRequest,
}
fn decode_srgb(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
fn encode_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}
