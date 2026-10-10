//! Immutable, bounded raster analysis shared by palette extraction and live trace.
//!
//! The input contract is straight-alpha RGBA8 in color-managed sRGB. Resource
//! decoding/profile conversion belongs to the caller; source bytes are never
//! mutated. Oklab conversion, median-cut initialization and k-means are derived.

use crate::jobs::CancelToken;
use petunia_core::color::{
    BuiltinColorSpace, ColorSpaceRef, ProcessColor, ProcessColorValue, Rgba,
};
use petunia_core::ColorValue;
use thiserror::Error;

pub const ANALYSIS_SEMANTIC_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum AnalysisError {
    #[error("analysis cancelled")]
    Cancelled,
    #[error("invalid analysis input: {0}")]
    InvalidInput(String),
    #[error("analysis limit exceeded: {0}; reduce source dimensions or color count")]
    LimitExceeded(&'static str),
    #[error("unsupported analysis color context; convert input/reference to sRGB first")]
    UnsupportedColorContext,
    #[error("contour extraction failed to produce a closed boundary")]
    InvalidBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaSnapshot {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl RgbaSnapshot {
    pub fn new(width: usize, height: usize, pixels: Vec<u8>) -> Result<Self, AnalysisError> {
        if width == 0
            || height == 0
            || width
                .checked_mul(height)
                .and_then(|count| count.checked_mul(4))
                != Some(pixels.len())
        {
            return Err(AnalysisError::InvalidInput(
                "RGBA dimensions/byte count mismatch".into(),
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub(crate) fn pixel(&self, index: usize) -> [u8; 4] {
        let offset = index * 4;
        [
            self.pixels[offset],
            self.pixels[offset + 1],
            self.pixels[offset + 2],
            self.pixels[offset + 3],
        ]
    }
}

/// Operational limits apply before allocations and during region/vertex growth.
#[derive(Debug, Clone, Copy)]
pub struct AnalysisLimits {
    pub max_pixels: usize,
    pub max_colors: usize,
    pub max_regions: usize,
    pub max_vertices: usize,
    pub max_temporary_bytes: usize,
    pub max_samples: usize,
    pub kmeans_iterations: usize,
}

impl Default for AnalysisLimits {
    fn default() -> Self {
        Self {
            max_pixels: 4_194_304,
            max_colors: 64,
            max_regions: 16_384,
            max_vertices: 1_000_000,
            max_temporary_bytes: 256 * 1024 * 1024,
            max_samples: 16_384,
            kmeans_iterations: 20,
        }
    }
}

impl AnalysisLimits {
    pub(crate) fn validate(
        &self,
        image: &RgbaSnapshot,
        colors: usize,
    ) -> Result<(), AnalysisError> {
        let pixels = image.width * image.height;
        if colors == 0 {
            return Err(AnalysisError::InvalidInput(
                "color count must be positive".into(),
            ));
        }
        if colors > self.max_colors || colors > 256 {
            return Err(AnalysisError::LimitExceeded("requested color count"));
        }
        if pixels > self.max_pixels {
            return Err(AnalysisError::LimitExceeded("source pixel count"));
        }
        // Conservative reservation: label/region maps, queue, worst-case boundaries,
        // and sample tables. Avoid constructing oversized temporary work first.
        let estimate = pixels.checked_mul(64).and_then(|bytes| {
            self.max_samples
                .checked_mul(128)
                .and_then(|samples| bytes.checked_add(samples))
        });
        if estimate.is_none_or(|bytes| bytes > self.max_temporary_bytes) {
            return Err(AnalysisError::LimitExceeded("temporary allocation budget"));
        }
        if self.max_samples == 0 || self.kmeans_iterations == 0 || self.kmeans_iterations > 100 {
            return Err(AnalysisError::InvalidInput(
                "sampling/iteration guard is invalid".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn check_cancel(token: &CancelToken) -> Result<(), AnalysisError> {
    if token.is_cancelled() {
        Err(AnalysisError::Cancelled)
    } else {
        Ok(())
    }
}

/// Palette alpha behavior is explicit; transparent samples never invent colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaletteAlphaPolicy {
    IgnoreTransparent {
        threshold: f32,
    },
    WeightByAlpha,
    CompositeAgainst {
        rgb: [f32; 3],
    },
    /// Used by Core's trace `Ignore`: inspect RGB even in zero-alpha pixels.
    IgnoreAlpha,
}

#[derive(Debug, Clone)]
pub struct PaletteExtractionSpec {
    pub colors: u16,
    pub alpha_policy: PaletteAlphaPolicy,
    pub min_population: f32,
    /// Euclidean Oklab distance, not encoded RGB distance.
    pub merge_delta: f32,
}

impl Default for PaletteExtractionSpec {
    fn default() -> Self {
        Self {
            colors: 8,
            alpha_policy: PaletteAlphaPolicy::WeightByAlpha,
            min_population: 0.0,
            merge_delta: 0.01,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedColor {
    pub color: ColorValue,
    pub population: f32,
}

/// One shared implementation supplies both extraction and trace assignment maps.
#[derive(Debug, Default)]
pub struct ColorQuantizer;

#[derive(Debug, Clone)]
pub struct Quantization {
    /// Stable descending population order; geometric ties use Oklab channels.
    pub palette: Vec<ExtractedColor>,
    /// A pixel belongs to exactly one palette entry, or no entry for transparency.
    pub assignments: Vec<Option<u16>>,
    pub(crate) centers: Vec<[f64; 3]>,
}

#[derive(Clone)]
struct Sample {
    lab: [f64; 3],
    weight: f64,
}

impl ColorQuantizer {
    pub fn quantize(
        &self,
        image: &RgbaSnapshot,
        colors: u16,
        alpha: PaletteAlphaPolicy,
        limits: &AnalysisLimits,
        token: &CancelToken,
    ) -> Result<Quantization, AnalysisError> {
        check_cancel(token)?;
        limits.validate(image, usize::from(colors))?;
        validate_alpha(alpha)?;
        let samples = stratified_samples(image, alpha, limits.max_samples, token)?;
        if samples.is_empty() {
            return Ok(Quantization {
                palette: Vec::new(),
                assignments: vec![None; image.width * image.height],
                centers: Vec::new(),
            });
        }
        let mut centers = median_cut(samples.clone(), usize::from(colors), token)?;
        for _ in 0..limits.kmeans_iterations {
            check_cancel(token)?;
            let mut sums = vec![[0.0; 4]; centers.len()];
            for (index, sample) in samples.iter().enumerate() {
                if index % 1024 == 0 {
                    check_cancel(token)?;
                }
                let nearest = nearest_center(sample.lab, &centers);
                for (channel, sum) in sums[nearest].iter_mut().enumerate().take(3) {
                    *sum += sample.lab[channel] * sample.weight;
                }
                sums[nearest][3] += sample.weight;
            }
            let next: Vec<_> = sums
                .iter()
                .filter(|sum| sum[3] > 0.0)
                .map(|sum| [sum[0] / sum[3], sum[1] / sum[3], sum[2] / sum[3]])
                .collect();
            let stable = next.len() == centers.len()
                && next
                    .iter()
                    .zip(&centers)
                    .all(|(a, b)| distance(*a, *b) <= 1e-12);
            centers = next;
            if stable {
                break;
            }
        }
        let mut assignments = Vec::with_capacity(image.width * image.height);
        let mut populations = vec![0.0; centers.len()];
        for index in 0..image.width * image.height {
            if index % 4096 == 0 {
                check_cancel(token)?;
            }
            if let Some(sample) = sample_pixel(image.pixel(index), alpha) {
                let cluster = nearest_center(sample.lab, &centers);
                populations[cluster] += sample.weight;
                assignments.push(Some(cluster as u16));
            } else {
                assignments.push(None);
            }
        }
        let mut order: Vec<usize> = (0..centers.len())
            .filter(|&index| populations[index] > 0.0)
            .collect();
        order.sort_by(|&a, &b| {
            populations[b]
                .total_cmp(&populations[a])
                .then_with(|| compare_lab(centers[a], centers[b]))
        });
        let total: f64 = populations.iter().sum();
        let mut remap = vec![None; centers.len()];
        for (new, &old) in order.iter().enumerate() {
            remap[old] = Some(new as u16);
        }
        for assignment in &mut assignments {
            *assignment = assignment.and_then(|old| remap[usize::from(old)]);
        }
        let palette = order
            .iter()
            .map(|&index| ExtractedColor {
                color: srgb_color(oklab_to_srgb(centers[index])),
                population: (populations[index] / total) as f32,
            })
            .collect();
        centers = order.iter().map(|&index| centers[index]).collect();
        Ok(Quantization {
            palette,
            assignments,
            centers,
        })
    }
}

pub fn extract_palette(
    image: &RgbaSnapshot,
    spec: &PaletteExtractionSpec,
    limits: &AnalysisLimits,
    token: &CancelToken,
) -> Result<Vec<ExtractedColor>, AnalysisError> {
    if !spec.min_population.is_finite()
        || !(0.0..=1.0).contains(&spec.min_population)
        || !spec.merge_delta.is_finite()
        || spec.merge_delta < 0.0
    {
        return Err(AnalysisError::InvalidInput(
            "population/merge parameters out of range".into(),
        ));
    }
    let quantized =
        ColorQuantizer.quantize(image, spec.colors, spec.alpha_policy, limits, token)?;
    let mut merged: Vec<([f64; 3], f64)> = Vec::new();
    for (entry, lab) in quantized.palette.iter().zip(quantized.centers) {
        check_cancel(token)?;
        let population = f64::from(entry.population);
        if let Some((center, weight)) = merged
            .iter_mut()
            .find(|(center, _)| distance(*center, lab) <= f64::from(spec.merge_delta).powi(2))
        {
            for channel in 0..3 {
                center[channel] = (center[channel] * *weight + lab[channel] * population)
                    / (*weight + population);
            }
            *weight += population;
        } else {
            merged.push((lab, population));
        }
    }
    merged.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| compare_lab(a.0, b.0)));
    Ok(merged
        .into_iter()
        .filter(|(_, population)| *population >= f64::from(spec.min_population))
        .map(|(lab, population)| ExtractedColor {
            color: srgb_color(oklab_to_srgb(lab)),
            population: population as f32,
        })
        .collect())
}

fn validate_alpha(alpha: PaletteAlphaPolicy) -> Result<(), AnalysisError> {
    let valid = match alpha {
        PaletteAlphaPolicy::IgnoreTransparent { threshold } => {
            threshold.is_finite() && (0.0..=1.0).contains(&threshold)
        }
        PaletteAlphaPolicy::CompositeAgainst { rgb } => rgb
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value)),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(AnalysisError::InvalidInput(
            "invalid alpha parameters".into(),
        ))
    }
}

fn sample_pixel(pixel: [u8; 4], policy: PaletteAlphaPolicy) -> Option<Sample> {
    let alpha = f64::from(pixel[3]) / 255.0;
    let mut rgb = [
        f64::from(pixel[0]) / 255.0,
        f64::from(pixel[1]) / 255.0,
        f64::from(pixel[2]) / 255.0,
    ];
    let weight = match policy {
        PaletteAlphaPolicy::IgnoreTransparent { threshold } => {
            if alpha <= f64::from(threshold) {
                return None;
            }
            1.0
        }
        PaletteAlphaPolicy::WeightByAlpha => {
            if alpha == 0.0 {
                return None;
            }
            alpha
        }
        PaletteAlphaPolicy::CompositeAgainst { rgb: background } => {
            // Composite in linear light, then encode for the shared conversion.
            for channel in 0..3 {
                rgb[channel] = encode_srgb(
                    decode_srgb(rgb[channel]) * alpha
                        + decode_srgb(f64::from(background[channel])) * (1.0 - alpha),
                );
            }
            1.0
        }
        PaletteAlphaPolicy::IgnoreAlpha => 1.0,
    };
    Some(Sample {
        lab: srgb_to_oklab(rgb),
        weight,
    })
}

fn stratified_samples(
    image: &RgbaSnapshot,
    policy: PaletteAlphaPolicy,
    maximum: usize,
    token: &CancelToken,
) -> Result<Vec<Sample>, AnalysisError> {
    let count = image.width * image.height;
    if count <= maximum {
        let mut samples = Vec::with_capacity(count);
        for index in 0..count {
            if index % 4096 == 0 {
                check_cancel(token)?;
            }
            if let Some(sample) = sample_pixel(image.pixel(index), policy) {
                samples.push(sample);
            }
        }
        return Ok(samples);
    }
    let columns = ((maximum as f64 * image.width as f64 / image.height as f64).sqrt() as usize)
        .clamp(1, image.width)
        .min(maximum);
    let rows = (maximum / columns).clamp(1, image.height);
    let mut samples = Vec::with_capacity(columns * rows);
    for row in 0..rows {
        check_cancel(token)?;
        let top = row * image.height / rows;
        let bottom = (row + 1) * image.height / rows;
        for column in 0..columns {
            let left = column * image.width / columns;
            let right = (column + 1) * image.width / columns;
            let index = ((top + bottom - 1) / 2) * image.width + (left + right - 1) / 2;
            if let Some(mut sample) = sample_pixel(image.pixel(index), policy) {
                sample.weight *= ((right - left) * (bottom - top)) as f64;
                samples.push(sample);
            }
        }
    }
    Ok(samples)
}

fn median_cut(
    samples: Vec<Sample>,
    count: usize,
    token: &CancelToken,
) -> Result<Vec<[f64; 3]>, AnalysisError> {
    let mut boxes = vec![samples];
    while boxes.len() < count {
        check_cancel(token)?;
        let candidate = boxes
            .iter()
            .enumerate()
            .filter(|(_, samples)| samples.len() > 1)
            .map(|(index, samples)| {
                let (axis, spread) = dominant_axis(samples);
                (index, axis, spread)
            })
            .filter(|(_, _, spread)| *spread > 1e-12)
            .max_by(|a, b| a.2.total_cmp(&b.2).then_with(|| b.0.cmp(&a.0)));
        let Some((index, axis, _)) = candidate else {
            break;
        };
        let mut samples = boxes.remove(index);
        samples.sort_by(|a, b| {
            a.lab[axis]
                .total_cmp(&b.lab[axis])
                .then_with(|| compare_lab(a.lab, b.lab))
        });
        let total: f64 = samples.iter().map(|sample| sample.weight).sum();
        let mut accumulated = 0.0;
        let mut split = 1;
        for (index, sample) in samples.iter().take(samples.len() - 1).enumerate() {
            accumulated += sample.weight;
            split = index + 1;
            if accumulated >= total / 2.0 {
                break;
            }
        }
        let right = samples.split_off(split);
        boxes.push(samples);
        boxes.push(right);
    }
    Ok(boxes
        .iter()
        .map(|samples| {
            let mut sum = [0.0; 3];
            let mut weight = 0.0;
            for sample in samples {
                for (channel, total) in sum.iter_mut().enumerate() {
                    *total += sample.lab[channel] * sample.weight;
                }
                weight += sample.weight;
            }
            [sum[0] / weight, sum[1] / weight, sum[2] / weight]
        })
        .collect())
}

fn dominant_axis(samples: &[Sample]) -> (usize, f64) {
    let mut minimum = [f64::INFINITY; 3];
    let mut maximum = [f64::NEG_INFINITY; 3];
    for sample in samples {
        for channel in 0..3 {
            minimum[channel] = minimum[channel].min(sample.lab[channel]);
            maximum[channel] = maximum[channel].max(sample.lab[channel]);
        }
    }
    (0..3)
        .map(|axis| (axis, maximum[axis] - minimum[axis]))
        .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        .unwrap_or((0, 0.0))
}

fn compare_lab(a: [f64; 3], b: [f64; 3]) -> std::cmp::Ordering {
    a[0].total_cmp(&b[0])
        .then_with(|| a[1].total_cmp(&b[1]))
        .then_with(|| a[2].total_cmp(&b[2]))
}
pub(crate) fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3)
        .map(|channel| (a[channel] - b[channel]).powi(2))
        .sum()
}
fn nearest_center(lab: [f64; 3], centers: &[[f64; 3]]) -> usize {
    (0..centers.len())
        .min_by(|&a, &b| distance(lab, centers[a]).total_cmp(&distance(lab, centers[b])))
        .unwrap_or(0)
}
pub(crate) fn decode_srgb(channel: f64) -> f64 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}
fn encode_srgb(channel: f64) -> f64 {
    if channel <= 0.0031308 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}
pub(crate) fn srgb_to_oklab(rgb: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = rgb.map(decode_srgb);
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}
fn oklab_to_srgb(lab: [f64; 3]) -> [f32; 3] {
    let [light, a, b] = lab;
    let l = (light + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (light - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (light - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ]
    .map(|channel| encode_srgb(channel).clamp(0.0, 1.0) as f32)
}
pub(crate) fn srgb_color(rgb: [f32; 3]) -> ColorValue {
    ColorValue::Process(ProcessColor {
        value: ProcessColorValue::Rgb(Rgba {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            alpha: 1.0,
        }),
        space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn perceptual_conversion_has_known_reference_and_roundtrips() {
        let red = srgb_to_oklab([1.0, 0.0, 0.0]);
        assert!((red[0] - 0.62795536).abs() < 1e-7);
        let rgb = oklab_to_srgb(red);
        assert!((rgb[0] - 1.0).abs() < 1e-5);
        assert!(rgb[1] < 1e-5);
    }
    #[test]
    fn deterministic_weighted_quantization_and_transparency() {
        let image = RgbaSnapshot::new(
            4,
            1,
            vec![255, 0, 0, 255, 255, 0, 0, 128, 0, 0, 255, 255, 0, 255, 0, 0],
        )
        .unwrap();
        let source = image.clone();
        let token = CancelToken::new();
        let first = ColorQuantizer
            .quantize(
                &image,
                2,
                PaletteAlphaPolicy::WeightByAlpha,
                &AnalysisLimits::default(),
                &token,
            )
            .unwrap();
        let second = ColorQuantizer
            .quantize(
                &image,
                2,
                PaletteAlphaPolicy::WeightByAlpha,
                &AnalysisLimits::default(),
                &token,
            )
            .unwrap();
        assert_eq!(first.palette, second.palette);
        assert_eq!(first.assignments, second.assignments);
        assert_eq!(first.assignments[3], None);
        assert_eq!(image, source);
        assert_eq!(first.palette.len(), 2);
        assert!((first.palette[0].population - 1.502 / 2.502).abs() < 0.001);
        token.cancel();
        assert_eq!(
            ColorQuantizer
                .quantize(
                    &image,
                    2,
                    PaletteAlphaPolicy::WeightByAlpha,
                    &AnalysisLimits::default(),
                    &token
                )
                .unwrap_err(),
            AnalysisError::Cancelled
        );
    }
    #[test]
    fn guards_run_before_allocation_and_min_population_is_explicit() {
        let image = RgbaSnapshot::new(2, 1, vec![0, 0, 0, 255, 255, 255, 255, 255]).unwrap();
        let limits = AnalysisLimits {
            max_temporary_bytes: 1,
            ..AnalysisLimits::default()
        };
        assert!(matches!(
            ColorQuantizer.quantize(
                &image,
                2,
                PaletteAlphaPolicy::IgnoreAlpha,
                &limits,
                &CancelToken::new()
            ),
            Err(AnalysisError::LimitExceeded(_))
        ));
        let spec = PaletteExtractionSpec {
            colors: 2,
            min_population: 0.6,
            ..Default::default()
        };
        assert!(extract_palette(
            &image,
            &spec,
            &AnalysisLimits::default(),
            &CancelToken::new()
        )
        .unwrap()
        .is_empty());
    }
}
