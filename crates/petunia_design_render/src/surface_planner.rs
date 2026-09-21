//! Intermediate surface allocation planner and memory telemetry.
//!
//! Tracks offscreen surfaces required by isolated groups, effects (blurs/shadows
//! with padding), and complex blending. Prevents unbounded allocations and recycles
//! compatible buffers.

use petunia_design_geometry::GRect;
use serde::{Deserialize, Serialize};

/// Supported pixel format for intermediate surfaces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceFormat {
    /// Standard 8-bit per channel RGBA (4 bytes/pixel).
    #[default]
    Rgba8,
    /// High-precision 16-bit per channel RGBA (8 bytes/pixel).
    Rgba16,
    /// 32-bit floating point per channel RGBA (16 bytes/pixel).
    F32,
}

impl SurfaceFormat {
    /// Returns the number of bytes required per pixel.
    #[must_use]
    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Rgba8 => 4,
            Self::Rgba16 => 8,
            Self::F32 => 16,
        }
    }
}

/// A planned intermediate offscreen surface reservation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedSurface {
    /// Unique identifier for this surface allocation.
    pub id: usize,
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Pixel format.
    pub format: SurfaceFormat,
    /// Effect padding included in the surface dimensions.
    pub padding: u32,
    /// Total memory footprint in bytes.
    pub memory_bytes: usize,
}

/// Error condition when intermediate surface planning exceeds memory budget.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SurfaceAllocationError {
    /// Requested dimensions exceed maximum texture limits.
    #[error("Surface dimensions {width}x{height} exceed maximum allowed limits")]
    DimensionsExceeded { width: u32, height: u32 },
    /// Allocation would exceed total memory budget.
    #[error("Surface allocation of {requested_bytes} bytes exceeds remaining budget (current={current_bytes}, budget={budget_bytes})")]
    BudgetExceeded {
        requested_bytes: usize,
        current_bytes: usize,
        budget_bytes: usize,
    },
}

/// Intermediate surface planner tracking active allocations, pool reuse, and memory telemetry.
#[derive(Clone, Debug)]
pub struct IntermediateSurfacePlanner {
    next_id: usize,
    max_dimension: u32,
    memory_budget_bytes: usize,
    current_allocated_bytes: usize,
    peak_allocated_bytes: usize,
    total_allocations: usize,
    recycled_allocations: usize,
    free_pool: Vec<PlannedSurface>,
}

impl Default for IntermediateSurfacePlanner {
    fn default() -> Self {
        Self {
            next_id: 1,
            max_dimension: 16384,
            memory_budget_bytes: 512 * 1024 * 1024, // 512 MB safety budget
            current_allocated_bytes: 0,
            peak_allocated_bytes: 0,
            total_allocations: 0,
            recycled_allocations: 0,
            free_pool: Vec::new(),
        }
    }
}

impl IntermediateSurfacePlanner {
    /// Creates a planner with custom maximum dimension and memory budget.
    #[must_use]
    pub fn new(max_dimension: u32, memory_budget_bytes: usize) -> Self {
        Self {
            max_dimension,
            memory_budget_bytes,
            ..Default::default()
        }
    }

