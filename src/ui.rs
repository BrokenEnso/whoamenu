use std::sync::{Arc, Mutex};

use eframe::egui::{self, RichText, ScrollArea, ViewportCommand};

use crate::cli::CliOptions;
use crate::monitor::{position_window, MonitorGeometry};
use crate::style::{
    apply_opacity, as_opaque, body_font_id, install_configured_font, list_row_height,
};

enum Action {
    Accept,
    Cancel,
}

#[derive(Default)]
pub struct SharedState {
    pub accepted: bool,
    pub result: String,
}

pub struct WhoaMenuApp {
    all_items: Vec<String>,
    all_items_normalized: Option<Vec<(String, String)>>,
    filtered_items: Vec<String>,
    query: String,
    query_lower: String,
    selected_index: usize,
    input_piped: bool,
    options: CliOptions,
    shared: Arc<Mutex<SharedState>>,
    last_window_height: f32,
    ensure_selected_visible: bool,
    monitor: Option<MonitorGeometry>,
}

impl WhoaMenuApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        all_items: Vec<String>,
        options: CliOptions,
        input_piped: bool,
        shared: Arc<Mutex<SharedState>>,
        monitor: Option<MonitorGeometry>,
    ) -> Self {
        install_configured_font(&cc.egui_ctx, &options);
        let opacity = options.transparency.unwrap_or(1.0);

        let mut style = (*cc.egui_ctx.style()).clone();
        style
            .text_styles
            .insert(egui::TextStyle::Body, body_font_id(&options));
        cc.egui_ctx.set_style(style);

        let mut visuals = egui::Visuals::dark();
        visuals.window_stroke = egui::Stroke::NONE;
        if let Some(bg) = options.normal_background {
            let fill = apply_opacity(bg, opacity);
            visuals.panel_fill = fill;
            visuals.window_fill = fill;
            visuals.extreme_bg_color = fill;
            visuals.faint_bg_color = fill;
        }
        if let Some(fg) = options.normal_foreground {
            visuals.override_text_color = Some(as_opaque(fg));
        }
        if let Some(sel_bg) = options.selected_background {
            visuals.selection.bg_fill = apply_opacity(sel_bg, opacity);
        }
        if let Some(sel_fg) = options.selected_foreground {
            visuals.selection.stroke.color = as_opaque(sel_fg);
        }
        visuals.widgets.noninteractive.bg_fill =
            apply_opacity(visuals.widgets.noninteractive.bg_fill, opacity);
        visuals.widgets.noninteractive.weak_bg_fill =
            apply_opacity(visuals.widgets.noninteractive.weak_bg_fill, opacity);
        visuals.widgets.inactive.bg_fill = apply_opacity(visuals.widgets.inactive.bg_fill, opacity);
        visuals.widgets.inactive.weak_bg_fill =
            apply_opacity(visuals.widgets.inactive.weak_bg_fill, opacity);
        visuals.widgets.hovered.bg_fill = apply_opacity(visuals.widgets.hovered.bg_fill, opacity);
        visuals.widgets.hovered.weak_bg_fill =
            apply_opacity(visuals.widgets.hovered.weak_bg_fill, opacity);
        visuals.widgets.active.bg_fill = apply_opacity(visuals.widgets.active.bg_fill, opacity);
        visuals.widgets.active.weak_bg_fill =
            apply_opacity(visuals.widgets.active.weak_bg_fill, opacity);
        visuals.widgets.open.bg_fill = apply_opacity(visuals.widgets.open.bg_fill, opacity);
        visuals.widgets.open.weak_bg_fill =
            apply_opacity(visuals.widgets.open.weak_bg_fill, opacity);
        cc.egui_ctx.set_visuals(visuals);
        if options.transparency.is_some() {
            eprintln!(
                "Transparency debug: tr={:.3}, nb_alpha={:?}, sb_alpha={:?}, panel_alpha={}, textedit_alpha={}",
                opacity,
                options.normal_background.map(|c| c.a()),
                options.selected_background.map(|c| c.a()),
                cc.egui_ctx.style().visuals.panel_fill.a(),
                cc.egui_ctx.style().visuals.widgets.inactive.bg_fill.a()
            );
        }

        let all_items_normalized = (!options.case_sensitive).then(|| {
            all_items
                .iter()
                .map(|item| (item.clone(), item.to_lowercase()))
                .collect()
        });

        let mut app = Self {
            all_items,
            all_items_normalized,
            filtered_items: Vec::new(),
            query: String::new(),
            query_lower: String::new(),
            selected_index: 0,
            input_piped,
            options,
            shared,
            last_window_height: 0.0,
            ensure_selected_visible: true,
            monitor,
        };
        app.apply_filter();
        app
    }

    fn apply_filter(&mut self) {
        self.filtered_items = if self.options.case_sensitive {
            self.all_items
                .iter()
                .filter(|item| self.matches(item, None))
                .cloned()
                .collect()
        } else {
            self.all_items_normalized
                .as_ref()
                .map(|items| {
                    items
                        .iter()
                        .filter(|(original, normalized)| self.matches(original, Some(normalized)))
                        .map(|(original, _)| original.clone())
                        .collect()
                })
                .unwrap_or_default()
        };

        if self.filtered_items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.filtered_items.len() {
            self.selected_index = self.filtered_items.len() - 1;
        }

        self.ensure_selected_visible = true;
    }

    fn matches(&self, item: &str, item_normalized: Option<&str>) -> bool {
        if self.options.case_sensitive {
            item.contains(&self.query)
        } else {
            item_normalized
                .unwrap_or(item)
                .contains(self.query_lower.as_str())
        }
    }

    fn refresh_query_cache(&mut self) {
        if self.options.case_sensitive {
            return;
        }
        self.query_lower = self.query.to_lowercase();
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

    fn handle_global_keys(&mut self, ctx: &egui::Context) -> Option<Action> {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return Some(Action::Cancel);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.move_selection(1);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.move_selection(-1);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            return Some(Action::Accept);
        }

        None
    }

    fn render_prompt_row(&mut self, ui: &mut egui::Ui) -> f32 {
        let prompt_row = ui.horizontal(|ui| {
            ui.label(RichText::new(&self.options.prompt).size(self.options.font_size as f32));

            let text_edit = egui::TextEdit::singleline(&mut self.query)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Body)
                .margin(egui::vec2(0.0, 0.0));
            let response = ui.add(text_edit);
            if response.changed() {
                self.refresh_query_cache();
                self.apply_filter();
            }
            response.request_focus();
        });

        prompt_row.response.rect.height()
    }

    fn render_list(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> f32 {
        if !self.input_piped {
            return 0.0;
        }

        let row_height = list_row_height(ctx, &self.options).ceil();
        let visible_rows = self.options.lines as usize;
        let row_spacing = self.options.vertical_spacing as f32;
        let visible_row_gaps = visible_rows.saturating_sub(1) as f32;
        let list_height =
            (visible_rows as f32 * row_height + visible_row_gaps * row_spacing).max(0.0);

        let list_container = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), list_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ScrollArea::vertical()
                    .max_height(list_height)
                    .show(ui, |ui| {
                        let last_index = self.filtered_items.len().saturating_sub(1);
                        for (index, item) in self.filtered_items.iter().enumerate() {
                            let selected = index == self.selected_index;
                            let row_width = ui.available_width();
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(row_width, row_height),
                                egui::Sense::click(),
                            );

                            if selected {
                                ui.painter()
                                    .rect_filled(rect, 0.0, ui.visuals().selection.bg_fill);
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
                                body_font_id(&self.options),
                                text_color,
                            );

                            if response.clicked() {
                                self.selected_index = index;
                                self.ensure_selected_visible = true;
                                self.accept_selection(ctx);
                                ctx.send_viewport_cmd(ViewportCommand::Close);
                            }

                            if row_spacing > 0.0 && index < last_index {
                                ui.add_space(row_spacing);
                            }
                        }
                    });
            },
        );

        self.ensure_selected_visible = false;
        list_container.response.rect.height()
    }

    fn resize_and_reposition(&mut self, ctx: &egui::Context, content_height: f32) {
        let target_height = content_height.ceil();
        if (target_height - self.last_window_height).abs() <= 0.5 {
            return;
        }

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
            self.monitor.as_ref(),
        );
    }
}

impl eframe::App for WhoaMenuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(action) = self.handle_global_keys(ctx) {
            match action {
                Action::Accept => self.accept_selection(ctx),
                Action::Cancel => self.cancel_selection(),
            }
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        let panel_frame = egui::Frame::default()
            .fill(ctx.style().visuals.panel_fill)
            .corner_radius(egui::CornerRadius::same(self.options.corner_radius_px()))
            .inner_margin(egui::Margin::same(0))
            .outer_margin(egui::Margin::same(0));
        let panel_vertical_margin = panel_frame.total_margin().sum().y;
        let mut panel_content_height = 0.0_f32;

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                let prompt_row_height = self.render_prompt_row(ui);
                let list_height = self.render_list(ui, ctx);
                panel_content_height = prompt_row_height + list_height;
            });

        self.resize_and_reposition(ctx, panel_content_height + panel_vertical_margin + 2.0);
    }
}
