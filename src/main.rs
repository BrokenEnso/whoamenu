use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal};
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};

use clap::Parser;
use eframe::egui::{self, Color32, RichText, ScrollArea, ViewportCommand};

fn main() {
    let config_args = read_config_args();
    let cli_args = env::args().skip(1).collect::<Vec<_>>();
    let merged_args = config_args
        .into_iter()
        .chain(cli_args)
        .collect::<Vec<String>>();

    let options = match CliOptions::parse(&merged_args) {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    let input_piped = !io::stdin().is_terminal();
    let items = if input_piped {
        read_items(io::stdin().lock())
    } else {
        Vec::new()
    };

    if input_piped && items.is_empty() {
        eprintln!("No items provided");
        process::exit(1);
    }

    let state = Arc::new(Mutex::new(SharedState::default()));
    let app_state = Arc::clone(&state);

    let initial_width = 720.0;
    let initial_height = 320.0;

    let viewport = egui::ViewportBuilder::default()
        .with_title("whoamenu")
        .with_decorations(false)
        .with_always_on_top()
        .with_inner_size([initial_width, initial_height])
        .with_transparent(options.transparency.map(|v| v < 1.0).unwrap_or(false));

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let app_options = options.clone();
    let run_result = eframe::run_native(
        "whoamenu",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(WhoaMenuApp::new(
                cc,
                items,
                app_options,
                input_piped,
                app_state,
            )))
        }),
    );

    if let Err(err) = run_result {
        eprintln!("Failed to start UI: {err}");
        process::exit(1);
    }

    let final_state = state.lock().expect("shared state poisoned");
    if final_state.accepted && !final_state.result.trim().is_empty() {
        println!("{}", final_state.result);
        process::exit(0);
    }

    process::exit(1);
}

#[derive(Default)]
struct SharedState {
    accepted: bool,
    result: String,
}

struct WhoaMenuApp {
    all_items: Vec<String>,
    filtered_items: Vec<String>,
    query: String,
    selected_index: usize,
    input_piped: bool,
    options: CliOptions,
    shared: Arc<Mutex<SharedState>>,
    last_window_height: f32,
    ensure_selected_visible: bool,
}

impl WhoaMenuApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        all_items: Vec<String>,
        options: CliOptions,
        input_piped: bool,
        shared: Arc<Mutex<SharedState>>,
    ) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(options.font_size as f32),
        );
        cc.egui_ctx.set_style(style);

        let mut visuals = egui::Visuals::dark();
        visuals.window_stroke = egui::Stroke::NONE;
        if let Some(bg) = options.normal_background {
            let fill = apply_opacity(bg, options.transparency.unwrap_or(1.0));
            visuals.panel_fill = fill;
            visuals.window_fill = fill;
            visuals.extreme_bg_color = fill;
            visuals.faint_bg_color = fill;
        }
        if let Some(fg) = options.normal_foreground {
            visuals.override_text_color = Some(as_opaque(fg));
        }
        if let Some(sel_bg) = options.selected_background {
            visuals.selection.bg_fill = apply_opacity(sel_bg, options.transparency.unwrap_or(1.0));
        }
        if let Some(sel_fg) = options.selected_foreground {
            visuals.selection.stroke.color = as_opaque(sel_fg);
        }
        cc.egui_ctx.set_visuals(visuals);

        let mut app = Self {
            all_items,
            filtered_items: Vec::new(),
            query: String::new(),
            selected_index: 0,
            input_piped,
            options,
            shared,
            last_window_height: 0.0,
            ensure_selected_visible: true,
        };
        app.apply_filter();
        app
    }

    fn apply_filter(&mut self) {
        self.filtered_items = self
            .all_items
            .iter()
            .filter(|item| self.matches(item))
            .cloned()
            .collect();

        if self.filtered_items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.filtered_items.len() {
            self.selected_index = self.filtered_items.len() - 1;
        }

        self.ensure_selected_visible = true;
    }

    fn matches(&self, item: &str) -> bool {
        if self.options.case_sensitive {
            item.contains(&self.query)
        } else {
            item.to_lowercase().contains(&self.query.to_lowercase())
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_items.is_empty() {
            return;
        }

        let current = self.selected_index as i32;
        let max_index = (self.filtered_items.len() - 1) as i32;
        let next = (current + delta).clamp(0, max_index);
        self.selected_index = next as usize;
        self.ensure_selected_visible = true;
    }

    fn accept_selection(&self, ctx: &egui::Context) {
        let selected = self
            .filtered_items
            .get(self.selected_index)
            .cloned()
            .unwrap_or_else(|| self.query.trim().to_string());

        let accepted = !selected.trim().is_empty();
        if accepted && self.options.clip {
            ctx.copy_text(selected.clone());
        }

        let mut state = self.shared.lock().expect("shared state poisoned");
        state.accepted = accepted;
        state.result = selected;
    }

    fn cancel_selection(&self) {
        let mut state = self.shared.lock().expect("shared state poisoned");
        state.accepted = false;
        state.result.clear();
    }
}

