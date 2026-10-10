//! Embedded, pinned MIT icon assets. Missing fill variants fall back within
//! the selected family, preserving the meaning of the command.
use base64::Engine;
use petunia_ui::{IconFamily, IconStyle};

include!("../assets/icons/catalog.rs");

pub fn source(key: &str, family: IconFamily, style: IconStyle, color: &str) -> String {
    let key = match key {
        "layer.visible" => "object.visible",
        "layer.hidden" => "object.hidden",
        "layer.locked" => "object.lock",
        "layer.unlocked" => "object.unlock",
        "object.rectangle" | "object.ellipse" => "object.shape",
        other => other,
    };
    let family = match family {
        IconFamily::Phosphor => "phosphor",
        IconFamily::Tabler => "tabler",
    };
    let style = match style {
        IconStyle::Outline => "outline",
        IconStyle::Fill => "fill",
    };
    let lookup = |variant| {
        CATALOG
            .iter()
            .find(|(k, f, s, _)| *k == key && *f == family && *s == variant)
    };
    let Some((_, _, _, svg)) = lookup(style).or_else(|| lookup("outline")) else {
        return String::new();
    };
    // The tint is an application palette token, never document or user markup.
    let tint = if matches!(color, "#18212B" | "#F3F5F7" | "#FFFFFF") {
        color
    } else {
        "#F3F5F7"
    };
    let svg = svg.replace("currentColor", tint);
    format!(
        "data:image/svg+xml;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(svg)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_semantic_icon_resolves_in_both_families_and_styles() {
        for (key, _, _, _) in CATALOG {
            for family in [IconFamily::Phosphor, IconFamily::Tabler] {
                for style in [IconStyle::Outline, IconStyle::Fill] {
                    let url = source(key, family, style, "#18212B");
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(url.strip_prefix("data:image/svg+xml;base64,").unwrap())
                        .unwrap();
                    let svg = String::from_utf8(bytes).unwrap();
                    assert!(svg.contains("<svg"));
                    assert!(!svg.contains("currentColor"));
                }
            }
        }
        assert!(source(
            "unknown.command",
            IconFamily::Tabler,
            IconStyle::Fill,
            "#FFFFFF"
        )
        .is_empty());
    }
}
