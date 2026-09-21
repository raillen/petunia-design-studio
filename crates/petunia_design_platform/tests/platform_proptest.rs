use petunia_design_platform::{ClipboardContent, ClipboardService, FileFilter, HeadlessClipboard};
use proptest::prelude::*;
use std::path::PathBuf;

proptest! {
    #[test]
    fn clipboard_arbitrary_text_roundtrip(text in ".*") {
        let mut cb = HeadlessClipboard::new();
        cb.set_text(&text).unwrap();
        let read = cb.get_text().unwrap().unwrap();
        prop_assert_eq!(read, text);
    }

    #[test]
    fn clipboard_arbitrary_image_data_roundtrip(w in 1u32..20, h in 1u32..20) {
        let len = (w * h * 4) as usize;
        let data = vec![128u8; len];
        let mut cb = HeadlessClipboard::new();
        let img = ClipboardContent::ImageRgba8 { width: w, height: h, data };
        cb.set_content(img.clone()).unwrap();
        let retrieved = cb.get_content().unwrap().unwrap();
        prop_assert_eq!(retrieved, img);
    }

    #[test]
    fn file_filter_petunia_design_matches_case_insensitively(stem in "[a-zA-Z0-9_-]{1,15}") {
        let native_upper = PathBuf::from(format!("{stem}.PTND"));
        let native_lower = PathBuf::from(format!("{stem}.ptnd"));
        let legacy_mixed = PathBuf::from(format!("{stem}.Aubrieta"));
        let legacy_short = PathBuf::from(format!("{stem}.aubri"));
        let unrelated = PathBuf::from(format!("{stem}.svg"));

        let filter = FileFilter::PtndPackage;
        // Current native format is case-tolerant.
        prop_assert!(filter.matches(&native_upper));
        prop_assert!(filter.matches(&native_lower));
        // Legacy suffixes never appear in the picker: migration goes through
        // the package reader in petunia_design_io (15.A).
        prop_assert!(!filter.matches(&legacy_mixed));
        prop_assert!(!filter.matches(&legacy_short));
        prop_assert!(!filter.matches(&unrelated));
    }
}