impl eframe::App for WhoaMenuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.cancel_selection();
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.move_selection(1);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.move_selection(-1);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.accept_selection(ctx);
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        let panel_frame = egui::Frame::default()
            .fill(ctx.style().visuals.panel_fill)
            .inner_margin(egui::Margin::same(0))
            .outer_margin(egui::Margin::same(0));

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.label(
                        RichText::new(&self.options.prompt).size(self.options.font_size as f32),
                    );

                    let text_edit = egui::TextEdit::singleline(&mut self.query)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Body)
                        .margin(egui::vec2(0.0, 0.0))
                        .hint_text("Type to filter...");
                    let response = ui.add(text_edit);
                    if response.changed() {
                        self.apply_filter();
                    }
                    response.request_focus();
                });

                if self.input_piped {
                    let row_height = ui.spacing().interact_size.y;
                    let list_height = row_height * self.options.lines as f32;

                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), list_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ScrollArea::vertical()
                                .max_height(list_height)
                                .show(ui, |ui| {
                                    for (index, item) in self.filtered_items.iter().enumerate() {
                                        let selected = index == self.selected_index;
                                        let row_width = ui.available_width();
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(row_width, row_height),
                                            egui::Sense::click(),
                                        );

                                        if selected {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                ui.visuals().selection.bg_fill,
                                            );
                                            if self.ensure_selected_visible {
                                                ui.scroll_to_rect(rect, None);
                                            }
                                        }

                                        let text_color = if selected {
                                            ui.visuals().selection.stroke.color
                                        } else {
                                            ui.visuals().text_color()
                                        };

                                        ui.painter().text(
                                            egui::pos2(rect.left(), rect.center().y),
                                            egui::Align2::LEFT_CENTER,
                                            item,
                                            egui::FontId::proportional(
                                                self.options.font_size as f32,
                                            ),
                                            text_color,
                                        );

                                        if response.clicked() {
                                            self.selected_index = index;
                                            self.ensure_selected_visible = true;
                                            self.accept_selection(ctx);
                                            ctx.send_viewport_cmd(ViewportCommand::Close);
                                        }
                                    }
                                });
                        },
                    );
                    self.ensure_selected_visible = false;
                }
            });

        let row_height = ctx.style().spacing.interact_size.y;
        let list_height = if self.input_piped {
            row_height * self.options.lines as f32
        } else {
            0.0
        };
        let text_height = row_height;
        let frame_padding = 0.0;
        let target_height = text_height + list_height + frame_padding;

        if (target_height - self.last_window_height).abs() > 0.5 {
            self.last_window_height = target_height;
            let viewport_width = ctx.screen_rect().width().max(720.0);
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(
                viewport_width,
                target_height,
            )));
            position_window(
                ctx,
                viewport_width,
                target_height,
                self.options.bottom,
                self.options.top,
            );
        }
    }
}

