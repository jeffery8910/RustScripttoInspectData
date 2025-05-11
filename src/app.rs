// src/app.rs
use crate::dep_check::{Dependencies, DependencyStatus, CHECKED_DEPS};
use duct::cmd;
use eframe::egui;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

// Constants
const R_SCRIPT_NAME: &str = "check_csv_dimensions_r.R";
const PYTHON_SCRIPT_NAME: &str = "check_csv_dimensions_py.py";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptOutput {
    tool: String,
    file_path: String,
    encoding_tested: String,
    status: String,
    rows: Option<i64>,
    cols: Option<i64>,
    cells: Option<i64>,
    error_message: Option<String>,
}

#[derive(Debug)]
enum AppMessage {
    ProcessingStarted,
    NewResult(ScriptOutput),
    ProcessingFinished(Vec<ScriptOutput>),
    ProcessingError(String),
    UpdateProgress(String),
}

pub struct CsvEncodingApp {
    csv_file_path: Option<PathBuf>,
    encodings_input: String,
    results: Vec<ScriptOutput>,
    processing_status: String,
    is_processing: bool,
    dependencies: &'static Dependencies,
    r_script_path: PathBuf,
    python_script_path: PathBuf,
    script_paths_ok: bool,
    sender: Sender<AppMessage>,
    receiver: Receiver<AppMessage>,
    progress_log: Vec<String>,
}

