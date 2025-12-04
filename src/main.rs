//! Iced Builder - A visual GUI builder for Iced applications.
//!
//! This application allows developers to visually design and modify
//! Iced UI layouts and export them as Rust code.

#![windows_subsystem = "windows"]

mod app;
mod codegen;
mod io;
mod logging;
mod model;
mod ui;
mod util;

use app::App;
use iced::Size;

fn main() -> iced::Result {
    // Initialize logging system first
    logging::init();

    tracing::info!("Starting Iced Builder");

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .window_size(Size::new(1280.0, 800.0))
        .run()
}
