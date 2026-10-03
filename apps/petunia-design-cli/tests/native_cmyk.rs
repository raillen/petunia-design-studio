use petunia_design_color::IccProfile;
use petunia_design_io::{export_cmyk_tiff, import_cmyk_tiff};
use petunia_design_raster::{EncodedImage, PixelFormat, RasterLayer};
use std::{
    path::Path,
    process::{Command, Output},
    sync::Arc,
};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_petunia-design-cli"))
        .args(args)
        .output()
        .unwrap()
}
fn successful(args: &[&str]) -> Output {
    let output = run(args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}
#[test]
fn import_inspect_assign_convert_and_export_are_real_native_file_flows() {
    let dir = tempfile::tempdir().unwrap();
    let profile = IccProfile::new(
        "Synthetic press".into(),
        Arc::new(include_bytes!("../../../fixtures/color/synthetic-cmyk.icc").to_vec()),
    )
    .unwrap();
    let input = dir.path().join("input.tif");
    let package = dir.path().join("input.PTND");
    let output = dir.path().join("output.tif");
    let icc = dir.path().join("press.icc");
    let assigned = dir.path().join("assigned.PTND");
    let converted = dir.path().join("converted.PTND");
    let pdf = dir.path().join("native.pdf");
    let data = [0_u16, 0, 0, 65535, 65535, 123, 456, 789, 0, 12345]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    let layer =
        RasterLayer::from_cmyka_bytes(2, 1, PixelFormat::Cmyka16, profile.clone(), &data).unwrap();
    std::fs::write(&input, export_cmyk_tiff(&layer, 300., &|| false).unwrap()).unwrap();
    std::fs::write(&icc, profile.bytes()).unwrap();
    successful(&["cmyk-import", text(&input), text(&package), "--dpi", "300"]);
    let inspection = successful(&[
        "cmyk-inspect",
        text(&package),
        "--object",
        "ObjectId:2",
        "--tac-percent",
        "90",
    ]);
    let report: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(report["scope"], "authored-layer");
    assert_eq!(report["format"], "Cmyka16");
    assert_eq!(report["pixels_over_limit"], 1);
    successful(&["cmyk-assign", text(&package), text(&icc), text(&assigned)]);
    successful(&[
        "cmyk-convert",
        text(&assigned),
        text(&icc),
        text(&converted),
    ]);
    successful(&[
        "cmyk-export-layer",
        text(&converted),
        text(&output),
        "--dpi",
        "300",
    ]);
    assert_eq!(
        import_cmyk_tiff(&std::fs::read(&output).unwrap(), EncodedImage::MAX_BYTES)
            .unwrap()
            .cmyka_bytes(&|| false)
            .unwrap(),
        data
    );
    successful(&["export-pdf", text(&converted), text(&pdf)]);
    assert!(std::fs::read(pdf).unwrap().starts_with(b"%PDF-"));
}
#[test]
fn help_unknown_commands_and_invalid_options_do_not_run_conformance() {
    assert!(String::from_utf8(successful(&["--help"]).stdout)
        .unwrap()
        .contains("authored layer"));
    for args in [
        vec!["bogus"],
        vec!["cmyk-import"],
        vec!["cmyk-inspect", "input.PTND", "--tac-percent", "NaN"],
        vec!["cmyk-inspect", "input.PTND", "--object", "2"],
        vec!["cmyk-import", "input.tif", "output.PTND", "--dpi", "-1"],
        vec!["cmyk-inspect", "input.PTND", "--dpi", "300"],
    ] {
        let output = run(&args);
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("scene fragments"));
    }
}