impl CsvEncodingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = channel();

        let base_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let r_script_path = base_path.join("scripts/R").join(R_SCRIPT_NAME);
        let python_script_path = base_path.join("scripts/python").join(PYTHON_SCRIPT_NAME);

        let mut script_paths_ok = true;
        let mut initial_progress = Vec::new();

        if !r_script_path.exists() {
            let msg = format!("Error: R script not found at {:?}", r_script_path);
            eprintln!("{}", msg);
            initial_progress.push(msg);
            script_paths_ok = false;
        }
        if !python_script_path.exists() {
            let msg = format!("Error: Python script not found at {:?}", python_script_path);
            eprintln!("{}", msg);
            initial_progress.push(msg);
            script_paths_ok = false;
        }

        Self {
            csv_file_path: None,
            encodings_input: "UTF-8,BIG5".to_string(),
            results: Vec::new(),
            processing_status: "Idle".to_string(),
            is_processing: false,
            dependencies: &CHECKED_DEPS,
            r_script_path,
            python_script_path,
            script_paths_ok,
            sender,
            receiver,
            progress_log: initial_progress,
        }
    }

    fn run_checks(&mut self) {
        if self.is_processing {
            return;
        }
        if self.csv_file_path.is_none() {
            self.processing_status = "Error: Please select a CSV file.".to_string();
            self.progress_log.push(self.processing_status.clone());
            return;
        }
        if !self.dependencies.all_ok() || !self.script_paths_ok {
            self.processing_status =
                "Error: Dependencies or script paths not met. Check 'System Dependencies' section."
                    .to_string();
            self.progress_log.push(self.processing_status.clone());
            return;
        }

        self.is_processing = true;
        self.processing_status = "Processing...".to_string();
        self.results.clear();
        self.progress_log.clear();
        self.progress_log.push("Starting checks...".to_string());

        let csv_file = self.csv_file_path.clone().unwrap();
        let encodings: Vec<String> = self
            .encodings_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if encodings.is_empty() {
            self.processing_status = "Error: Please enter at least one encoding.".to_string();
            self.progress_log.push(self.processing_status.clone());
            self.is_processing = false;
            return;
        }

        let r_executable = self
            .dependencies
            .r_script
            .get_path()
            .expect("Rscript path checked")
            .clone();
        let python_executable = self
            .dependencies
            .python_interpreter
            .get_path()
            .expect("Python path checked")
            .clone();
        let r_script_path_clone = self.r_script_path.clone();
        let python_script_path_clone = self.python_script_path.clone();
        let sender_clone = self.sender.clone();

        thread::spawn(move || {
            let _ = sender_clone.send(AppMessage::ProcessingStarted);
            let mut all_script_outputs: Vec<ScriptOutput> = Vec::new();

            if !csv_file.exists() || !csv_file.is_file() {
                let _ = sender_clone.send(AppMessage::ProcessingError(format!(
                    "CSV file not found: {:?}",
                    csv_file
                )));
                return;
            }
            let csv_file_abs_path = match csv_file.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    let _ = sender_clone.send(AppMessage::ProcessingError(format!(
                        "Error getting absolute path for CSV: {}",
                        e
                    )));
                    return;
                }
            };

            for encoding in &encodings {
                let _ = sender_clone.send(AppMessage::UpdateProgress(format!(
                    "\nTesting with encoding: {}",
                    encoding
                )));

                // --- Execute R Script ---
                let r_output_temp_file = match tempfile::Builder::new()
                    .prefix("r_gui_")
                    .suffix(".csv")
                    .tempfile()
                {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = sender_clone.send(AppMessage::ProcessingError(format!(
                            "Failed to create R temp file: {}",
                            e
                        )));
                        continue;
                    }
                };
                let r_output_path_string = match r_output_temp_file.path().to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        let _ = sender_clone.send(AppMessage::ProcessingError(
                            "R temp file path invalid UTF-8".to_string(),
                        ));
                        continue;
                    }
                };

                let _ = sender_clone.send(AppMessage::UpdateProgress(format!(
                    "  Running R script for encoding: {}...",
                    encoding
                )));
                let r_cmd_output = cmd!(
                    &r_executable,
                    &r_script_path_clone,
                    &csv_file_abs_path,
                    &r_output_path_string,
                    encoding
                )
                .stderr_to_stdout()
                .stdout_capture()
                .run();

                match r_cmd_output {
                    Ok(output) => {
                        let stdout_str = String::from_utf8_lossy(&output.stdout);
                        if output.status.success() {
                            let _ = sender_clone.send(AppMessage::UpdateProgress(format!(
                                "    R script finished successfully for {}.",
                                encoding
                            )));
                            match parse_script_output_csv_from_app(r_output_temp_file.path()) {
                                Ok(r_results) => {
                                    for res in r_results {
                                        all_script_outputs.push(res.clone());
                                        let _ = sender_clone.send(AppMessage::NewResult(res));
                                    }
                                }
                                Err(e) => {
                                    let err_msg = format!("    Error parsing R script output for {}: {}. R script stdout: {}", encoding, e, stdout_str);
                                    let _ = sender_clone
                                        .send(AppMessage::UpdateProgress(err_msg.clone()));
                                    all_script_outputs.push(create_error_output(
                                        "R_Orchestrator",
                                        &csv_file,
                                        encoding,
                                        "OutputParseFailure",
                                        &err_msg,
                                    ));
                                }
                            }
                        } else {
                            let err_msg = format!(
                                "    R script failed for {} with status: {:?}. Output:\n{}",
                                encoding, output.status, stdout_str
                            );
                            let _ = sender_clone.send(AppMessage::UpdateProgress(err_msg.clone()));
                            all_script_outputs.push(create_error_output(
                                "R_Overall",
                                &csv_file,
                                encoding,
                                "ExecutionFailure",
                                &err_msg,
                            ));
                        }
                    }
                    Err(e) => {
                        let err_msg =
                            format!("    Failed to execute R script for {}: {}", encoding, e);
                        let _ = sender_clone.send(AppMessage::UpdateProgress(err_msg.clone()));
                        all_script_outputs.push(create_error_output(
                            "R_Overall",
                            &csv_file,
                            encoding,
                            "ExecutionError",
                            &err_msg,
                        ));
                    }
                }

                // --- Execute Python Script ---
                let py_output_temp_file = match tempfile::Builder::new()
                    .prefix("py_gui_")
                    .suffix(".csv")
                    .tempfile()
                {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = sender_clone.send(AppMessage::ProcessingError(format!(
                            "Failed to create Py temp file: {}",
                            e
                        )));
                        continue;
                    }
                };
                let py_output_path_string = match py_output_temp_file.path().to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        let _ = sender_clone.send(AppMessage::ProcessingError(
                            "Py temp file path invalid UTF-8".to_string(),
                        ));
                        continue;
                    }
                };

                let _ = sender_clone.send(AppMessage::UpdateProgress(format!(
                    "  Running Python script for encoding: {}...",
                    encoding
                )));
                let py_cmd_output = cmd!(
                    &python_executable,
                    &python_script_path_clone,
                    &csv_file_abs_path,
                    &py_output_path_string,
                    encoding
                )
                .stderr_to_stdout()
                .stdout_capture()
                .run();

                match py_cmd_output {
                    Ok(output) => {
                        let stdout_str = String::from_utf8_lossy(&output.stdout);
                        if output.status.success() {
                            let _ = sender_clone.send(AppMessage::UpdateProgress(format!(
                                "    Python script finished successfully for {}.",
                                encoding
                            )));
                            match parse_script_output_csv_from_app(py_output_temp_file.path()) {
                                Ok(py_results) => {
                                    for res in py_results {
                                        all_script_outputs.push(res.clone());
                                        let _ = sender_clone.send(AppMessage::NewResult(res));
                                    }
                                }
                                Err(e) => {
                                    let err_msg = format!("    Error parsing Python script output for {}: {}. Python script stdout: {}", encoding, e, stdout_str);
                                    let _ = sender_clone
                                        .send(AppMessage::UpdateProgress(err_msg.clone()));
                                    all_script_outputs.push(create_error_output(
                                        "Python_Orchestrator",
                                        &csv_file,
                                        encoding,
                                        "OutputParseFailure",
                                        &err_msg,
                                    ));
                                }
                            }
                        } else {
                            let err_msg = format!(
                                "    Python script failed for {} with status: {:?}. Output:\n{}",
                                encoding, output.status, stdout_str
                            );
                            let _ = sender_clone.send(AppMessage::UpdateProgress(err_msg.clone()));
                            all_script_outputs.push(create_error_output(
                                "Python_Overall",
                                &csv_file,
                                encoding,
                                "ExecutionFailure",
                                &err_msg,
                            ));
                        }
                    }
                    Err(e) => {
                        let err_msg = format!(
                            "    Failed to execute Python script for {}: {}",
                            encoding, e
                        );
                        let _ = sender_clone.send(AppMessage::UpdateProgress(err_msg.clone()));
                        all_script_outputs.push(create_error_output(
                            "Python_Overall",
                            &csv_file,
                            encoding,
                            "ExecutionError",
                            &err_msg,
                        ));
                    }
                }
            }
            let _ = sender_clone.send(AppMessage::ProcessingFinished(all_script_outputs));
        });
    }

    fn render_dependencies(&self, ui: &mut egui::Ui) {
        ui.collapsing("System Dependencies", |ui| {
            ui.label("This tool relies on R and Python being installed and configured correctly.");
            ui.add_space(5.0);
            render_dep_status_gui(ui, "Rscript", &self.dependencies.r_script);
            if self.dependencies.r_script.is_ok() {
                render_dep_status_gui(ui, "  - R package 'readr'", &self.dependencies.r_readr_pkg);
                render_dep_status_gui(ui, "  - R package 'dplyr'", &self.dependencies.r_dplyr_pkg);
            }
            ui.separator();
            render_dep_status_gui(ui, "Python (python/python3)", &self.dependencies.python_interpreter);
            if self.dependencies.python_interpreter.is_ok() {
                render_dep_status_gui(ui, "  - Python package 'pandas'", &self.dependencies.py_pandas_pkg);
                render_dep_status_gui(ui, "  - Python package 'duckdb'", &self.dependencies.py_duckdb_pkg);
            }
             ui.separator();
            if self.script_paths_ok {
                ui.colored_label(egui::Color32::GREEN, "✓ Helper scripts (R/Python) found in expected locations.");
                 ui.label(format!("  R script: {:?}", self.r_script_path));
                 ui.label(format!("  Python script: {:?}", self.python_script_path));
            } else {
                ui.colored_label(egui::Color32::RED, "✗ Error: Helper scripts (R/Python) not found in expected locations relative to executable.");
                 if !self.r_script_path.exists() { ui.label(format!("  Missing R script at: {:?}", self.r_script_path)); }
                 if !self.python_script_path.exists() { ui.label(format!("  Missing Python script at: {:?}", self.python_script_path)); }
            }
        });
    }
}

