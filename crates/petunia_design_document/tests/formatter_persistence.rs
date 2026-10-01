use petunia_design_document::ValueFormatter;

#[test]
fn all_data_formatters_roundtrip_and_legacy_currency_remains_readable() {
    for formatter in [
        ValueFormatter::None,
        ValueFormatter::Uppercase,
        ValueFormatter::Lowercase,
        ValueFormatter::Currency {
            symbol: "R$".into(),
            decimals: 2,
        },
        ValueFormatter::NumberDecimals(3),
        ValueFormatter::Prefix("Dr. ".into()),
        ValueFormatter::Suffix(" kg".into()),
    ] {
        let json = serde_json::to_string(&formatter).unwrap();
        assert_eq!(
            serde_json::from_str::<ValueFormatter>(&json).unwrap(),
            formatter
        );
    }
    assert_eq!(
        serde_json::from_str::<ValueFormatter>(r#"{"kind":"currency","symbol":"R$","decimals":2}"#)
            .unwrap(),
        ValueFormatter::Currency {
            symbol: "R$".into(),
            decimals: 2
        }
    );
}
