//! Checked ICC v2/v4 profiles and actual LittleCMS transforms. No profile name
//! is interpreted as a profile file, and ink channels never pass through RGB
//! unless the caller explicitly requests that conversion.
use lcms2::{ColorSpaceSignature, Flags, Intent, PixelFormat, Profile, Transform};
use petunia_design_foundation::PetuniaError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, OnceLock};

const MAX_PROFILE_BYTES: usize = 4 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IccProfileId([u8; 32]);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IccColorSpace {
    Rgb,
    Cmyk,
    Gray,
    Lab,
    Xyz,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "ProfileWire", into = "ProfileWire")]
pub struct IccProfile {
    id: IccProfileId,
    name: String,
    bytes: Arc<Vec<u8>>,
    space: IccColorSpace,
}
#[derive(Serialize, Deserialize)]
struct ProfileWire {
    name: String,
    bytes: Arc<Vec<u8>>,
}
impl TryFrom<ProfileWire> for IccProfile {
    type Error = PetuniaError;
    fn try_from(wire: ProfileWire) -> Result<Self, Self::Error> {
        Self::new(wire.name, wire.bytes)
    }
}
impl From<IccProfile> for ProfileWire {
    fn from(profile: IccProfile) -> Self {
        Self {
            name: profile.name,
            bytes: profile.bytes,
        }
    }
}
fn invalid(reason: impl Into<String>) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
impl PartialEq for IccProfile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}
impl IccProfile {
    pub fn new(name: String, bytes: Arc<Vec<u8>>) -> Result<Self, PetuniaError> {
        if name.is_empty() || name.len() > 512 || name.chars().any(char::is_control) {
            return Err(invalid("invalid ICC profile name"));
        }
        validate_header(&bytes)?;
        let parsed = Profile::new_icc(&bytes).map_err(|e| invalid(format!("ICC profile: {e}")))?;
        let space = match parsed.color_space() {
            ColorSpaceSignature::RgbData => IccColorSpace::Rgb,
            ColorSpaceSignature::CmykData => IccColorSpace::Cmyk,
            ColorSpaceSignature::GrayData => IccColorSpace::Gray,
            ColorSpaceSignature::LabData => IccColorSpace::Lab,
            ColorSpaceSignature::XYZData => IccColorSpace::Xyz,
            _ => {
                return Err(PetuniaError::capability_unavailable(
                    "ICC profile channel space",
                ))
            }
        };
        Ok(Self {
            id: IccProfileId(Sha256::digest(bytes.as_slice()).into()),
            name,
            bytes,
            space,
        })
    }
    pub fn srgb() -> Result<Self, PetuniaError> {
        static SRGB: OnceLock<Result<IccProfile, String>> = OnceLock::new();
        SRGB.get_or_init(|| {
            let bytes = Profile::new_srgb()
                .icc()
                .map_err(|e| format!("sRGB profile: {e}"))?;
            Self::new("sRGB IEC61966-2.1".into(), Arc::new(bytes)).map_err(|e| e.to_string())
        })
        .clone()
        .map_err(invalid)
    }
    pub fn id(&self) -> IccProfileId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn color_space(&self) -> IccColorSpace {
        self.space
    }
    pub fn is_press_profile(&self) -> bool {
        self.space == IccColorSpace::Cmyk && &self.bytes[12..16] == b"prtr"
    }
    pub fn is_monitor_profile(&self) -> bool {
        self.space == IccColorSpace::Rgb && &self.bytes[12..16] == b"mntr"
    }
    pub fn read(path: &std::path::Path) -> Result<Self, PetuniaError> {
        use std::io::Read;
        let file = std::fs::File::open(path).map_err(|e| PetuniaError::io(e.to_string()))?;
        if !file
            .metadata()
            .map_err(|e| PetuniaError::io(e.to_string()))?
            .is_file()
        {
            return Err(invalid("ICC source must be a regular file"));
        }
        let mut bytes = Vec::new();
        file.take((MAX_PROFILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| PetuniaError::io(e.to_string()))?;
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("ICC profile")
            .to_owned();
        Self::new(name, Arc::new(bytes))
    }
    fn native(&self) -> Result<Profile, PetuniaError> {
        Profile::new_icc(&self.bytes).map_err(|e| invalid(format!("ICC profile: {e}")))
    }
}
fn validate_header(bytes: &[u8]) -> Result<(), PetuniaError> {
    let read = |at: usize| -> usize {
        u32::from_be_bytes(bytes[at..at + 4].try_into().expect("checked ICC header")) as usize
    };
    if bytes.len() < 132 || bytes.len() > MAX_PROFILE_BYTES {
        return Err(invalid("ICC profile size must be 132 bytes–4 MiB"));
    }
    if read(0) != bytes.len() || &bytes[36..40] != b"acsp" || !matches!(bytes[8], 2 | 4) {
        return Err(invalid("invalid ICC size, signature or version"));
    }
    let tags = read(128);
    if tags > 4096 || 132 + tags * 12 > bytes.len() {
        return Err(invalid("ICC tag table budget"));
    }
    for index in 0..tags {
        let at = 132 + index * 12;
        let offset = read(at + 4);
        let length = read(at + 8);
        if length < 8
            || offset < 132 + tags * 12
            || offset % 4 != 0
            || offset
                .checked_add(length)
                .is_none_or(|end| end > bytes.len())
        {
            return Err(invalid("ICC tag range/layout"));
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IccTransformOptions {
    pub intent: crate::RenderingIntent,
    pub black_point_compensation: bool,
}
impl Default for IccTransformOptions {
    fn default() -> Self {
        Self {
            intent: crate::RenderingIntent::RelativeColorimetric,
            black_point_compensation: true,
        }
    }
}
fn intent(value: crate::RenderingIntent) -> Intent {
    match value {
        crate::RenderingIntent::Perceptual => Intent::Perceptual,
        crate::RenderingIntent::RelativeColorimetric => Intent::RelativeColorimetric,
        crate::RenderingIntent::Saturation => Intent::Saturation,
        crate::RenderingIntent::AbsoluteColorimetric => Intent::AbsoluteColorimetric,
    }
}
fn flags(options: IccTransformOptions) -> Flags {
    if options.black_point_compensation {
        Flags::BLACKPOINT_COMPENSATION
    } else {
        Flags::default()
    }
}
pub fn cmyk_to_rgb(
    source: &IccProfile,
    target: &IccProfile,
    pixels: &[[f32; 4]],
    options: IccTransformOptions,
) -> Result<Vec<[f32; 3]>, PetuniaError> {
    if source.space != IccColorSpace::Cmyk || target.space != IccColorSpace::Rgb {
        return Err(invalid("ICC CMYK→RGB profile mismatch"));
    }
    validate_channels(pixels)?;
    let transform: Transform<[f32; 4], [f32; 3]> = Transform::new_flags(
        &source.native()?,
        PixelFormat::CMYK_FLT,
        &target.native()?,
        PixelFormat::RGB_FLT,
        intent(options.intent),
        flags(options),
    )
    .map_err(|e| invalid(format!("ICC CMYK→RGB: {e}")))?;
    // LittleCMS floating CMYK uses percentages, RGB uses unit channels.
    let mut output = vec![[0.; 3]; pixels.len()];
    for (input, target) in pixels.chunks(4096).zip(output.chunks_mut(4096)) {
        let ink: Vec<_> = input.iter().map(|p| p.map(|v| v * 100.)).collect();
        transform.transform_pixels(&ink, target);
    }
    Ok(output)
}
pub fn rgb_to_cmyk(
    source: &IccProfile,
    target: &IccProfile,
    pixels: &[[f32; 3]],
    options: IccTransformOptions,
) -> Result<Vec<[f32; 4]>, PetuniaError> {
    if source.space != IccColorSpace::Rgb || target.space != IccColorSpace::Cmyk {
        return Err(invalid("ICC RGB→CMYK profile mismatch"));
    }
    validate_channels(pixels)?;
    let transform: Transform<[f32; 3], [f32; 4]> = Transform::new_flags(
        &source.native()?,
        PixelFormat::RGB_FLT,
        &target.native()?,
        PixelFormat::CMYK_FLT,
        intent(options.intent),
        flags(options),
    )
    .map_err(|e| invalid(format!("ICC RGB→CMYK: {e}")))?;
    let mut output = vec![[0.; 4]; pixels.len()];
    transform.transform_pixels(pixels, &mut output);
    for pixel in &mut output {
        for channel in pixel {
            if !channel.is_finite() {
                return Err(invalid("ICC RGB→CMYK nonfinite output"));
            }
            *channel = (*channel / 100.).clamp(0., 1.);
        }
    }
    Ok(output)
}
fn validate_channels<const N: usize>(pixels: &[[f32; N]]) -> Result<(), PetuniaError> {
    if pixels.len() > 16_777_216
        || pixels
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
    {
        return Err(invalid("ICC pixel quota or channel range"));
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IccProofSettings {
    pub proof: IccProfile,
    pub monitor: IccProfile,
    pub options: IccTransformOptions,
    pub proof_intent: crate::RenderingIntent,
}
impl IccProofSettings {
    /// Derived sRGB artwork→press→actual monitor, preserving straight alpha.
    /// Profiles remain immutable; no authored color is written back.
    pub fn apply_rgba8(
        &self,
        data: &mut [u8],
        cancelled: &dyn Fn() -> bool,
    ) -> Result<(), PetuniaError> {
        if !self.monitor.is_monitor_profile()
            || !self.proof.is_press_profile()
            || !data.len().is_multiple_of(4)
            || data.len() / 4 > 16_777_216
        {
            return Err(invalid(
                "ICC proof requires CMYK press, RGB monitor and bounded RGBA artwork",
            ));
        }
        let source = Profile::new_srgb();
        let transform: Transform<[u8; 3], [u8; 3]> = Transform::new_proofing(
            &source,
            PixelFormat::RGB_8,
            &self.monitor.native()?,
            PixelFormat::RGB_8,
            &self.proof.native()?,
            intent(self.options.intent),
            intent(self.proof_intent),
            flags(self.options) | Flags::SOFT_PROOFING,
        )
        .map_err(|e| invalid(format!("ICC proof transform: {e}")))?;
        for chunk in data.chunks_mut(4096 * 4) {
            if cancelled() {
                return Err(invalid("ICC proof cancelled"));
            }
            let mut rgb: Vec<[u8; 3]> = chunk
                .as_chunks::<4>()
                .0
                .iter()
                .map(|p| [p[0], p[1], p[2]])
                .collect();
            transform.transform_in_place(&mut rgb);
            for (pixel, mapped) in chunk.as_chunks_mut::<4>().0.iter_mut().zip(rgb) {
                pixel[..3].copy_from_slice(&mapped);
            }
        }
        Ok(())
    }
}

/// Canonical CMYK token syntax: unit channels, or explicit percentages. A bare
/// `100` is rejected, preventing ambiguous percent/unit reinterpretation.
pub fn parse_cmyk_token(token: &str) -> Result<Option<[f32; 4]>, PetuniaError> {
    let token = token.trim();
    if !token.to_ascii_lowercase().starts_with("cmyk(") {
        return Ok(None);
    }
    let inner = token
        .get(5..)
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| invalid("invalid CMYK token"))?;
    let mut parts = inner.split(',');
    let mut channels = [0.; 4];
    for value in &mut channels {
        let source = parts
            .next()
            .ok_or_else(|| invalid("CMYK requires four channels"))?
            .trim();
        let (source, scale) = source
            .strip_suffix('%')
            .map_or((source, 1.), |v| (v.trim(), 100.));
        *value = source
            .parse::<f32>()
            .map_err(|_| invalid("invalid CMYK channel"))?
            / scale;
        if !value.is_finite() || !(0.0..=1.).contains(value) {
            return Err(invalid("CMYK channel must be in 0–1 or 0–100%"));
        }
    }
    if parts.next().is_some() {
        return Err(invalid("CMYK requires exactly four channels"));
    }
    Ok(Some(channels))
}

/// A native transform belongs to the rendering worker, never to shared storage.
/// Creating it once per frame avoids reparsing/recompiling a profile per paint.
pub struct CmykDisplayTransform {
    transform: Transform<[f32; 4], [f32; 3]>,
}
impl CmykDisplayTransform {
    pub fn new(profile: &IccProfile, options: IccTransformOptions) -> Result<Self, PetuniaError> {
        if profile.space != IccColorSpace::Cmyk {
            return Err(invalid("CMYK display profile mismatch"));
        }
        let transform = Transform::new_flags(
            &profile.native()?,
            PixelFormat::CMYK_FLT,
            &Profile::new_srgb(),
            PixelFormat::RGB_FLT,
            intent(options.intent),
            flags(options),
        )
        .map_err(|e| invalid(format!("ICC CMYK display: {e}")))?;
        Ok(Self { transform })
    }
    pub fn convert(&self, ink: [f32; 4]) -> Result<[f32; 3], PetuniaError> {
        validate_channels(&[ink])?;
        let mut rgb = [[0.; 3]];
        self.transform
            .transform_pixels(&[ink.map(|v| v * 100.)], &mut rgb);
        if rgb[0].iter().any(|v| !v.is_finite()) {
            return Err(invalid("ICC transform produced invalid channels"));
        }
        Ok(rgb[0].map(|v| v.clamp(0., 1.)))
    }
    /// Bounded caller-owned output, suitable for one resident tile. Native
    /// alpha is not passed to the CMM and is retained by the raster adapter.
    pub fn convert_batch(
        &self,
        ink: &[[f32; 4]],
        rgb: &mut [[f32; 3]],
    ) -> Result<(), PetuniaError> {
        validate_channels(ink)?;
        if ink.len() != rgb.len() {
            return Err(invalid("ICC batch layout mismatch"));
        }
        for (input, output) in ink.chunks(4096).zip(rgb.chunks_mut(4096)) {
            let percent: Vec<_> = input.iter().map(|p| p.map(|v| v * 100.)).collect();
            self.transform.transform_pixels(&percent, output);
            for pixel in output {
                for channel in pixel {
                    if !channel.is_finite() {
                        return Err(invalid("ICC produced nonfinite display channels"));
                    }
                    *channel = channel.clamp(0., 1.);
                }
            }
        }
        Ok(())
    }
}

/// One explicit CMYK-to-CMYK worker transform. Identity profile conversion
/// copies exact unit samples rather than sending ink through a PCS round trip.
pub struct CmykInkTransform {
    transform: Option<Transform<[f32; 4], [f32; 4]>>,
}
impl CmykInkTransform {
    pub fn new(
        source: &IccProfile,
        target: &IccProfile,
        options: IccTransformOptions,
    ) -> Result<Self, PetuniaError> {
        if source.space != IccColorSpace::Cmyk || !target.is_press_profile() {
            return Err(invalid("ICC CMYK ink profile mismatch"));
        }
        let transform = if source.id == target.id {
            None
        } else {
            Some(
                Transform::new_flags(
                    &source.native()?,
                    PixelFormat::CMYK_FLT,
                    &target.native()?,
                    PixelFormat::CMYK_FLT,
                    intent(options.intent),
                    flags(options),
                )
                .map_err(|e| invalid(format!("ICC CMYK ink conversion: {e}")))?,
            )
        };
        Ok(Self { transform })
    }
    pub fn convert(&self, input: &[[f32; 4]], output: &mut [[f32; 4]]) -> Result<(), PetuniaError> {
        validate_channels(input)?;
        if input.len() != output.len() {
            return Err(invalid("ICC ink batch layout mismatch"));
        }
        let Some(transform) = &self.transform else {
            output.copy_from_slice(input);
            return Ok(());
        };
        for (input, output) in input.chunks(4096).zip(output.chunks_mut(4096)) {
            let percent: Vec<_> = input.iter().map(|p| p.map(|v| v * 100.)).collect();
            transform.transform_pixels(&percent, output);
            for pixel in output {
                for channel in pixel {
                    if !channel.is_finite() {
                        return Err(invalid("ICC produced nonfinite ink channels"));
                    }
                    *channel = (*channel / 100.).clamp(0., 1.);
                }
            }
        }
        Ok(())
    }
}

/// Real CMYK→CMYK conversion; no RGB intermediary. Black preservation needs a
/// tested DeviceLink/profile policy and is never implemented with fixed factors.
pub fn cmyk_to_cmyk(
    source: &IccProfile,
    target: &IccProfile,
    pixels: &[[f32; 4]],
    options: IccTransformOptions,
) -> Result<Vec<[f32; 4]>, PetuniaError> {
    if source.space != IccColorSpace::Cmyk || target.space != IccColorSpace::Cmyk {
        return Err(invalid("ICC CMYK→CMYK profile mismatch"));
    }
    validate_channels(pixels)?;
    if source.id == target.id {
        return Ok(pixels.to_vec());
    }
    let transform: Transform<[f32; 4], [f32; 4]> = Transform::new_flags(
        &source.native()?,
        PixelFormat::CMYK_FLT,
        &target.native()?,
        PixelFormat::CMYK_FLT,
        intent(options.intent),
        flags(options),
    )
    .map_err(|e| invalid(format!("ICC CMYK→CMYK: {e}")))?;
    let mut output = vec![[0.; 4]; pixels.len()];
    // Bound temporary ink storage to 4096 pixels independently of caller size.
    for (input, target) in pixels.chunks(4096).zip(output.chunks_mut(4096)) {
        let ink: Vec<_> = input.iter().map(|p| p.map(|v| v * 100.)).collect();
        transform.transform_pixels(&ink, target);
        for pixel in target {
            for channel in pixel {
                *channel /= 100.;
            }
        }
    }
    Ok(output)
}