fn render_dep_status_gui(ui: &mut egui::Ui, name: &str, status: &DependencyStatus) {
    ui.horizontal(|ui| match status {
        DependencyStatus::Ok(path) => {
            ui.colored_label(egui::Color32::GREEN, "✓");
            ui.label(format!("{}: Found", name));
            if name == "Rscript" || name == "Python (python/python3)" {
                ui.weak(format!("(at {:?})", path));
            }
        }
        DependencyStatus::NotFound => {
            ui.colored_label(egui::Color32::RED, "✗");
            ui.label(format!("{}: Not found in PATH.", name));
        }
        DependencyStatus::PackageMissing(pkg_name) => {
            ui.colored_label(egui::Color32::RED, "✗");
            ui.label(format!("{}: Package '{}' not installed.", name, pkg_name));
        }
        DependencyStatus::Error(err) => {
            ui.colored_label(egui::Color32::YELLOW, "⚠️");
            ui.label(format!(
                "{}: Check error - {}",
                name,
                truncate_middle_egui(err, 60)
            ));
        }
    });
}

impl eframe::App for CsvEncodingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(msg) = self.receiver.try_recv() {
            match msg {
                AppMessage::ProcessingStarted => {
                    self.processing_status = "Processing...".to_string();
                    self.progress_log.push("Processing started.".to_string());
                }
                AppMessage::NewResult(_res) => { /* Results are now collected and sent with ProcessingFinished */
                }
                AppMessage::ProcessingFinished(all_outputs) => {
                    self.is_processing = false;
                    self.results = all_outputs;
                    self.processing_status = "Done".to_string();
                    self.progress_log.push("All checks finished.".to_string());
                }
                AppMessage::ProcessingError(err) => {
                    self.is_processing = false;
                    self.processing_status = format!("Error: {}", err);
                    self.progress_log.push(format!("ERROR: {}", err));
                }
                AppMessage::UpdateProgress(log_entry) => {
                    self.progress_log.push(log_entry);
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                ui.heading("CSV Encoding & Dimension Checker");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            self.render_dependencies(ui);
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Select CSV File").on_hover_text("Open a CSV file for analysis").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("CSV Files", &["csv"]).pick_file() {
                        self.csv_file_path = Some(path);
                        self.results.clear();
                        self.progress_log.clear();
                        self.processing_status = "Idle (New file selected)".to_string();
                    }
                }
                if let Some(path) = &self.csv_file_path { ui.label(format!("Selected: {}", path.file_name().unwrap_or_default().to_string_lossy())); }
                else { ui.label("No file selected."); }
            });
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Encodings (comma-separated):");
                ui.text_edit_singleline(&mut self.encodings_input).on_hover_text("e.g., UTF-8,BIG5,GB18030");
            });
            ui.add_space(5.0);

            let run_button_enabled = !self.is_processing && self.dependencies.all_ok() && self.script_paths_ok && self.csv_file_path.is_some();
            if ui.add_enabled(run_button_enabled, egui::Button::new("Run Checks"))
                .on_hover_text_at_pointer("Run dimension checks. Requires all dependencies to be met.")
                .clicked() { self.run_checks(); }

            ui.separator();
            ui.horizontal(|ui|{ ui.heading("Progress Log"); if ui.button("Clear Log").clicked() { self.progress_log.clear(); } });

            egui::ScrollArea::vertical().max_height(200.0).stick_to_bottom(true).show(ui, |ui| {
                for entry in &self.progress_log { ui.label(entry); }
            });

            ui.separator();
            ui.heading(format!("Results ({})", self.processing_status));

            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                if self.results.is_empty() && !self.is_processing { ui.label("No results yet. Select a file, enter encodings, and click 'Run Checks'."); }
                else {
                    egui::Grid::new("results_table").striped(true).num_columns(8).min_col_width(60.0).show(ui, |ui| {
                        ui.strong("Tool"); ui.strong("File"); ui.strong("Encoding"); ui.strong("Status");
                        ui.strong("Rows"); ui.strong("Cols"); ui.strong("Cells"); ui.strong("Error/Notes");
                        ui.end_row();
                        for res in &self.results {
                            ui.label(&res.tool);
                            ui.label(truncate_middle_egui(&res.file_path, 15));
                            ui.label(&res.encoding_tested);
                            match res.status.as_str() {
                                "Success" => ui.colored_label(egui::Color32::GREEN, &res.status),
                                _ => ui.colored_label(egui::Color32::LIGHT_RED, &res.status),
                            };
                            ui.label(res.rows.map_or("N/A".to_string(), |v| v.to_string()));
                            ui.label(res.cols.map_or("N/A".to_string(), |v| v.to_string()));
                            ui.label(res.cells.map_or("N/A".to_string(), |v| v.to_string()));
                            ui.label(truncate_middle_egui(res.error_message.as_deref().unwrap_or(""), 50));
                            ui.end_row();
                        }
                    });
                }
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn parse_script_output_csv_from_app(
    file_path: &Path,
) -> Result<Vec<ScriptOutput>, Box<dyn std::error::Error + Send + Sync>> {
    if !file_path.exists() {
        return Err(format!("Script output CSV file not found: {:?}", file_path).into());
    }
    if std::fs::metadata(file_path)?.len() == 0 {
        eprintln!("Warning: Script output file {:?} is empty.", file_path);
        return Ok(Vec::new());
    }
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut results = Vec::new();
    for (i, result) in rdr.deserialize().enumerate() {
        match result {
            Ok(record) => results.push(record),
            Err(e) => {
                let err_msg = format!(
                    "Failed to deserialize record {} from {:?}: {}",
                    i + 1,
                    file_path,
                    e
                );
                eprintln!("{}", err_msg);
                return Err(err_msg.into());
            }
        }
    }
    Ok(results)
}

