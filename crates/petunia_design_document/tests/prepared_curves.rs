use petunia_design_document::adjustments::PreparedCurve;

#[test]
fn monotone_curve_retains_endpoints_without_overshoot() {
    let curve = PreparedCurve::new(&[[0.0, 0.0], [0.25, 0.7], [0.75, 0.8], [1.0, 1.0]]).unwrap();
    assert_eq!(curve.sample(0.0), 0.0);
    assert_eq!(curve.sample(1.0), 1.0);
    let mut previous = 0.0;
    for i in 0..=1000 {
        let value = curve.sample(i as f32 / 1000.0);
        assert!(value >= previous && value <= 1.0);
        previous = value;
    }
}

#[test]
fn neutral_linear_and_constant_curves_have_defined_results() {
    assert_eq!(PreparedCurve::new(&[]).unwrap().sample(0.3), 0.3);
    assert_eq!(PreparedCurve::new(&[[0.5, 0.2]]).unwrap().sample(0.8), 0.2);
    assert_eq!(
        PreparedCurve::new(&[[0.0, 0.0], [1.0, 1.0]])
            .unwrap()
            .sample(0.5),
        0.5
    );
}

#[test]
fn invalid_curve_controls_fail_before_pixel_sampling() {
    assert!(PreparedCurve::new(&[[0.5, 0.3], [0.2, 0.8]]).is_err());
    assert!(PreparedCurve::new(&[[0.0, f64::NAN]]).is_err());
    assert!(PreparedCurve::new(&[[0.0, -0.2]]).is_err());
}
