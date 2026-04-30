use std::fs;

use eframe::egui::{self, Color32, FontFamily};
use font_kit::handle::Handle;
use font_kit::source::SystemSource;

use crate::cli::CliOptions;

pub fn parse_color(input: Option<&str>) -> Result<Option<Color32>, String> {
    let Some(raw) = input else {
        return Ok(None);
    };

    let parsed = csscolorparser::parse(raw).map_err(|_| format!("Invalid color value: {raw}"))?;
    let [r, g, b, a] = parsed.to_rgba8();
    Ok(Some(Color32::from_rgba_unmultiplied(r, g, b, a)))
}

pub fn apply_opacity(color: Color32, opacity: f32) -> Color32 {
    let alpha = ((opacity.clamp(0.0, 1.0)) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn as_opaque(color: Color32) -> Color32 {
    Color32::from_rgb(color.r(), color.g(), color.b())
}

pub fn body_font_id(options: &CliOptions) -> egui::FontId {
    let family = options
        .font_name
        .as_deref()
        .map(|name| FontFamily::Name(name.into()))
        .unwrap_or(FontFamily::Proportional);
    egui::FontId::new(options.font_size as f32, family)
}

pub fn list_row_height(ctx: &egui::Context, options: &CliOptions) -> f32 {
    let font_id = body_font_id(options);
    let text_height = ctx.fonts(|fonts| fonts.row_height(&font_id));
    let min_interact_height = ctx.style().spacing.interact_size.y;
    let vertical_padding = ctx.style().spacing.button_padding.y * 2.0;
    (text_height + vertical_padding).max(min_interact_height)
}

pub fn find_matching_system_font_name(requested_name: &str) -> Option<String> {
    let normalized_requested = requested_name.trim().to_lowercase();
    if normalized_requested.is_empty() {
        return None;
    }

    let source = SystemSource::new();
    let families = source.all_families().ok()?;
    families
        .into_iter()
        .find(|family_name| family_name.trim().to_lowercase() == normalized_requested)
}

pub fn install_configured_font(ctx: &egui::Context, options: &CliOptions) {
    let Some(font_name) = options.font_name.as_deref() else {
        return;
    };

    let source = SystemSource::new();
    let Ok(handle) = source.select_family_by_name(font_name) else {
        eprintln!("Failed to open configured font family '{font_name}'");
        return;
    };
    let Some(font_bytes) = font_bytes_from_handle(handle.fonts().first()) else {
        eprintln!("Failed to load bytes for configured font family '{font_name}'");
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        font_name.to_string(),
        egui::FontData::from_owned(font_bytes).into(),
    );

    if let Some(proportional_family) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional_family.insert(0, font_name.to_string());
    }
    if let Some(monospace_family) = fonts.families.get_mut(&FontFamily::Monospace) {
        monospace_family.insert(0, font_name.to_string());
    }

    fonts.families.insert(
        FontFamily::Name(font_name.to_owned().into()),
        vec![font_name.to_string()],
    );
    ctx.set_fonts(fonts);
}

fn font_bytes_from_handle(handle: Option<&Handle>) -> Option<Vec<u8>> {
    match handle? {
        Handle::Path { path, .. } => fs::read(path).ok(),
        Handle::Memory { bytes, .. } => Some(bytes.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_color;
    use eframe::egui::Color32;

    #[test]
    fn parse_color_accepts_named_values() {
        let parsed = parse_color(Some("red")).expect("named color should parse");
        assert_eq!(parsed, Some(Color32::from_rgb(255, 0, 0)));
    }

    #[test]
    fn parse_color_accepts_hex_values() {
        let parsed = parse_color(Some("#00ff7f")).expect("hex color should parse");
        assert_eq!(parsed, Some(Color32::from_rgb(0, 255, 127)));
    }

    #[test]
    fn parse_color_returns_error_for_invalid_input() {
        let err = parse_color(Some("definitely-not-a-color")).expect_err("expected parse error");
        assert!(err.contains("Invalid color value"));
        assert!(err.contains("definitely-not-a-color"));
    }
}