fn create_error_output(
    tool: &str,
    csv_file: &Path,
    encoding: &str,
    status: &str,
    error_message: &str,
) -> ScriptOutput {
    ScriptOutput {
        tool: tool.to_string(),
        file_path: csv_file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        encoding_tested: encoding.to_string(),
        status: status.to_string(),
        rows: None,
        cols: None,
        cells: None,
        error_message: Some(error_message.to_string()),
    }
}

fn truncate_middle_egui(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return s.chars().take(max_len).collect();
    }
    let ellipsis = "...";
    let ellipsis_len = ellipsis.chars().count();
    let max_text_len = max_len.saturating_sub(ellipsis_len); // Ensure non-negative
    let mut front_len = max_text_len / 2;
    let mut back_len = max_text_len - front_len;

    // Adjust if one part becomes 0 due to small max_len and integer division
    if front_len == 0 && back_len > 0 && char_count > ellipsis_len {
        front_len = 1;
        back_len = max_text_len.saturating_sub(1);
    } else if back_len == 0 && front_len > 0 && char_count > ellipsis_len {
        back_len = 1;
        front_len = max_text_len.saturating_sub(1);
    }

    let front: String = s.chars().take(front_len).collect();
    let back: String = s
        .chars()
        .skip(char_count.saturating_sub(back_len))
        .take(back_len)
        .collect();
    format!("{}{}{}", front, ellipsis, back)
}
