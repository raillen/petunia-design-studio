use petunia_design_document::{DocumentObject, ShapeKind};
use petunia_design_foundation::ObjectId;
use petunia_design_io::{decode_clipboard_fragment, encode_clipboard_fragment};
use petunia_design_raster::{BitDepth, RasterLayer};
use std::sync::Arc;
#[test]
fn native_fragment_roundtrips_opaque_mask_and_frame_origin() {
    let mut object = DocumentObject::new(ObjectId::new(7), "Mask");
    object.bounds = Some([-100., 50., 32., 32.]);
    object.shape = Some(ShapeKind::Raster {
        layer: Arc::new(RasterLayer::opaque_mask(32, 32, BitDepth::Sixteen).unwrap()),
    });
    let bytes = encode_clipboard_fragment(vec![object.clone()], [-100., 50., 300., 300.]).unwrap();
    let restored = decode_clipboard_fragment(&bytes).unwrap();
    assert_eq!(restored.surfaces()[0].origin, [-100., 50.]);
    assert_eq!(restored.find_object(object.id), Some(&object));
}
#[test]
fn malformed_or_overbudget_native_fragments_are_rejected() {
    assert!(decode_clipboard_fragment(b"not a PTND archive").is_err());
    assert!(decode_clipboard_fragment(&vec![0; 16 * 1024 * 1024 + 1]).is_err());
}
