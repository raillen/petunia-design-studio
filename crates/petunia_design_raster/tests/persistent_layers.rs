//! Regression sources prepared for the final MVP validation batch.
use petunia_design_geometry::{GAffine, GPoint};
use petunia_design_raster::{
    AlphaMode, BitDepth, BlendMode, BrushDab, PixelFormat, RasterLayer, RasterLayerKind, Tile,
    TileCoord, TileMap,
};
use std::sync::Arc;
fn pixels() -> RasterLayer {
    RasterLayer::new(256, 128, RasterLayerKind::Pixels, BitDepth::Sixteen).unwrap()
}
#[test]
fn snapshot_copies_only_the_touched_tile_payload() {
    let mut layer = pixels();
    layer.set_pixel(1, 1, [1.0, 0.0, 0.0, 1.0]).unwrap();
    layer.set_pixel(130, 1, [0.0, 1.0, 0.0, 1.0]).unwrap();
    layer.commit();
    let old = layer.clone();
    layer.set_pixel(2, 1, [0.0, 0.0, 1.0, 1.0]).unwrap();
    assert!(!Arc::ptr_eq(
        &old.tiles().get_tile(TileCoord::new(0, 0)).unwrap().data,
        &layer.tiles().get_tile(TileCoord::new(0, 0)).unwrap().data
    ));
    assert!(Arc::ptr_eq(
        &old.tiles().get_tile(TileCoord::new(1, 0)).unwrap().data,
        &layer.tiles().get_tile(TileCoord::new(1, 0)).unwrap().data
    ));
    assert_eq!(old.pixel(2, 1).unwrap(), [0.0; 4]);
}
#[test]
fn commit_changes_lifecycle_without_copying_pixels() {
    let mut layer = pixels();
    layer.set_pixel(1, 1, [1.0; 4]).unwrap();
    let previous = layer.clone();
    layer.commit();
    assert!(Arc::ptr_eq(
        &previous
            .tiles()
            .get_tile(TileCoord::new(0, 0))
            .unwrap()
            .data,
        &layer.tiles().get_tile(TileCoord::new(0, 0)).unwrap().data
    ));
}
#[test]
fn sixteen_bit_layer_and_coordinate_sequences_roundtrip() {
    let mut layer = pixels();
    layer
        .set_pixel(200, 50, [0.12345, 0.56789, 0.33333, 0.78901])
        .unwrap();
    layer.commit();
    let bytes = serde_json::to_vec(&layer).unwrap();
    let restored: RasterLayer = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(layer, restored);
    assert!((restored.pixel(200, 50).unwrap()[0] - 0.12345).abs() < 0.00002);
}
#[test]
fn finite_layer_edges_never_allocate_outside_tiles() {
    let mut layer = pixels();
    assert!(!layer.set_pixel(-1, 1, [1.0; 4]).unwrap());
    assert!(!layer.set_pixel(256, 1, [1.0; 4]).unwrap());
    assert_eq!(layer.tiles().resident_tile_count(), 0);
}
#[test]
fn writing_zero_to_absent_pixel_or_mask_is_sparse() {
    let mut p = pixels();
    let mut m = RasterLayer::new(256, 128, RasterLayerKind::Mask, BitDepth::Eight).unwrap();
    assert!(!p.set_pixel(1, 1, [0.0; 4]).unwrap());
    assert!(!m.set_pixel(1, 1, [1.0, 1.0, 1.0, 0.0]).unwrap());
    assert_eq!(p.resident_bytes() + m.resident_bytes(), 0);
}
#[test]
fn masks_expose_coverage_alpha_and_preserve_deep_samples() {
    let mut layer = RasterLayer::new(16, 16, RasterLayerKind::Mask, BitDepth::Sixteen).unwrap();
    layer.set_pixel(4, 4, [0.0, 0.0, 0.0, 0.12345]).unwrap();
    assert!((layer.pixel(4, 4).unwrap()[3] - 0.12345).abs() < 0.00002);
    assert_eq!(layer.pixel(4, 4).unwrap()[..3], [1.0; 3]);
}
#[test]
fn malformed_storage_and_edge_padding_are_rejected_at_admission() {
    let mut tile = Tile::new_empty(
        TileCoord::new(0, 0),
        PixelFormat::Rgba8,
        AlphaMode::Straight,
    );
    tile.set_pixel_normalized(2, 2, [1.0; 4]).unwrap();
    let map =
        TileMap::from_tiles(PixelFormat::Rgba8, AlphaMode::Straight, [Arc::new(tile)]).unwrap();
    assert!(RasterLayer::from_tiles(1, 1, RasterLayerKind::Pixels, map).is_err());
    let wrong = Tile::new_empty(
        TileCoord::new(0, 0),
        PixelFormat::Gray8,
        AlphaMode::Straight,
    );
    assert!(
        TileMap::from_tiles(PixelFormat::Rgba8, AlphaMode::Straight, [Arc::new(wrong)]).is_err()
    );
}
#[test]
fn duplicate_tile_coordinates_and_oversized_planes_are_errors() {
    let tile = Arc::new(Tile::new_empty(
        TileCoord::new(0, 0),
        PixelFormat::Rgba8,
        AlphaMode::Straight,
    ));
    assert!(TileMap::from_tiles(
        PixelFormat::Rgba8,
        AlphaMode::Straight,
        [tile.clone(), tile]
    )
    .is_err());
    assert!(RasterLayer::new(0, 1, RasterLayerKind::Pixels, BitDepth::Eight).is_err());
    assert!(RasterLayer::new(32768, 32768, RasterLayerKind::Pixels, BitDepth::Eight).is_err());
}
#[test]
fn world_circle_is_preserved_under_nonuniform_layer_scaling() {
    let mut layer = pixels();
    let original = layer.clone();
    let mut coverage =
        RasterLayer::new(256, 128, RasterLayerKind::Mask, BitDepth::Sixteen).unwrap();
    let transform = GAffine::translate(20.0, 10.0).after(GAffine::scale(2.0, 0.5));
    let center = transform.apply(GPoint::new(20.5, 20.5));
    let dab = BrushDab {
        center_x: center.x,
        center_y: center.y,
        radius: 4.0,
        hardness: 1.0,
        opacity: 1.0,
        color: [1.0, 0.0, 0.0, 1.0],
        blend_mode: BlendMode::Normal,
    };
    layer
        .stamp(
            &dab,
            transform,
            &original,
            &mut coverage,
            1.0,
            &mut 100_000,
            |_, _| Ok(1.0),
        )
        .unwrap();
    assert_eq!(layer.pixel(20, 26).unwrap()[3], 1.0);
    assert_eq!(layer.pixel(23, 20).unwrap()[3], 0.0);
}
#[test]
fn overlapping_dabs_do_not_exceed_master_stroke_opacity() {
    let mut layer = pixels();
    let original = layer.clone();
    let mut coverage =
        RasterLayer::new(256, 128, RasterLayerKind::Mask, BitDepth::Sixteen).unwrap();
    let mut dab = BrushDab::paint_dab(10.5, 10.5);
    dab.radius = 3.0;
    dab.hardness = 1.0;
    dab.opacity = 1.0;
    for _ in 0..10 {
        layer
            .stamp(
                &dab,
                GAffine::IDENTITY,
                &original,
                &mut coverage,
                0.5,
                &mut 100_000,
                |_, _| Ok(1.0),
            )
            .unwrap();
    }
    assert!((layer.pixel(10, 10).unwrap()[3] - 0.5).abs() < 0.00002);
}
#[test]
fn dab_budget_failure_precedes_pixel_writes() {
    let mut layer = pixels();
    let original = layer.clone();
    let mut coverage =
        RasterLayer::new(256, 128, RasterLayerKind::Mask, BitDepth::Sixteen).unwrap();
    let dab = BrushDab::paint_dab(20.0, 20.0);
    assert!(layer
        .stamp(
            &dab,
            GAffine::IDENTITY,
            &original,
            &mut coverage,
            1.0,
            &mut 1,
            |_, _| Ok(1.0)
        )
        .is_err());
    assert_eq!(layer, original);
}

#[test]
fn opaque_masks_are_sparse_and_keep_erased_tiles_after_commit() {
    let mut mask = RasterLayer::opaque_mask(129, 3, BitDepth::Sixteen).unwrap();
    assert_eq!(mask.resident_bytes(), 0);
    assert_eq!(mask.pixel(128, 2).unwrap(), [1.; 4]);
    assert_eq!(mask.pixel(129, 2).unwrap(), [0.; 4]);
    let original = mask.clone();
    mask.set_pixel(128, 2, [1., 1., 1., 0.]).unwrap();
    mask.commit();
    assert_eq!(original.pixel(128, 2).unwrap(), [1.; 4]);
    assert_eq!(mask.pixel(128, 2).unwrap()[3], 0.);
    assert_eq!(mask.pixel(127, 2).unwrap()[3], 1.);
    let restored: RasterLayer =
        serde_json::from_slice(&serde_json::to_vec(&mask).unwrap()).unwrap();
    assert_eq!(restored, mask);
    restored.validate().unwrap();
}
