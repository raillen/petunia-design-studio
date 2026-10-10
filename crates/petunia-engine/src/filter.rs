//! ROI filters over working raster surfaces.
//!
//! CPU filters obey the same edge-mode and region contracts any
//! future GPU path must honor. Gaussian blur shares its kernel
//! semantics with selection feathering: one implementation, no
//! special-case twin.

use crate::error::{EngineError, Result};
use serde::{Deserialize, Serialize};

/// Integer pixel rectangle for regions of interest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RectI {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl RectI {
    /// Empty regions are rejected: filters never process nothing.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(EngineError::Execution(
                "filter region needs non-zero dimensions".to_string(),
            ));
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    /// Expand by `radius` pixels, saturating at zero.
    #[must_use]
    pub fn expanded(self, radius: u32) -> Self {
        Self {
            x: self.x.saturating_sub(radius),
            y: self.y.saturating_sub(radius),
            width: self
                .x
                .saturating_add(self.width)
                .saturating_add(radius)
                .saturating_sub(self.x.saturating_sub(radius)),
            height: self
                .y
                .saturating_add(self.height)
                .saturating_add(radius)
                .saturating_sub(self.y.saturating_sub(radius)),
        }
    }
}

/// Out-of-surface sampling behavior, declared per effect. Backends
/// never pick different defaults silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EdgeMode {
    #[default]
    Transparent,
    Clamp,
    Repeat,
    Mirror,
}

/// Working surface: straight-alpha f32 RGBA in rows.
#[derive(Debug, Clone, PartialEq)]
pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<f32>,
}

impl Surface {
    /// Allocate a transparent surface inside the memory guard.
    pub fn transparent(width: u32, height: u32, max_pixels: u64) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(EngineError::Execution(
                "surface needs non-zero dimensions".to_string(),
            ));
        }
        let count = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|count| count.checked_mul(4))
            .ok_or_else(|| EngineError::Execution("surface allocation overflow".into()))?;
        if count > max_pixels || usize::try_from(count).is_err() {
            return Err(EngineError::Execution(format!(
                "surface of {count} floats exceeds guard {max_pixels}"
            )));
        }
        Ok(Self {
            width,
            height,
            pixels: vec![0.0; count as usize],
        })
    }

    /// Read one channel with the declared edge behavior.
    #[must_use]
    pub fn sample(&self, x: i64, y: i64, channel: usize, edge: EdgeMode) -> f32 {
        if self.width == 0 || self.height == 0 || channel >= 4 {
            return 0.0;
        }
        let (width, height) = (i64::from(self.width), i64::from(self.height));
        let (sx, sy) = match edge {
            EdgeMode::Transparent => {
                if x < 0 || y < 0 || x >= width || y >= height {
                    return 0.0;
                }
                (x, y)
            }
            EdgeMode::Clamp => (x.clamp(0, width - 1), y.clamp(0, height - 1)),
            EdgeMode::Repeat => (x.rem_euclid(width), y.rem_euclid(height)),
            EdgeMode::Mirror => (mirror_index(x, width), mirror_index(y, height)),
        };
        self.pixels
            .get((sy as usize * self.width as usize + sx as usize) * 4 + channel)
            .copied()
            .unwrap_or(0.0)
    }

    /// Write one channel; out-of-range writes are dropped.
    pub fn set(&mut self, x: u32, y: u32, channel: usize, value: f32) {
        if x < self.width && y < self.height && channel < 4 {
            self.pixels[((y * self.width + x) * 4 + channel as u32) as usize] = value;
        }
    }
}

fn mirror_index(index: i64, extent: i64) -> i64 {
    if extent <= 1 {
        return 0;
    }
    let period = 2 * (extent - 1);
    let folded = index.rem_euclid(period);
    if folded < extent {
        folded
    } else {
        period - folded
    }
}

/// Filter execution context: edge behavior plus scratch budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterContext {
    pub edge: EdgeMode,
    pub max_pixels: u64,
}

/// One ROI-capable filter. `input_region` answers which input area an
/// output area needs; `evaluate` computes exactly that area.
pub trait ImageFilter {
    /// Input region required to produce `output`.
    fn input_region(&self, output: RectI) -> RectI;

    /// Evaluate over `output`, reading a superset from `input`.
    fn evaluate(&self, ctx: &FilterContext, input: &Surface, output: &mut Surface) -> Result<()>;

