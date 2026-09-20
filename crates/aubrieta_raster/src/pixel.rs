//! Pixel format contracts, bit depth and alpha mode (09.6).

use serde::{Deserialize, Serialize};

/// Bit depth per channel. 8-bit and 16-bit integer are V1 requirements (09.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BitDepth {
    /// 8 bits per channel (standard SDR).
    Eight,
    /// 16 bits per channel (deep color, high dynamic precision).
    Sixteen,
}

impl BitDepth {
    #[must_use]
    pub const fn bytes_per_channel(self) -> usize {
        match self {
            Self::Eight => 1,
            Self::Sixteen => 2,
        }
    }
}

/// Alpha representation contract at raster boundaries (09.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlphaMode {
    Straight,
    Premultiplied,
}

/// Pixel format description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba8,
    Rgba16,
    Gray8,
    Gray16,
}

impl PixelFormat {
    #[must_use]
    pub const fn bit_depth(self) -> BitDepth {
        match self {
            Self::Rgba8 | Self::Gray8 => BitDepth::Eight,
            Self::Rgba16 | Self::Gray16 => BitDepth::Sixteen,
        }
    }

    #[must_use]
    pub const fn channels(self) -> usize {
        match self {
            Self::Rgba8 | Self::Rgba16 => 4,
            Self::Gray8 | Self::Gray16 => 1,
        }
    }

    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        self.channels() * self.bit_depth().bytes_per_channel()
    }
}
