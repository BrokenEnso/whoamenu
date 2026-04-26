use clap::Parser;
use eframe::egui::Color32;

use crate::style::{find_matching_system_font_name, parse_color};

#[derive(Clone, Debug)]
pub struct CliOptions {
    pub clip: bool,
    pub prompt: String,
    pub case_sensitive: bool,
    pub font_size: i32,
    pub font_name: Option<String>,
    pub monitor: usize,
    pub bottom: bool,
    pub top: bool,
    pub lines: i32,
    pub vertical_spacing: i32,
    pub corner_radius: Option<f32>,
    pub transparency: Option<f32>,
    pub normal_background: Option<Color32>,
    pub normal_foreground: Option<Color32>,
    pub selected_background: Option<Color32>,
    pub selected_foreground: Option<Color32>,
}

impl CliOptions {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let normalized_args = normalize_legacy_flags(args);
        let cli_args =
            CliArgs::try_parse_from(std::iter::once("whoamenu".to_string()).chain(normalized_args))
                .map_err(|e| e.to_string())?;

        if cli_args.bottom && cli_args.top {
            return Err(
                "Invalid flags: `-b` (bottom) and `-t` (top) are mutually exclusive. \
Choose one placement flag only (for example: `whoamenu -b` or `whoamenu -t`)."
                    .to_string(),
            );
        }

        Ok(Self {
            clip: cli_args.clip,
            prompt: cli_args.prompt,
            case_sensitive: cli_args.case_sensitive,
            font_size: cli_args.font_size,
            font_name: cli_args.font_name,
            monitor: clamp_monitor_index(cli_args.monitor),
            bottom: cli_args.bottom,
            top: cli_args.top,
            lines: clamp_lines(cli_args.lines),
            vertical_spacing: clamp_vertical_spacing(cli_args.vertical_spacing),
            corner_radius: clamp_corner_radius(cli_args.corner_radius),
            transparency: clamp_transparency(cli_args.transparency),
            normal_background: parse_color(cli_args.normal_background.as_deref())?,
            normal_foreground: parse_color(cli_args.normal_foreground.as_deref())?,
            selected_background: parse_color(cli_args.selected_background.as_deref())?,
            selected_foreground: parse_color(cli_args.selected_foreground.as_deref())?,
        })
    }

    pub fn corner_radius_px(&self) -> u8 {
        self.corner_radius
            .map(|r| r.clamp(0.0, u8::MAX as f32).round() as u8)
            .unwrap_or(0)
    }

    pub fn resolve_font_name(&mut self) {
        let Some(requested_name) = self.font_name.clone() else {
            return;
        };

        match find_matching_system_font_name(&requested_name) {
            Some(matched) => self.font_name = Some(matched),
            None => {
                eprintln!(
                    "Configured font '{requested_name}' was not found in system fonts; using default font"
                );
                self.font_name = None;
            }
        }
    }
}

#[derive(Clone, Debug, Parser)]
#[command(name = "whoamenu", args_override_self = true, ignore_errors = true)]
pub struct CliArgs {
    /// Copy selected output to clipboard
    #[arg(long)]
    clip: bool,

    /// Prompt text
    #[arg(short = 'p', default_value = ">")]
    prompt: String,

    /// Enable case-sensitive filtering
    #[arg(long = "case-sensitive")]
    case_sensitive: bool,

    /// Set font size
    #[arg(long = "font-size", default_value_t = 12)]
    font_size: i32,

    /// Set font family
    #[arg(long = "fn")]
    font_name: Option<String>,

    /// Monitor number (1-based)
    #[arg(short = 'm', default_value_t = 1)]
    monitor: i32,

    /// Place menu near the bottom
    #[arg(short = 'b', conflicts_with = "top")]
    bottom: bool,

    /// Place menu near the top
    #[arg(short = 't', conflicts_with = "bottom")]
    top: bool,

    /// Number of visible lines
    #[arg(short = 'l', default_value_t = 10)]
    lines: i32,

    /// Vertical space between list items
    #[arg(long = "vs", default_value_t = 0)]
    vertical_spacing: i32,

    /// Corner radius
    #[arg(long = "rc")]
    corner_radius: Option<f32>,

    /// Window opacity (0..1)
    #[arg(long = "tr")]
    transparency: Option<f32>,

    /// Normal background color
    #[arg(long = "nb")]
    normal_background: Option<String>,

    /// Normal foreground color
    #[arg(long = "nf")]
    normal_foreground: Option<String>,

    /// Selected background color
    #[arg(long = "sb")]
    selected_background: Option<String>,

    /// Selected foreground color
    #[arg(long = "sf")]
    selected_foreground: Option<String>,
}

pub fn normalize_legacy_flags(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| match arg.as_str() {
            "-clip" => "--clip".to_string(),
            "-case-sensitive" => "--case-sensitive".to_string(),
            "-font-size" => "--font-size".to_string(),
            "-fn" => "--fn".to_string(),
            "-rc" => "--rc".to_string(),
            "-tr" => "--tr".to_string(),
            "-nb" => "--nb".to_string(),
            "-nf" => "--nf".to_string(),
            "-sb" => "--sb".to_string(),
            "-sf" => "--sf".to_string(),
            "-vs" => "--vs".to_string(),
            _ => arg.clone(),
        })
        .collect()
}

pub fn clamp_monitor_index(monitor: i32) -> usize {
    monitor.saturating_sub(1) as usize
}

pub fn clamp_lines(lines: i32) -> i32 {
    lines.max(1)
}

pub fn clamp_vertical_spacing(vertical_spacing: i32) -> i32 {
    vertical_spacing.max(0)
}

pub fn clamp_corner_radius(corner_radius: Option<f32>) -> Option<f32> {
    corner_radius.map(|radius| radius.clamp(0.0, 30.0))
}

pub fn clamp_transparency(transparency: Option<f32>) -> Option<f32> {
    transparency.map(|opacity| opacity.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::{normalize_legacy_flags, CliOptions};

    #[test]
    fn parse_rejects_bottom_and_top_together() {
        let args = vec!["-b".to_string(), "-t".to_string()];
        let err =
            CliOptions::parse(&args).expect_err("expected conflicting placement flags to fail");

        assert!(err.contains("mutually exclusive"));
        assert!(err.contains("`-b`"));
        assert!(err.contains("`-t`"));
    }

    #[test]
    fn normalize_legacy_flags_converts_known_legacy_flags() {
        let args = vec![
            "-clip".to_string(),
            "-font-size".to_string(),
            "12".to_string(),
            "-vs".to_string(),
            "2".to_string(),
        ];

        let normalized = normalize_legacy_flags(&args);
        assert_eq!(
            normalized,
            vec!["--clip", "--font-size", "12", "--vs", "2"]
        );
    }

    #[test]
    fn normalize_legacy_flags_passes_through_unknown_flags_and_values() {
        let args = vec![
            "--prompt".to_string(),
            "hello".to_string(),
            "-x".to_string(),
            "custom".to_string(),
        ];

        let normalized = normalize_legacy_flags(&args);
        assert_eq!(normalized, args);
    }
}
