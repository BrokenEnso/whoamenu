use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal};
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};

use clap::{Arg, ArgAction, Command};
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

    if options.show_help {
        println!("{}", CliOptions::usage_text());
        process::exit(0);
    }

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

    let viewport = egui::ViewportBuilder::default()
        .with_title("whoamenu")
        .with_decorations(false)
        .with_always_on_top()
        .with_inner_size([720.0, 320.0])
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.options.prompt).size(self.options.font_size as f32));

                let text_edit = egui::TextEdit::singleline(&mut self.query)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Body)
                    .hint_text("Type to filter...");
                let response = ui.add(text_edit);
                if response.changed() {
                    self.apply_filter();
                }
                response.request_focus();
            });

            if self.input_piped {
                ui.separator();
                let max_height =
                    (self.options.lines as f32) * (self.options.font_size as f32 * 1.8);
                ScrollArea::vertical()
                    .max_height(max_height)
                    .show(ui, |ui| {
                        for (index, item) in self.filtered_items.iter().enumerate() {
                            let selected = index == self.selected_index;
                            let response = ui.selectable_label(selected, item);
                            if response.clicked() {
                                self.selected_index = index;
                                self.accept_selection(ctx);
                                ctx.send_viewport_cmd(ViewportCommand::Close);
                            }
                        }
                    });
            }
        });
    }
}

#[derive(Clone, Debug)]
struct CliOptions {
    show_help: bool,
    clip: bool,
    prompt: String,
    case_sensitive: bool,
    font_size: i32,
    _font_name: Option<String>,
    _monitor: i32,
    _bottom: bool,
    _top: bool,
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
        let matches = Command::new("whoamenu")
            .disable_help_flag(true)
            .arg(Arg::new("help").short('h').action(ArgAction::SetTrue))
            .arg(
                Arg::new("clip")
                    .long("clip")
                    .visible_alias("clip")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("prompt").short('p').num_args(1))
            .arg(
                Arg::new("case-sensitive")
                    .long("case-sensitive")
                    .visible_alias("case-sensitive")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("font-size")
                    .long("font-size")
                    .visible_alias("font-size")
                    .num_args(1),
            )
            .arg(Arg::new("fn").long("fn").visible_alias("fn").num_args(1))
            .arg(Arg::new("m").short('m').num_args(1))
            .arg(Arg::new("b").short('b').action(ArgAction::SetTrue))
            .arg(Arg::new("t").short('t').action(ArgAction::SetTrue))
            .arg(Arg::new("l").short('l').num_args(1))
            .arg(
                Arg::new("rc")
                    .long("rc")
                    .visible_alias("rc")
                    .num_args(0..=1),
            )
            .arg(Arg::new("tr").long("tr").visible_alias("tr").num_args(1))
            .arg(Arg::new("nb").long("nb").visible_alias("nb").num_args(1))
            .arg(Arg::new("nf").long("nf").visible_alias("nf").num_args(1))
            .arg(Arg::new("sb").long("sb").visible_alias("sb").num_args(1))
            .arg(Arg::new("sf").long("sf").visible_alias("sf").num_args(1))
            .try_get_matches_from(
                std::iter::once("whoamenu").chain(args.iter().map(|s| s.as_str())),
            )
            .map_err(|e| e.to_string())?;

        Ok(Self {
            show_help: matches.get_flag("help"),
            clip: matches.get_flag("clip"),
            prompt: matches
                .get_one::<String>("prompt")
                .cloned()
                .unwrap_or_else(|| ">".to_string()),
            case_sensitive: matches.get_flag("case-sensitive"),
            font_size: matches
                .get_one::<String>("font-size")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(12),
            _font_name: matches.get_one::<String>("fn").cloned(),
            _monitor: matches
                .get_one::<String>("m")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1)
                - 1,
            _bottom: matches.get_flag("b"),
            _top: matches.get_flag("t"),
            lines: matches
                .get_one::<String>("l")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(10)
                .max(1),
            _corner_radius: matches
                .get_one::<String>("rc")
                .and_then(|s| s.parse::<f32>().ok())
                .map(|r| r.clamp(0.0, 30.0)),
            transparency: matches
                .get_one::<String>("tr")
                .and_then(|s| s.parse::<f32>().ok())
                .map(|t| t.clamp(0.0, 1.0)),
            normal_background: parse_color(matches.get_one::<String>("nb").map(String::as_str))?,
            normal_foreground: parse_color(matches.get_one::<String>("nf").map(String::as_str))?,
            selected_background: parse_color(matches.get_one::<String>("sb").map(String::as_str))?,
            selected_foreground: parse_color(matches.get_one::<String>("sf").map(String::as_str))?,
        })
    }

    fn usage_text() -> &'static str {
        "Usage: whoamenu [options]\n\n  -h                   Show help\n  -p <prompt>          Prompt text\n  -clip                Copy selected output to clipboard\n  -case-sensitive      Enable case-sensitive filtering\n  -font-size <size>    Set font size\n  -fn <font>           Set font family\n  -m <monitor>         Monitor number (1-based)\n  -b                   Place menu near the bottom\n  -t                   Place menu near the top\n  -l <lines>           Number of visible lines\n  -rc [radius]         Corner radius\n  -tr <opacity>        Window opacity (0..1)\n  -nb <color>          Normal background color\n  -nf <color>          Normal foreground color\n  -sb <color>          Selected background color\n  -sf <color>          Selected foreground color"
    }
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
