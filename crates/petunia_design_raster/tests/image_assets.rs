//! Regression source prepared during implementation; execution is deferred.
use image::ImageEncoder;
use petunia_design_raster::{
    decode_image, EncodedImage, ImageAssetError, ImageCache, ImageCacheLimits, ImageDecodeLimits,
    PixelFormat,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn png(width: u32, height: u32, bytes: &[u8], color: image::ExtendedColorType) -> EncodedImage {
    let mut encoded = Vec::new();
    image::codecs::png::PngEncoder::new(&mut encoded)
        .write_image(bytes, width, height, color)
        .unwrap();
    EncodedImage::new(encoded).unwrap()
}
fn solid(pixel: [u8; 4]) -> EncodedImage {
    png(1, 1, &pixel, image::ExtendedColorType::Rgba8)
}
fn cache(bytes: usize, entries: usize) -> ImageCache {
    ImageCache::new(ImageCacheLimits {
        max_bytes: bytes,
        max_entries: entries,
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn content_key_and_native_wire_ignore_ownership() {
    let a = solid([10, 20, 30, 255]);
    let b = EncodedImage::new(a.as_slice().to_vec()).unwrap();
    assert_eq!(a.content_key(), b.content_key());
    assert_ne!(a.content_key(), solid([30, 20, 10, 255]).content_key());
    let wire = serde_json::to_value(&a).unwrap();
    assert_eq!(wire, serde_json::json!(a.as_slice()));
    assert_eq!(serde_json::from_value::<EncodedImage>(wire).unwrap(), a);
}

#[test]
fn duplicate_sources_share_preparation_and_preserve_original_bytes() {
    let source = solid([100, 50, 0, 128]);
    let original = source.as_slice().to_vec();
    let cache = cache(1024, 4);
    let first = cache.prepare(&source, &|| false).unwrap();
    let duplicate = EncodedImage::new(original.clone()).unwrap();
    let second = cache.prepare(&duplicate, &|| false).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(source.as_slice(), original);
    assert_eq!(cache.stats().misses, 1);
    assert_eq!(cache.stats().hits, 1);
    assert_eq!(first.levels()[0].premultiplied_rgba8(), [50, 25, 0, 128]);
}

#[test]
fn concurrent_misses_have_one_resident_preparation() {
    let source = Arc::new(solid([0, 100, 50, 255]));
    let cache = Arc::new(cache(1024, 4));
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let source = source.clone();
            let cache = cache.clone();
            std::thread::spawn(move || cache.prepare(&source, &|| false).unwrap())
        })
        .collect();
    let images: Vec<_> = handles
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert!(images.iter().all(|image| Arc::ptr_eq(image, &images[0])));
    assert_eq!(cache.stats().misses, 1);
    assert_eq!(cache.stats().live_bytes, 4);
}

#[test]
fn pinned_bytes_cannot_be_hidden_by_eviction() {
    let cache = cache(4, 1);
    let first = cache.prepare(&solid([255, 0, 0, 255]), &|| false).unwrap();
    assert!(matches!(
        cache.prepare(&solid([0, 255, 0, 255]), &|| false),
        Err(ImageAssetError::Limit(_))
    ));
    assert_eq!(cache.stats().live_bytes, 4);
    assert_eq!(first.levels()[0].premultiplied_rgba8(), [255, 0, 0, 255]);
    drop(first);
    let second = cache.prepare(&solid([0, 255, 0, 255]), &|| false).unwrap();
    assert_eq!(second.levels()[0].premultiplied_rgba8(), [0, 255, 0, 255]);
    assert_eq!(cache.stats().evictions, 1);
    assert_eq!(cache.stats().live_bytes, 4);
}

#[test]
fn resident_account_survives_cache_owner_until_last_image_drops() {
    let cache = Arc::new(cache(1024, 4));
    let image = cache.prepare(&solid([0, 0, 255, 255]), &|| false).unwrap();
    drop(cache);
    assert_eq!(image.width(), 1);
    assert_eq!(image.levels()[0].premultiplied_rgba8(), [0, 0, 255, 255]);
}

#[test]
fn lru_keeps_the_recently_used_source() {
    let cache = cache(8, 2);
    let a = solid([255, 0, 0, 255]);
    let b = solid([0, 255, 0, 255]);
    let c = solid([0, 0, 255, 255]);
    drop(cache.prepare(&a, &|| false).unwrap());
    drop(cache.prepare(&b, &|| false).unwrap());
    drop(cache.prepare(&a, &|| false).unwrap());
    drop(cache.prepare(&c, &|| false).unwrap());
    let misses = cache.stats().misses;
    drop(cache.prepare(&a, &|| false).unwrap());
    assert_eq!(cache.stats().misses, misses);
    drop(cache.prepare(&b, &|| false).unwrap());
    assert_eq!(cache.stats().misses, misses + 1);
}

#[test]
fn invalid_sources_are_negatively_cached_with_bounded_entries() {
    let cache = cache(1024, 2);
    let source = EncodedImage::new(b"not an image".to_vec()).unwrap();
    assert!(matches!(
        cache.prepare(&source, &|| false),
        Err(ImageAssetError::Invalid(_))
    ));
    assert!(cache.prepare(&source, &|| false).is_err());
    assert_eq!(cache.stats().misses, 1);
    for byte in 0..8 {
        let _ = cache.prepare(&EncodedImage::new(vec![byte]).unwrap(), &|| false);
    }
    assert_eq!(cache.stats().entries, 2);
    assert_eq!(cache.stats().live_bytes, 0);
}

#[test]
fn cancellation_rejects_warm_results_and_releases_cold_reservations() {
    let source = png(8, 8, &[100; 8 * 8 * 4], image::ExtendedColorType::Rgba8);
    let cache = cache(4096, 4);
    let calls = AtomicUsize::new(0);
    let result = cache.prepare(&source, &|| calls.fetch_add(1, Ordering::Relaxed) > 5);
    assert!(matches!(result, Err(ImageAssetError::Cancelled)));
    assert_eq!(cache.stats().live_bytes, 0);
    assert_eq!(cache.stats().entries, 0);
    drop(cache.prepare(&source, &|| false).unwrap());
    assert!(matches!(
        cache.prepare(&source, &|| true),
        Err(ImageAssetError::Cancelled)
    ));
}

#[test]
fn odd_area_mips_keep_the_last_source_pixel_and_use_linear_light() {
    let source = png(
        3,
        1,
        &[0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255],
        image::ExtendedColorType::Rgba8,
    );
    let image = cache(1024, 4).prepare(&source, &|| false).unwrap();
    let pixel = image.levels()[1].premultiplied_rgba8();
    assert!((155..=157).contains(&pixel[0])); // encode_sRGB(1/3), not encoded-byte average 85
    assert_eq!(pixel[0], pixel[1]);
    assert_eq!(pixel[1], pixel[2]);
    assert_eq!(pixel[3], 255);
    assert_eq!(image.level_for_scale(1.0 / 3.0), 1);
    assert_eq!(image.level_for_scale(0.5), 0);
}

#[test]
fn transparent_source_rgb_does_not_create_mip_fringes() {
    let source = png(
        2,
        1,
        &[255, 0, 0, 255, 0, 0, 255, 0],
        image::ExtendedColorType::Rgba8,
    );
    let image = cache(1024, 4).prepare(&source, &|| false).unwrap();
    assert_eq!(image.levels()[1].premultiplied_rgba8(), [128, 0, 0, 128]);
}

#[test]
fn gray_and_gray_alpha_16_keep_low_sample_bits_in_little_endian() {
    for (color, samples) in [
        (image::ExtendedColorType::L16, vec![0x1234_u16]),
        (image::ExtendedColorType::La16, vec![0x1234, 0x3456]),
    ] {
        let bytes: Vec<_> = samples
            .iter()
            .flat_map(|sample| sample.to_ne_bytes())
            .collect();
        let source = png(1, 1, &bytes, color);
        let decoded = decode_image(source.as_slice(), ImageDecodeLimits::default()).unwrap();
        assert_eq!(decoded.format, PixelFormat::Rgba16);
        let alpha = if samples.len() == 2 {
            samples[1]
        } else {
            65535
        };
        let expected: Vec<_> = [0x1234_u16, 0x1234, 0x1234, alpha]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(decoded.data, expected);
    }
}

#[test]
fn quotas_reject_before_mip_publication() {
    let source = png(2, 2, &[255; 16], image::ExtendedColorType::Rgba8);
    for limits in [
        ImageDecodeLimits {
            max_encoded_bytes: 1,
            ..Default::default()
        },
        ImageDecodeLimits {
            max_pixels: 3,
            ..Default::default()
        },
        ImageDecodeLimits {
            max_decoded_bytes: 15,
            ..Default::default()
        },
        ImageDecodeLimits {
            max_working_bytes: 10,
            ..Default::default()
        },
        ImageDecodeLimits {
            max_dimension: 1,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            decode_image(source.as_slice(), limits),
            Err(ImageAssetError::Limit(_))
        ));
    }
    let cache = cache(19, 4); // 2x2 base + 1x1 level needs 20 bytes
    assert!(matches!(
        cache.prepare(&source, &|| false),
        Err(ImageAssetError::Limit(_))
    ));
    assert_eq!(cache.stats().live_bytes, 0);
}

#[test]
fn cmyk_jpeg_is_rejected_before_lossy_library_preview() {
    let header = [
        0xff, 0xd8, 0xff, 0xc0, 0, 20, 8, 0, 1, 0, 1, 4, 1, 0x11, 0, 2, 0x11, 0, 3, 0x11, 0, 4,
        0x11, 0,
    ];
    assert!(matches!(
        decode_image(&header, ImageDecodeLimits::default()),
        Err(ImageAssetError::Unsupported(_))
    ));
}

#[test]
fn profiles_remain_in_raw_decode_but_require_a_cmm_for_preparation() {
    let mut bytes = Vec::new();
    let mut encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    encoder.set_icc_profile(vec![7; 128]).unwrap();
    encoder
        .write_image(&[10, 20, 30, 255], 1, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    let source = EncodedImage::new(bytes).unwrap();
    let decoded = decode_image(source.as_slice(), ImageDecodeLimits::default()).unwrap();
    assert_eq!(decoded.icc_profile.unwrap().as_slice(), &[7; 128]);
    assert!(matches!(
        cache(1024, 4).prepare(&source, &|| false),
        Err(ImageAssetError::Unsupported(_))
    ));
}

#[test]
fn empty_cache_budgets_are_rejected() {
    for limits in [
        ImageCacheLimits {
            max_entries: 0,
            ..Default::default()
        },
        ImageCacheLimits {
            max_bytes: 0,
            ..Default::default()
        },
    ] {
        assert!(ImageCache::new(limits).is_err());
    }
}

#[test]
fn jpeg_exif_orientation_changes_display_dimensions_without_rewriting_original() {
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new(&mut jpeg)
        .encode(
            &[255, 0, 0, 255, 0, 0],
            2,
            1,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
    let exif = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
    let mut bytes = jpeg[..2].to_vec();
    bytes.extend_from_slice(&[0xff, 0xe1]);
    bytes.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
    bytes.extend_from_slice(exif);
    bytes.extend_from_slice(&jpeg[2..]);
    let source = EncodedImage::new(bytes.clone()).unwrap();
    let decoded = decode_image(source.as_slice(), ImageDecodeLimits::default()).unwrap();
    assert_eq!((decoded.width, decoded.height), (1, 2));
    assert_eq!(source.as_slice(), bytes);
}

#[test]
fn pinned_entry_quota_releases_unpublishable_preparation() {
    let cache = cache(1024, 1);
    let a = solid([255, 0, 0, 255]);
    let b = solid([0, 255, 0, 255]);
    let pinned = cache.prepare(&a, &|| false).unwrap();
    assert!(matches!(
        cache.prepare(&b, &|| false),
        Err(ImageAssetError::Limit(_))
    ));
    assert_eq!(cache.stats().live_bytes, 4);
    assert_eq!(cache.stats().entries, 1);
    assert!(Arc::ptr_eq(&pinned, &cache.prepare(&a, &|| false).unwrap()));
}

#[test]
fn fixed_budget_failures_remain_retryable_without_negative_entries() {
    let cache = cache(3, 4);
    let source = solid([0, 0, 0, 255]);
    assert!(matches!(
        cache.prepare(&source, &|| false),
        Err(ImageAssetError::Limit(_))
    ));
    assert!(matches!(
        cache.prepare(&source, &|| false),
        Err(ImageAssetError::Limit(_))
    ));
    assert_eq!(cache.stats().misses, 2);
    assert_eq!(cache.stats().entries, 0);
}
