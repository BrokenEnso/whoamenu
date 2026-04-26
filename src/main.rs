mod cli;
mod config;
mod monitor;
mod style;
mod ui;

use std::env;
use std::io::{self, BufRead, IsTerminal};
use std::process;
use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::cli::CliOptions;
use crate::config::read_config_args;
use crate::monitor::{detect_monitor, window_position_for_monitor};
use crate::ui::{SharedState, WhoaMenuApp};

fn main() {
    let config_args = read_config_args();
    let cli_args = env::args().skip(1).collect::<Vec<_>>();
    let merged_args = config_args
        .into_iter()
        .chain(cli_args)
        .collect::<Vec<String>>();

    let mut options = match CliOptions::parse(&merged_args) {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };
    options.resolve_font_name();

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
    let monitor = detect_monitor(options.monitor);
    let initial_position = window_position_for_monitor(
        monitor.as_ref(),
        initial_width,
        initial_height,
        options.bottom,
        options.top,
    );

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("whoamenu")
        .with_decorations(false)
        .with_always_on_top()
        .with_inner_size([initial_width, initial_height])
        .with_transparent(
            options.transparency.map(|v| v < 1.0).unwrap_or(false)
                || options.corner_radius.unwrap_or(0.0) > 0.0,
        );
    if let Some(position) = initial_position {
        viewport = viewport.with_position(position);
    }

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
                monitor,
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

fn read_items<R: BufRead>(reader: R) -> Vec<String> {
    reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}