    /// Plans and reserves an intermediate surface for a given bounding box, effect padding, and format.
    pub fn plan_surface(
        &mut self,
        bounds: GRect,
        effect_padding: f32,
        format: SurfaceFormat,
    ) -> Result<PlannedSurface, SurfaceAllocationError> {
        let padding_u32 = effect_padding.max(0.0).ceil() as u32;
        let padded_w = (bounds.width().max(1.0).ceil() as u32).saturating_add(padding_u32 * 2);
        let padded_h = (bounds.height().max(1.0).ceil() as u32).saturating_add(padding_u32 * 2);

        if padded_w > self.max_dimension || padded_h > self.max_dimension {
            return Err(SurfaceAllocationError::DimensionsExceeded {
                width: padded_w,
                height: padded_h,
            });
        }

        let memory_bytes = (padded_w as usize)
            .saturating_mul(padded_h as usize)
            .saturating_mul(format.bytes_per_pixel());

        // Check if a reusable surface in the free pool matches dimensions and format
        if let Some(pos) = self.free_pool.iter().position(|s| {
            s.width >= padded_w
                && s.height >= padded_h
                && s.format == format
                && s.memory_bytes <= memory_bytes.saturating_mul(2) // Do not waste huge surfaces on tiny requests
        }) {
            let mut recycled = self.free_pool.swap_remove(pos);
            recycled.padding = padding_u32;
            self.recycled_allocations += 1;
            return Ok(recycled);
        }

        if self.current_allocated_bytes.saturating_add(memory_bytes) > self.memory_budget_bytes {
            return Err(SurfaceAllocationError::BudgetExceeded {
                requested_bytes: memory_bytes,
                current_bytes: self.current_allocated_bytes,
                budget_bytes: self.memory_budget_bytes,
            });
        }

        let surface = PlannedSurface {
            id: self.next_id,
            width: padded_w,
            height: padded_h,
            format,
            padding: padding_u32,
            memory_bytes,
        };
        self.next_id += 1;
        self.total_allocations += 1;
        self.current_allocated_bytes += memory_bytes;
        self.peak_allocated_bytes = self.peak_allocated_bytes.max(self.current_allocated_bytes);

        Ok(surface)
    }

    /// Releases a surface back to the pool for reuse.
    pub fn release_surface(&mut self, surface: PlannedSurface) {
        self.free_pool.push(surface);
    }

    /// Total memory currently allocated in bytes.
    #[must_use]
    pub fn current_allocated_bytes(&self) -> usize {
        self.current_allocated_bytes
    }

    /// Peak memory allocated over the lifetime of this planner.
    #[must_use]
    pub fn peak_allocated_bytes(&self) -> usize {
        self.peak_allocated_bytes
    }

    /// Total number of surfaces created.
    #[must_use]
    pub fn total_allocations(&self) -> usize {
        self.total_allocations
    }

    /// Total number of times a surface was recycled from the free pool.
    #[must_use]
    pub fn recycled_allocations(&self) -> usize {
        self.recycled_allocations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planning_surface_computes_correct_dimensions_and_bytes() {
        let mut planner = IntermediateSurfacePlanner::default();
        let bounds = GRect::new(0.0, 0.0, 100.0, 50.0);
        let surface = planner
            .plan_surface(bounds, 10.0, SurfaceFormat::Rgba8)
            .unwrap();

        // 100 + 2*10 = 120, 50 + 2*10 = 70
        assert_eq!(surface.width, 120);
        assert_eq!(surface.height, 70);
        assert_eq!(surface.format, SurfaceFormat::Rgba8);
        assert_eq!(surface.memory_bytes, 120 * 70 * 4);
        assert_eq!(planner.total_allocations(), 1);
        assert_eq!(planner.recycled_allocations(), 0);
    }

    #[test]
    fn releasing_surface_allows_recycling() {
        let mut planner = IntermediateSurfacePlanner::default();
        let bounds = GRect::new(0.0, 0.0, 100.0, 100.0);
        let surface1 = planner
            .plan_surface(bounds, 0.0, SurfaceFormat::Rgba8)
            .unwrap();
        let id1 = surface1.id;
        planner.release_surface(surface1);

        let surface2 = planner
            .plan_surface(bounds, 0.0, SurfaceFormat::Rgba8)
            .unwrap();
        assert_eq!(surface2.id, id1);
        assert_eq!(planner.recycled_allocations(), 1);
    }

    #[test]
    fn exceeding_budget_returns_error() {
        let mut planner = IntermediateSurfacePlanner::new(1000, 1024); // 1 KB budget
        let bounds = GRect::new(0.0, 0.0, 100.0, 100.0);
        // 100*100*4 = 40,000 bytes > 1024
        let err = planner
            .plan_surface(bounds, 0.0, SurfaceFormat::Rgba8)
            .unwrap_err();
        assert!(matches!(err, SurfaceAllocationError::BudgetExceeded { .. }));
    }
}
