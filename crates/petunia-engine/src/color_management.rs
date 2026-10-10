//! Little CMS 2 adapter. Profile bytes are supplied by resource resolution;
//! external locations never authorize filesystem reads. Backend handles remain
//! private. Transforms are cached by profile content, formats, intent and BPC.

use lcms2::{ColorSpaceSignature, Flags, Intent, PixelFormat, Profile, ToneCurve, Transform};
use petunia_core::color::{
    BuiltinColorSpace, ColorSpaceRef, ProcessColor, ProcessColorValue, RenderingIntent, Rgba,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ColorError {
    #[error("profile unavailable: {0:?}")]
    ProfileUnavailable(ColorSpaceRef),
    #[error("invalid ICC profile: {0}")]
    InvalidProfile(String),
    #[error("unsupported color format or profile combination: {0}")]
    UnsupportedFormat(String),
    #[error("color transform failed: {0}")]
    Transform(String),
    #[error("invalid color: {0}")]
    InvalidColor(String),
    #[error("unknown color transform")]
    UnknownTransform,
}

type ColorResult<T> = std::result::Result<T, ColorError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorFormat {
    Rgb,
    Cmyk,
    Gray,
    Lab,
}
impl ColorFormat {
    fn signature(self) -> ColorSpaceSignature {
        match self {
            Self::Rgb => ColorSpaceSignature::RgbData,
            Self::Cmyk => ColorSpaceSignature::CmykData,
            Self::Gray => ColorSpaceSignature::GrayData,
            Self::Lab => ColorSpaceSignature::LabData,
        }
    }
    fn pixel_format(self) -> PixelFormat {
        match self {
            Self::Rgb => PixelFormat::RGB_FLT,
            Self::Cmyk => PixelFormat::CMYK_FLT,
            Self::Gray => PixelFormat::GRAY_FLT,
            Self::Lab => PixelFormat::Lab_FLT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorTransformSpec {
    pub source: ColorSpaceRef,
    pub destination: ColorSpaceRef,
    pub source_format: ColorFormat,
    pub destination_format: ColorFormat,
    pub intent: RenderingIntent,
    pub black_point_compensation: bool,
}

/// Opaque, engine-local derived handle. It never enters authorial storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorTransformId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TransformKey {
    source_ref: ColorSpaceRef,
    destination_ref: ColorSpaceRef,
    source: [u8; 32],
    destination: [u8; 32],
    input: ColorFormat,
    output: ColorFormat,
    intent: RenderingIntent,
    bpc: bool,
}

struct ProfileRecord {
    profile: Profile,
    hash: [u8; 32],
}
enum Mapping {
    Rgb(Transform<[f32; 3], [f32; 3]>),
    CmykToRgb(Transform<[f32; 4], [f32; 3]>),
    RgbToCmyk(Transform<[f32; 3], [f32; 4]>),
    GrayToRgb(Transform<[f32; 1], [f32; 3]>),
    LabToRgb(Transform<[f32; 3], [f32; 3]>),
}
struct CachedTransform {
    spec: ColorTransformSpec,
    mapping: Mapping,
}

/// Host-owned CMM. A document can supply embedded, resolved external and named
/// profiles. No backend type appears in methods or DTOs. Alpha stays straight.
#[derive(Default)]
pub struct ColorEngine {
    profiles: HashMap<ColorSpaceRef, ProfileRecord>,
    transforms: Vec<CachedTransform>,
    cache: HashMap<TransformKey, ColorTransformId>,
}

impl ColorEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_profile(&mut self, reference: ColorSpaceRef, bytes: &[u8]) -> ColorResult<()> {
        if matches!(reference, ColorSpaceRef::Builtin(_)) {
            return Err(ColorError::InvalidProfile(
                "builtin profiles cannot be overridden".into(),
            ));
        }
        // Avoid oversized allocations in native profile parsing.
        if bytes.len() < 128 || bytes.len() > 16 << 20 {
            return Err(ColorError::InvalidProfile(
                "ICC size outside 128 bytes..16 MiB".into(),
            ));
        }
        let profile =
            Profile::new_icc(bytes).map_err(|e| ColorError::InvalidProfile(e.to_string()))?;
        self.profiles.insert(
            reference,
            ProfileRecord {
                profile,
                hash: *blake3::hash(bytes).as_bytes(),
            },
        );
        Ok(())
    }

    fn ensure_profile(&mut self, reference: &ColorSpaceRef) -> ColorResult<()> {
        if self.profiles.contains_key(reference) {
            return Ok(());
        }
        let ColorSpaceRef::Builtin(builtin) = reference else {
            return Err(ColorError::ProfileUnavailable(reference.clone()));
        };
        let profile = match builtin {
            BuiltinColorSpace::Srgb => Profile::new_srgb(),
            BuiltinColorSpace::LinearSrgb | BuiltinColorSpace::DisplayP3 => {
                let p3 = matches!(builtin, BuiltinColorSpace::DisplayP3);
                let xy = |x, y| lcms2::CIExyY { x, y, Y: 1.0 };
                let primaries = lcms2::CIExyYTRIPLE {
                    Red: if p3 { xy(0.68, 0.32) } else { xy(0.64, 0.33) },
                    Green: if p3 { xy(0.265, 0.69) } else { xy(0.30, 0.60) },
                    Blue: xy(0.15, 0.06),
                };
                let curve = if p3 {
                    // IEC 61966-2-1 transfer used by Display P3: type 4 =
                    // (a*x+b)^g above d, c*x otherwise (ICC parametric type 3).
                    ToneCurve::new_parametric(
                        4,
                        &[2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045],
                    )
                    .map_err(|e| ColorError::InvalidProfile(e.to_string()))?
                } else {
                    ToneCurve::new(1.0)
                };
                Profile::new_rgb(&xy(0.3127, 0.3290), &primaries, &[&curve, &curve, &curve])
                    .map_err(|e| ColorError::InvalidProfile(e.to_string()))?
            }
        };
        let bytes = profile
            .icc()
            .map_err(|e| ColorError::InvalidProfile(e.to_string()))?;
        self.profiles.insert(
            reference.clone(),
            ProfileRecord {
                profile,
                hash: *blake3::hash(&bytes).as_bytes(),
            },
        );
        Ok(())
    }

    pub fn build_transform(&mut self, spec: ColorTransformSpec) -> ColorResult<ColorTransformId> {
        self.ensure_profile(&spec.source)?;
        self.ensure_profile(&spec.destination)?;
        let source = &self.profiles[&spec.source];
        let destination = &self.profiles[&spec.destination];
        if source.profile.color_space() != spec.source_format.signature()
            || destination.profile.color_space() != spec.destination_format.signature()
        {
            return Err(ColorError::UnsupportedFormat(
                "channel format does not match profile".into(),
            ));
        }
        let key = TransformKey {
            source_ref: spec.source.clone(),
            destination_ref: spec.destination.clone(),
            source: source.hash,
            destination: destination.hash,
            input: spec.source_format,
            output: spec.destination_format,
            intent: spec.intent,
            bpc: spec.black_point_compensation,
        };
        if let Some(id) = self.cache.get(&key) {
            return Ok(*id);
        }
        let intent = match spec.intent {
            RenderingIntent::Perceptual => Intent::Perceptual,
            RenderingIntent::RelativeColorimetric => Intent::RelativeColorimetric,
            RenderingIntent::AbsoluteColorimetric => Intent::AbsoluteColorimetric,
            RenderingIntent::Saturation => Intent::Saturation,
        };
        let flags = if spec.black_point_compensation {
            Flags::BLACKPOINT_COMPENSATION
        } else {
            Flags::default()
        };
        let input = spec.source_format.pixel_format();
        let output = spec.destination_format.pixel_format();
        let mapping = match (spec.source_format, spec.destination_format) {
            (ColorFormat::Rgb, ColorFormat::Rgb) => Mapping::Rgb(
                Transform::new_flags(
                    &source.profile,
                    input,
                    &destination.profile,
                    output,
                    intent,
                    flags,
                )
                .map_err(backend_error)?,
            ),
            (ColorFormat::Cmyk, ColorFormat::Rgb) => Mapping::CmykToRgb(
                Transform::new_flags(
                    &source.profile,
                    input,
                    &destination.profile,
                    output,
                    intent,
                    flags,
                )
                .map_err(backend_error)?,
            ),
            (ColorFormat::Rgb, ColorFormat::Cmyk) => Mapping::RgbToCmyk(
                Transform::new_flags(
                    &source.profile,
                    input,
                    &destination.profile,
                    output,
                    intent,
                    flags,
                )
                .map_err(backend_error)?,
            ),
            (ColorFormat::Gray, ColorFormat::Rgb) => Mapping::GrayToRgb(
                Transform::new_flags(
                    &source.profile,
                    input,
                    &destination.profile,
                    output,
                    intent,
                    flags,
                )
                .map_err(backend_error)?,
            ),
            (ColorFormat::Lab, ColorFormat::Rgb) => Mapping::LabToRgb(
                Transform::new_flags(
                    &source.profile,
                    input,
                    &destination.profile,
                    output,
                    intent,
                    flags,
                )
                .map_err(backend_error)?,
            ),
            _ => {
                return Err(ColorError::UnsupportedFormat(
                    "conversion pair unavailable".into(),
                ))
            }
        };
        let id = ColorTransformId(self.transforms.len());
        self.transforms.push(CachedTransform { spec, mapping });
        self.cache.insert(key, id);
        Ok(id)
    }

    pub fn convert_rgb_pixels(
        &self,
        id: ColorTransformId,
        src: &[[f32; 3]],
        dst: &mut [[f32; 3]],
    ) -> ColorResult<()> {
        if src.len() != dst.len()
            || src.len() > u32::MAX as usize
            || src.iter().flatten().any(|c| !c.is_finite())
        {
            return Err(ColorError::InvalidColor(
                "pixel length mismatch, overflow or nonfinite channel".into(),
            ));
        }
        let transform = self
            .transforms
            .get(id.0)
            .ok_or(ColorError::UnknownTransform)?;
        let Mapping::Rgb(mapping) = &transform.mapping else {
            return Err(ColorError::UnsupportedFormat(
                "requires RGB pixel transform".into(),
            ));
        };
        mapping.transform_pixels(src, dst);
        Ok(())
    }

    pub fn convert_color(
        &self,
        id: ColorTransformId,
        color: &ProcessColor,
    ) -> ColorResult<ProcessColor> {
        color
            .validate()
            .map_err(|e| ColorError::InvalidColor(e.to_string()))?;
        let transform = self
            .transforms
            .get(id.0)
            .ok_or(ColorError::UnknownTransform)?;
        if color.space != transform.spec.source {
            return Err(ColorError::InvalidColor("source profile mismatch".into()));
        }
        let rgb = |out: [f32; 3], alpha| {
            ProcessColorValue::Rgb(Rgba {
                r: out[0],
                g: out[1],
                b: out[2],
                alpha,
            })
        };
        let value = match (&transform.mapping, color.value) {
            (Mapping::Rgb(t), ProcessColorValue::Rgb(c)) => {
                let mut out = [[0.0; 3]];
                t.transform_pixels(&[[c.r, c.g, c.b]], &mut out);
                rgb(out[0], c.alpha)
            }
            (Mapping::CmykToRgb(t), ProcessColorValue::Cmyk(c)) => {
                let mut out = [[0.0; 3]];
                t.transform_pixels(
                    &[[c.c * 100.0, c.m * 100.0, c.y * 100.0, c.k * 100.0]],
                    &mut out,
                );
                rgb(out[0], c.alpha)
            }
            (Mapping::RgbToCmyk(t), ProcessColorValue::Rgb(c)) => {
                let mut out = [[0.0; 4]];
                t.transform_pixels(&[[c.r, c.g, c.b]], &mut out);
                ProcessColorValue::Cmyk(petunia_core::color::Cmyka {
                    c: out[0][0] / 100.0,
                    m: out[0][1] / 100.0,
                    y: out[0][2] / 100.0,
                    k: out[0][3] / 100.0,
                    alpha: c.alpha,
                })
            }
            (Mapping::GrayToRgb(t), ProcessColorValue::Gray(c)) => {
                let mut out = [[0.0; 3]];
                t.transform_pixels(&[[c.gray]], &mut out);
                rgb(out[0], c.alpha)
            }
            (Mapping::LabToRgb(t), ProcessColorValue::Lab(c)) => {
                let mut out = [[0.0; 3]];
                t.transform_pixels(&[[c.l, c.a_axis, c.b_axis]], &mut out);
                rgb(out[0], c.alpha)
            }
            _ => {
                return Err(ColorError::UnsupportedFormat(
                    "scalar channels do not match transform".into(),
                ))
            }
        };
        Ok(ProcessColor {
            space: transform.spec.destination.clone(),
            value,
        })
    }

    pub fn to_srgb(&mut self, color: &ProcessColor) -> ColorResult<Rgba> {
        self.convert_rgb(color, BuiltinColorSpace::Srgb)
    }
    pub fn to_linear_srgb(&mut self, color: &ProcessColor) -> ColorResult<Rgba> {
        self.convert_rgb(color, BuiltinColorSpace::LinearSrgb)
    }
    fn convert_rgb(&mut self, color: &ProcessColor, space: BuiltinColorSpace) -> ColorResult<Rgba> {
        let source_format = match color.value {
            ProcessColorValue::Rgb(_) => ColorFormat::Rgb,
            ProcessColorValue::Cmyk(_) => ColorFormat::Cmyk,
            ProcessColorValue::Lab(_) => ColorFormat::Lab,
            ProcessColorValue::Gray(_) => ColorFormat::Gray,
        };
        let id = self.build_transform(ColorTransformSpec {
            source: color.space.clone(),
            destination: ColorSpaceRef::Builtin(space),
            source_format,
            destination_format: ColorFormat::Rgb,
            intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: true,
        })?;
        self.convert_color(id, color)?
            .as_rgb()
            .ok_or_else(|| ColorError::UnsupportedFormat("output is not RGB".into()))
    }
}
fn backend_error(error: lcms2::Error) -> ColorError {
    ColorError::Transform(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn color(space: ColorSpaceRef, r: f32, g: f32, b: f32) -> ProcessColor {
        ProcessColor {
            space,
            value: ProcessColorValue::Rgb(Rgba {
                r,
                g,
                b,
                alpha: 0.7,
            }),
        }
    }
    fn spec(source: ColorSpaceRef, destination: ColorSpaceRef) -> ColorTransformSpec {
        ColorTransformSpec {
            source,
            destination,
            source_format: ColorFormat::Rgb,
            destination_format: ColorFormat::Rgb,
            intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: true,
        }
    }
    #[test]
    fn linear_and_encoded_rgb_use_lcms_preserve_alpha_and_pixels_agree() {
        let mut engine = ColorEngine::new();
        let src = color(
            ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            0.5,
            0.25,
            1.0,
        );
        let linear = engine.to_linear_srgb(&src).expect("linear conversion");
        assert!((linear.r - 0.214041).abs() < 0.0001, "{linear:?}");
        assert_eq!(linear.alpha, 0.7);
        let roundtrip = engine
            .to_srgb(&ProcessColor {
                space: ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb),
                value: ProcessColorValue::Rgb(linear),
            })
            .expect("roundtrip");
        assert!((roundtrip.r - 0.5).abs() < 0.0001);
        let id = engine
            .build_transform(spec(
                src.space.clone(),
                ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb),
            ))
            .expect("build");
        let mut out = [[0.0; 3]];
        engine
            .convert_rgb_pixels(id, &[[0.5, 0.25, 1.0]], &mut out)
            .expect("pixels");
        assert_eq!(out[0][0], linear.r);
    }
    #[test]
    fn real_icc_bytes_are_parsed_and_change_numeric_mapping() {
        let xy = |x, y| lcms2::CIExyY { x, y, Y: 1.0 };
        let curve = ToneCurve::new(1.8);
        let custom = Profile::new_rgb(
            &xy(0.3127, 0.3290),
            &lcms2::CIExyYTRIPLE {
                Red: xy(0.64, 0.33),
                Green: xy(0.30, 0.60),
                Blue: xy(0.15, 0.06),
            },
            &[&curve, &curve, &curve],
        )
        .expect("custom profile")
        .icc()
        .expect("ICC bytes");
        let reference = ColorSpaceRef::EmbeddedIcc(petunia_core::ResourceId::new_v4());
        let mut engine = ColorEngine::new();
        engine
            .register_profile(reference.clone(), &custom)
            .expect("register");
        let converted = engine
            .to_srgb(&color(reference, 0.5, 0.5, 0.5))
            .expect("ICC transform");
        assert!((converted.r - 0.57231).abs() < 0.003, "{converted:?}");
    }
    #[test]
    fn display_p3_is_not_srgb_alias() {
        let mut engine = ColorEngine::new();
        let mapped = engine
            .to_linear_srgb(&color(
                ColorSpaceRef::Builtin(BuiltinColorSpace::DisplayP3),
                1.0,
                0.0,
                0.0,
            ))
            .expect("P3");
        assert!(mapped.r > 1.15, "P3 red primary outside sRGB: {mapped:?}");
        assert!(mapped.g < 0.0, "P3 red has negative green: {mapped:?}");
    }
    #[test]
    fn missing_cmyk_profile_invalid_icc_and_wrong_formats_are_errors() {
        let mut engine = ColorEngine::new();
        let reference = ColorSpaceRef::ExternalIcc {
            location: "printer.icc".into(),
        };
        let cmyk = ProcessColor {
            space: reference.clone(),
            value: ProcessColorValue::Cmyk(petunia_core::color::Cmyka {
                c: 0.0,
                m: 1.0,
                y: 1.0,
                k: 0.0,
                alpha: 1.0,
            }),
        };
        assert_eq!(
            engine.to_srgb(&cmyk),
            Err(ColorError::ProfileUnavailable(reference.clone()))
        );
        assert!(engine.register_profile(reference, &[0; 128]).is_err());
        let invalid = ProcessColor {
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            ..cmyk
        };
        assert!(matches!(
            engine.to_srgb(&invalid),
            Err(ColorError::UnsupportedFormat(_))
        ));
    }
    #[test]
    fn cache_covers_profile_content_intent_bpc_and_semantic_reference() {
        let mut engine = ColorEngine::new();
        let srgb = ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb);
        let linear = ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb);
        let a = spec(srgb.clone(), linear.clone());
        let id = engine.build_transform(a.clone()).expect("transform");
        assert_eq!(id, engine.build_transform(a.clone()).expect("cache"));
        let mut b = a.clone();
        b.black_point_compensation = false;
        assert_ne!(id, engine.build_transform(b).expect("BPC key"));
        let mut b = a;
        b.intent = RenderingIntent::AbsoluteColorimetric;
        assert_ne!(id, engine.build_transform(b).expect("intent key"));
        let reference = ColorSpaceRef::DocumentSpace {
            name: "workspace".into(),
        };
        engine
            .register_profile(
                reference.clone(),
                &Profile::new_srgb().icc().expect("bytes"),
            )
            .expect("register");
        let first = engine
            .build_transform(spec(reference.clone(), linear.clone()))
            .expect("first");
        engine
            .register_profile(
                reference.clone(),
                &Profile::new_gray(
                    &lcms2::CIExyY {
                        x: 0.3127,
                        y: 0.3290,
                        Y: 1.0,
                    },
                    &ToneCurve::new(2.2),
                )
                .expect("gray")
                .icc()
                .expect("bytes"),
            )
            .expect("replace");
        assert!(engine.build_transform(spec(reference, linear)).is_err());
        assert!(engine
            .convert_color(first, &color(srgb, 0.5, 0.5, 0.5))
            .is_err());
    }
}