fn position_window(
    ctx: &egui::Context,
    width: f32,
    height: f32,
    bottom_align: bool,
    top_align: bool,
) {
    if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
        let centered_x = ((monitor_size.x - width) * 0.5).max(0.0);
        let target_y = if top_align {
            0.0
        } else if bottom_align {
            (monitor_size.y - height).max(0.0)
        } else {
            ((monitor_size.y - height) * 0.5).max(0.0)
        };
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(
            centered_x, target_y,
        )));
    }
}

#[derive(Clone, Debug)]
struct CliOptions {
    clip: bool,
    prompt: String,
    case_sensitive: bool,
    font_size: i32,
    _font_name: Option<String>,
    _monitor: i32,
    bottom: bool,
    top: bool,
    lines: i32,
    _corner_radius: Option<f32>,
    transparency: Option<f32>,
    normal_background: Option<Color32>,
    normal_foreground: Option<Color32>,
    selected_background: Option<Color32>,
    selected_foreground: Option<Color32>,
}

impl CliOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let normalized_args = normalize_legacy_flags(args);
        let cli_args =
            CliArgs::try_parse_from(std::iter::once("whoamenu".to_string()).chain(normalized_args))
                .map_err(|e| e.to_string())?;

        Ok(Self {
            clip: cli_args.clip,
            prompt: cli_args.prompt,
            case_sensitive: cli_args.case_sensitive,
            font_size: cli_args.font_size,
            _font_name: cli_args.font_name,
            _monitor: cli_args.monitor - 1,
            bottom: cli_args.bottom,
            top: cli_args.top,
            lines: cli_args.lines.max(1),
            _corner_radius: cli_args.corner_radius.map(|r| r.clamp(0.0, 30.0)),
            transparency: cli_args.transparency.map(|t| t.clamp(0.0, 1.0)),
            normal_background: parse_color(cli_args.normal_background.as_deref())?,
            normal_foreground: parse_color(cli_args.normal_foreground.as_deref())?,
            selected_background: parse_color(cli_args.selected_background.as_deref())?,
            selected_foreground: parse_color(cli_args.selected_foreground.as_deref())?,
        })
    }
}

#[derive(Clone, Debug, Parser)]
#[command(name = "whoamenu", args_override_self = true, ignore_errors = true)]
struct CliArgs {
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
    #[arg(short = 'b')]
    bottom: bool,

    /// Place menu near the top
    #[arg(short = 't')]
    top: bool,

    /// Number of visible lines
    #[arg(short = 'l', default_value_t = 10)]
    lines: i32,

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

fn normalize_legacy_flags(args: &[String]) -> Vec<String> {
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
            _ => arg.clone(),
        })
        .collect()
}

fn parse_color(input: Option<&str>) -> Result<Option<Color32>, String> {
    let Some(raw) = input else {
        return Ok(None);
    };

    let parsed = csscolorparser::parse(raw).map_err(|_| format!("Invalid color value: {raw}"))?;
    let [r, g, b, a] = parsed.to_rgba8();
    Ok(Some(Color32::from_rgba_unmultiplied(r, g, b, a)))
}

fn apply_opacity(color: Color32, opacity: f32) -> Color32 {
    let alpha = ((opacity.clamp(0.0, 1.0)) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn as_opaque(color: Color32) -> Color32 {
    Color32::from_rgb(color.r(), color.g(), color.b())
}

fn read_items<R: BufRead>(reader: R) -> Vec<String> {
    reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn read_config_args() -> Vec<String> {
    let path = match config_path() {
        Some(path) => path,
        None => return Vec::new(),
    };

    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .flat_map(tokenize_config_line)
        .collect()
}

fn config_path() -> Option<PathBuf> {
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        if !xdg_config_home.trim().is_empty() {
            return Some(
                PathBuf::from(xdg_config_home)
                    .join("whoamenu")
                    .join("config"),
            );
        }
    }

    let home = env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("whoamenu")
            .join("config"),
    )
}

fn tokenize_config_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            c => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