    /// Compute a compact output whose origin is `region.x/y` in the full input.
    fn evaluate_region(
        &self,
        ctx: &FilterContext,
        input: &Surface,
        region: RectI,
        output: &mut Surface,
    ) -> Result<()>;
}

/// Separable Gaussian blur with sigma-derived radius. Sigma zero is a
/// passthrough; the kernel integrates to one so energy preserves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaussianBlur {
    pub sigma_x: f32,
    pub sigma_y: f32,
}

impl GaussianBlur {
    /// Sigmas must be finite and non-negative.
    pub fn new(sigma_x: f32, sigma_y: f32) -> Result<Self> {
        for sigma in [sigma_x, sigma_y] {
            if !sigma.is_finite() || !(0.0..=1365.0).contains(&sigma) {
                return Err(EngineError::Execution(format!(
                    "invalid blur sigma rejected: {sigma}"
                )));
            }
        }
        Ok(Self { sigma_x, sigma_y })
    }

    fn radius(sigma: f32) -> u32 {
        (sigma * 3.0).ceil().max(0.0) as u32
    }

    fn kernel(sigma: f32, radius: u32) -> Vec<f32> {
        if radius == 0 {
            return vec![1.0];
        }
        let mut weights: Vec<f32> = (0..=radius)
            .map(|offset| (-0.5 * (f64::from(offset) / f64::from(sigma)).powi(2)).exp() as f32)
            .collect();
        let total = weights[0] + 2.0 * weights[1..].iter().sum::<f32>();
        for weight in weights.iter_mut() {
            *weight /= total;
        }
        weights
    }
}

impl ImageFilter for GaussianBlur {
    fn input_region(&self, output: RectI) -> RectI {
        output.expanded(Self::radius(self.sigma_x).max(Self::radius(self.sigma_y)))
    }

