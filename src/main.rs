/*
 Main entry

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
 9/18/2024 3:34:10 PM
*/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use egui::ViewportBuilder;
use eframe::Theme;
use eframe::egui;
mod application;
use application::*;
mod vysisxml;
mod vysyslib;
mod vysis;
mod vysyslibxml;
mod outline;
mod graphs;
mod project;
mod bfs;
mod xlsxtable;
mod wire_list_xlsx_formatter;
mod flexible_table;
mod table_convert;
mod wire_list_grid;
mod wirelist;
mod deviceindex;
mod traverse;
mod harness_commands;
mod shchleuniger;
mod vesys_table_reader;
mod logic_commands;


fn main() {
    let mut native_options = eframe::NativeOptions::default();
    //native_options.initial_window_size = Some(egui::vec2(450.0, 800.0));
    native_options.viewport = ViewportBuilder::default().with_inner_size(egui::vec2(1280.0, 720.0));
    native_options.default_theme = Theme::Dark;
    native_options.follow_system_theme = false;

    eframe::run_native(
        "Vesys project post-processor",
        native_options,
        Box::new(|cc| Box::new(Application::new(cc))),
    );
}

