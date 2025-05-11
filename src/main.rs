// src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

mod app;         // << 取消註解 - 聲明 app 模組
mod dep_check;   // << 取消註解 - 聲明 dep_check 模組

use app::CsvEncodingApp; // << 導入 CsvEncodingApp

fn main() -> Result<(), eframe::Error> {
    // env_logger::init();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(
                // 確保 ../assets/icon.png 存在，或者移除這部分
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                .unwrap_or_default()
            ),
        ..Default::default()
    };

    eframe::run_native(
        "CSV Encoding Orchestrator",
        options,
        Box::new(|cc| {
            Box::new(CsvEncodingApp::new(cc)) // << 這裡需要 CsvEncodingApp
        }),
    )
}