    fn evaluate(&self, ctx: &FilterContext, input: &Surface, output: &mut Surface) -> Result<()> {
        if input.width != output.width || input.height != output.height {
            return Err(EngineError::Execution(
                "filter input and output dimensions must match".to_string(),
            ));
        }
        self.evaluate_region(
            ctx,
            input,
            RectI::new(0, 0, input.width, input.height)?,
            output,
        )
    }
    fn evaluate_region(
        &self,
        ctx: &FilterContext,
        input: &Surface,
        region: RectI,
        output: &mut Surface,
    ) -> Result<()> {
        Self::new(self.sigma_x, self.sigma_y)?;
        if region.width != output.width
            || region.height != output.height
            || region.width == 0
            || region.height == 0
            || input.width == 0
            || input.height == 0
            || input.pixels.len() != input.width as usize * input.height as usize * 4
            || output.pixels.len() != output.width as usize * output.height as usize * 4
        {
            return Err(EngineError::Execution(
                "invalid ROI/surface dimensions".into(),
            ));
        }
        let radius_x = Self::radius(self.sigma_x);
        let radius_y = Self::radius(self.sigma_y);
        let kernel_x = Self::kernel(self.sigma_x, radius_x);
        let kernel_y = Self::kernel(self.sigma_y, radius_y);
        let scratch_height = region
            .height
            .checked_add(
                radius_y
                    .checked_mul(2)
                    .ok_or_else(|| EngineError::Execution("blur expansion overflow".into()))?,
            )
            .ok_or_else(|| EngineError::Execution("blur expansion overflow".into()))?;
        let mut scratch = Surface::transparent(region.width, scratch_height, ctx.max_pixels)?;
        let origin_y = i64::from(region.y) - i64::from(radius_y);
        for sy in 0..scratch_height {
            let y = origin_y + i64::from(sy);
            for sx in 0..region.width {
                let x = i64::from(region.x) + i64::from(sx);
                for channel in 0..4 {
                    let mut sum = 0.0;
                    for delta in -i64::from(radius_x)..=i64::from(radius_x) {
                        let weight = kernel_x[delta.unsigned_abs() as usize];
                        let alpha = input.sample(x + delta, y, 3, ctx.edge);
                        let value = input.sample(x + delta, y, channel, ctx.edge);
                        sum += if channel == 3 {
                            value * weight
                        } else {
                            value * alpha * weight
                        };
                    }
                    scratch.set(sx, sy, channel, sum);
                }
            }
        }
        // Keep the destination unchanged until all guards/allocations succeed.
        let mut evaluated = Surface::transparent(output.width, output.height, ctx.max_pixels)?;
        for y in 0..output.height {
            for x in 0..output.width {
                let mut pixel = [0.0; 4];
                for (channel, sum) in pixel.iter_mut().enumerate() {
                    for (offset, weight) in kernel_y.iter().enumerate() {
                        *sum += scratch.sample(
                            i64::from(x),
                            i64::from(y) + i64::from(radius_y) - offset as i64,
                            channel,
                            EdgeMode::Transparent,
                        ) * weight;
                        if offset > 0 {
                            *sum += scratch.sample(
                                i64::from(x),
                                i64::from(y) + i64::from(radius_y) + offset as i64,
                                channel,
                                EdgeMode::Transparent,
                            ) * weight;
                        }
                    }
                }
                for channel in 0..3 {
                    evaluated.set(
                        x,
                        y,
                        channel,
                        if pixel[3] > 0.0 {
                            pixel[channel] / pixel[3]
                        } else {
                            0.0
                        },
                    );
                }
                evaluated.set(x, y, 3, pixel[3]);
            }
        }
        *output = evaluated;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> FilterContext {
        FilterContext {
            edge: EdgeMode::Clamp,
            max_pixels: 1 << 26,
        }
    }

    fn impulse_at(size: u32, at: u32) -> Surface {
        let mut surface = Surface::transparent(size, size, 1 << 26).expect("fits guard");
        surface.set(at, at, 0, 1.0);
        surface.set(at, at, 3, 1.0);
        surface
    }

    fn impulse() -> Surface {
        impulse_at(9, 4)
    }

    fn total(surface: &Surface, channel: usize) -> f32 {
        (0..surface.height)
            .flat_map(|y| (0..surface.width).map(move |x| (x, y)))
            .map(|(x, y)| surface.pixels[((y * surface.width + x) * 4 + channel as u32) as usize])
            .sum()
    }

    #[test]
    fn blur_preserves_energy_and_spreads() {
        // Impulse far from borders: every kernel tap lands inside.
        let input = impulse_at(25, 12);
        let mut output = Surface::transparent(25, 25, 1 << 26).expect("fits guard");
        GaussianBlur::new(1.5, 1.5)
            .expect("valid")
            .evaluate(&ctx(), &input, &mut output)
            .expect("evaluates");
        // Unit kernel: energy preserved within float error.
        assert!(
            (total(&output, 3) - 1.0).abs() < 1e-4,
            "total {}",
            total(&output, 3)
        );
        // Peak spread below the impulse, neighbors above zero.
        assert!(output.pixels[((12 * 25 + 12) * 4 + 3) as usize] < 1.0);
        assert!(output.pixels[((12 * 25 + 13) * 4 + 3) as usize] > 0.0);
        // At the border, out-of-frame energy drops gracefully.
        let mut edge = Surface::transparent(9, 9, 1 << 26).expect("fits guard");
        GaussianBlur::new(1.5, 1.5)
            .expect("valid")
            .evaluate(&ctx(), &impulse(), &mut edge)
            .expect("evaluates");
        let edge_total = total(&edge, 3);
        assert!(edge_total > 0.9 && edge_total <= 1.0, "total {edge_total}");
    }

    #[test]
    fn zero_sigma_passes_through() {
        let input = impulse();
        let mut output = Surface::transparent(9, 9, 1 << 26).expect("fits guard");
        GaussianBlur::new(0.0, 0.0)
            .expect("valid")
            .evaluate(&ctx(), &input, &mut output)
            .expect("evaluates");
        assert_eq!(input.pixels, output.pixels);
    }

    #[test]
    fn input_region_expands_by_kernel_radius() {
        let blur = GaussianBlur::new(2.0, 2.0).expect("valid");
        let output = RectI::new(10, 10, 20, 20).expect("valid");
        let input = blur.input_region(output);
        assert_eq!(input, RectI::new(4, 4, 32, 32).expect("valid"));
    }

    #[test]
    fn guards_reject_absurd_allocations() {
        assert!(Surface::transparent(0, 8, 1 << 26).is_err());
        assert!(Surface::transparent(1 << 20, 1 << 20, 1 << 26).is_err());
        assert!(RectI::new(0, 0, 0, 8).is_err());
    }
}
