use aubrieta_platform::{ClipboardContent, ClipboardService, FileFilter, HeadlessClipboard};
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
    fn file_filter_aubrieta_matches_case_insensitively(stem in "[a-zA-Z0-9_-]{1,15}") {
        let path1 = PathBuf::from(format!("{stem}.aubrieta"));
        let path2 = PathBuf::from(format!("{stem}.AUBRIETA"));
        let path3 = PathBuf::from(format!("{stem}.aubri"));
        let path4 = PathBuf::from(format!("{stem}.AUBRI"));

        let filter = FileFilter::AubrietaPackage;
        prop_assert!(filter.matches(&path1));
        prop_assert!(filter.matches(&path2));
        prop_assert!(filter.matches(&path3));
        prop_assert!(filter.matches(&path4));
    }
}
