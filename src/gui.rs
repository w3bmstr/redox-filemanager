use eframe::egui;
use std::fs;
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Redox File Manager",
        options,
        Box::new(|_cc| Box::new(FileManagerApp::default())),
    )
}

#[derive(Clone)]
struct FileEntry {
    display: String,
    path: String,
    size: u64,
    modified: String,
    is_dir: bool,
    hidden: bool,
}

struct FileManagerApp {
    files: Vec<FileEntry>,
    current_dir: String,
    selected: Option<FileEntry>,
    rename_input: String,
    new_name_input: String,
    move_input: String,
    copy_input: String,
    search_input: String,
    batch_input: String,
    // top-level actions inputs
    change_dir_input: String,
    batch_copy_input: String,
    batch_copy_dest_input: String,
    batch_rename_input: String,
    // advanced action inputs
    archive_input: String,
    archive_dest_input: String,
    archive_sources_input: String,
    archive_output_input: String,
    archive_format_input: String,
    archive_password_input: String,
    hash_input: String,
    hash_algo_input: String,
    duplicates_dir_input: String,
    secure_delete_input: String,
    split_input: String,
    split_chunk_input: String,
    join_base_input: String,
    join_output_input: String,
    // confirmation dialogs state
    confirm_delete_open: bool,
    confirm_delete_target: String,
    confirm_delete_is_dir: bool,
    confirm_batch_open: bool,
    confirm_batch_targets: Vec<String>,
    confirm_secure_open: bool,
    confirm_secure_target: String,
    // background worker state
    worker_rx: Option<mpsc::Receiver<String>>,
    worker_cancel: Option<Arc<AtomicBool>>,
    progress_messages: Vec<String>,
    is_busy: bool,
    status: String,
    status_is_error: bool,
    sort_mode: SortMode,
}

#[derive(Clone, Copy)]
enum SortMode {
    Name,
    Size,
    Date,
}

impl Default for FileManagerApp {
    fn default() -> Self {
        let dir = ".".to_string();
        let files = read_files(&dir);
        Self {
            files,
            current_dir: dir,
            selected: None,
            rename_input: String::new(),
            new_name_input: String::new(),
            move_input: String::new(),
            copy_input: String::new(),
            search_input: String::new(),
            batch_input: String::new(),
            change_dir_input: String::new(),
            batch_copy_input: String::new(),
            batch_copy_dest_input: String::new(),
            batch_rename_input: String::new(),
            archive_input: String::new(),
            archive_dest_input: String::new(),
            archive_sources_input: String::new(),
            archive_output_input: String::new(),
            archive_format_input: String::new(),
            archive_password_input: String::new(),
            hash_input: String::new(),
            hash_algo_input: String::from("sha256"),
            duplicates_dir_input: String::new(),
            secure_delete_input: String::new(),
            split_input: String::new(),
            split_chunk_input: String::from("100"),
            join_base_input: String::new(),
            join_output_input: String::new(),
            confirm_delete_open: false,
            confirm_delete_target: String::new(),
            confirm_delete_is_dir: false,
            confirm_batch_open: false,
            confirm_batch_targets: Vec::new(),
            confirm_secure_open: false,
            confirm_secure_target: String::new(),
            worker_rx: None,
            worker_cancel: None,
            progress_messages: Vec::new(),
            is_busy: false,
            status: String::new(),
            status_is_error: false,
            sort_mode: SortMode::Name,
        }
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Directory: {}", self.current_dir));

            // Sorting options
            ui.horizontal(|ui| {
                ui.label("Sort by:");
                if ui.button("Name").clicked() {
                    self.sort_mode = SortMode::Name;
                    sort_files(&mut self.files, self.sort_mode);
                }
                if ui.button("Size").clicked() {
                    self.sort_mode = SortMode::Size;
                    sort_files(&mut self.files, self.sort_mode);
                }
                if ui.button("Date").clicked() {
                    self.sort_mode = SortMode::Date;
                    sort_files(&mut self.files, self.sort_mode);
                }
            });

            ui.separator();

            // Actions panel mirroring CLI menu
            egui::CollapsingHeader::new("Actions (CLI parity)")
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("1. List files").clicked() {
                            self.files = read_files(&self.current_dir);
                            self.status = "Listed files".to_string();
                        }
                        if ui.button("2. Copy file").clicked() {
                            /* copy uses selected + copy_input */
                            if let Some(sel) = &self.selected {
                                match crate::actions::copy_file_noninteractive(
                                    &sel.path,
                                    &self.copy_input,
                                ) {
                                    Ok(_) => {
                                        self.status = format!(
                                            "Copied {} -> {}",
                                            sel.display, self.copy_input
                                        );
                                        self.files = read_files(&self.current_dir);
                                    }
                                    Err(e) => {
                                        self.status = format!("Error copying: {}", e);
                                        self.status_is_error = true
                                    }
                                }
                            } else {
                                self.status = "No file selected".to_string();
                            }
                        }
                        if ui.button("3. Delete file").clicked() {
                            if let Some(sel) = &self.selected {
                                self.confirm_delete_open = true;
                                self.confirm_delete_target = sel.path.clone();
                                self.confirm_delete_is_dir = false;
                            } else {
                                self.status = "No file selected".to_string();
                            }
                        }
                        if ui.button("4. Handle error").clicked() {
                            self.status = "Centralized error handling is active.".to_string();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Change dir:");
                        ui.text_edit_singleline(&mut self.change_dir_input);
                        if ui.button("5. Change directory").clicked() {
                            if !self.change_dir_input.trim().is_empty() {
                                self.current_dir = self.change_dir_input.clone();
                                self.files = read_files(&self.current_dir);
                                self.status = format!("Changed dir to {}", self.current_dir);
                            } else {
                                self.status = "Enter directory".to_string();
                            }
                        }
                        if ui.button("6. Search files").clicked() {
                            let query = self.search_input.to_lowercase();
                            self.files = read_files(&self.current_dir)
                                .into_iter()
                                .filter(|f| f.display.to_lowercase().contains(&query))
                                .collect();
                            self.status = format!("Search results for '{}'", self.search_input);
                        }
                        if ui.button("7. Batch delete files").clicked() {
                            self.confirm_batch_targets = self
                                .batch_input
                                .split(',')
                                .map(|s| self.current_dir.clone() + "/" + s.trim())
                                .filter(|s| !s.is_empty())
                                .collect();
                            self.confirm_batch_open = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("8. Rename file").clicked() {
                            if let Some(sel) = &self.selected {
                                let new_path = Path::new(&self.current_dir)
                                    .join(&self.rename_input)
                                    .to_string_lossy()
                                    .to_string();
                                match crate::actions::rename_file_noninteractive(
                                    &sel.path, &new_path,
                                ) {
                                    Ok(_) => {
                                        self.status = format!(
                                            "Renamed {} -> {}",
                                            sel.display, self.rename_input
                                        );
                                        self.files = read_files(&self.current_dir);
                                        self.selected = None;
                                    }
                                    Err(e) => {
                                        self.status = format!("Error renaming: {}", e);
                                        self.status_is_error = true
                                    }
                                }
                            } else {
                                self.status = "No file selected".to_string();
                            }
                        }
                        if ui.button("9. Move file").clicked() {
                            if let Some(sel) = &self.selected {
                                match crate::actions::move_file_noninteractive(
                                    &sel.path,
                                    &self.move_input,
                                ) {
                                    Ok(_) => {
                                        self.status =
                                            format!("Moved {} -> {}", sel.display, self.move_input);
                                        self.files = read_files(&self.current_dir);
                                        self.selected = None;
                                    }
                                    Err(e) => {
                                        self.status = format!("Error moving: {}", e);
                                        self.status_is_error = true
                                    }
                                }
                            } else {
                                self.status = "No file selected".to_string();
                            }
                        }
                        if ui.button("10. Batch copy files").clicked() {
                            let names: Vec<String> = self
                                .batch_copy_input
                                .split(',')
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .map(|n| {
                                    Path::new(&self.current_dir)
                                        .join(n)
                                        .to_string_lossy()
                                        .to_string()
                                })
                                .collect();
                            let results = crate::actions::batch_copy_noninteractive(
                                &names,
                                &self.batch_copy_dest_input,
                            );
                            for (i, res) in results.iter().enumerate() {
                                match res {
                                    Ok(_) => self.status = format!("Copied {}", names[i]),
                                    Err(e) => {
                                        self.status = format!("Error copying {}: {}", names[i], e);
                                        self.status_is_error = true
                                    }
                                }
                            }
                            self.files = read_files(&self.current_dir);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Batch rename pairs (old:new,comma):");
                        ui.text_edit_singleline(&mut self.batch_rename_input);
                        if ui.button("11. Batch rename files").clicked() {
                            let pairs: Vec<(String, String)> = self
                                .batch_rename_input
                                .split(',')
                                .filter(|s| !s.trim().is_empty())
                                .filter_map(|p| {
                                    p.split_once(':').map(|(a, b)| {
                                        (
                                            self.current_dir.clone() + "/" + a.trim(),
                                            self.current_dir.clone() + "/" + b.trim(),
                                        )
                                    })
                                })
                                .collect();
                            let results = crate::actions::batch_rename_noninteractive(&pairs);
                            for (i, res) in results.iter().enumerate() {
                                match res {
                                    Ok(_) => {
                                        self.status =
                                            format!("Renamed {} -> {}", pairs[i].0, pairs[i].1)
                                    }
                                    Err(e) => {
                                        self.status =
                                            format!("Error renaming {}: {}", pairs[i].0, e);
                                        self.status_is_error = true
                                    }
                                }
                            }
                            self.files = read_files(&self.current_dir);
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("12. Create file").clicked() {
                            let path = Path::new(&self.current_dir)
                                .join(&self.new_name_input)
                                .to_string_lossy()
                                .to_string();
                            match crate::actions::create_file_noninteractive(&path) {
                                Ok(_) => {
                                    self.status = format!("Created file {}", self.new_name_input);
                                    self.files = read_files(&self.current_dir);
                                }
                                Err(e) => {
                                    self.status = format!(
                                        "Error creating file {}: {}",
                                        self.new_name_input, e
                                    );
                                    self.status_is_error = true
                                }
                            }
                        }
                        if ui.button("13. Create directory").clicked() {
                            let path = Path::new(&self.current_dir)
                                .join(&self.new_name_input)
                                .to_string_lossy()
                                .to_string();
                            match crate::actions::create_directory_noninteractive(&path) {
                                Ok(_) => {
                                    self.status =
                                        format!("Created directory {}", self.new_name_input);
                                    self.files = read_files(&self.current_dir);
                                }
                                Err(e) => {
                                    self.status = format!(
                                        "Error creating dir {}: {}",
                                        self.new_name_input, e
                                    );
                                    self.status_is_error = true
                                }
                            }
                        }
                        if ui.button("14. Delete directory").clicked() {
                            let path = Path::new(&self.current_dir)
                                .join(&self.new_name_input)
                                .to_string_lossy()
                                .to_string();
                            self.confirm_delete_open = true;
                            self.confirm_delete_target = path;
                            self.confirm_delete_is_dir = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("17. Archive: List contents").clicked() {
                            match crate::actions::archive_list_noninteractive(&self.archive_input) {
                                Ok(s) => {
                                    self.status = s;
                                    self.status_is_error = false
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                        }
                        if ui.button("18. Archive: Extract").clicked() {
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                let (tx, rx) = mpsc::channel();
                                let cancel = Arc::new(AtomicBool::new(false));
                                let inp = self.archive_input.clone();
                                let dest = self.archive_dest_input.clone();
                                let pwd = if self.archive_password_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.archive_password_input.clone())
                                };
                                let cancel_clone = cancel.clone();
                                std::thread::spawn(move || {
                                    crate::actions::archive_extract_progress(
                                        &inp,
                                        &dest,
                                        pwd.as_deref(),
                                        tx,
                                        cancel_clone,
                                    );
                                });
                                self.worker_rx = Some(rx);
                                self.worker_cancel = Some(cancel);
                                self.progress_messages.clear();
                                self.is_busy = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("19. Archive: Create").clicked() {
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                let sources: Vec<String> = self
                                    .archive_sources_input
                                    .split(',')
                                    .map(|s| s.trim())
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect();
                                let fmt_owned = if self.archive_format_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.archive_format_input.clone())
                                };
                                let pwd_owned = if self.archive_password_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.archive_password_input.clone())
                                };
                                let (tx, rx) = mpsc::channel();
                                let cancel = Arc::new(AtomicBool::new(false));
                                let cancel_clone = cancel.clone();
                                let sources_clone = sources.clone();
                                let output = self.archive_output_input.clone();
                                std::thread::spawn(move || {
                                    crate::actions::archive_create_progress(
                                        &sources_clone,
                                        &output,
                                        fmt_owned.as_deref(),
                                        pwd_owned.as_deref(),
                                        tx,
                                        cancel_clone,
                                    );
                                });
                                self.worker_rx = Some(rx);
                                self.worker_cancel = Some(cancel);
                                self.progress_messages.clear();
                                self.is_busy = true;
                            }
                        }
                        if ui.button("20. Calculate file hash").clicked() {
                            match crate::actions::calculate_hash_noninteractive(
                                &self.hash_input,
                                &self.hash_algo_input,
                            ) {
                                Ok(s) => {
                                    self.status = s;
                                    self.status_is_error = false
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("21. Find duplicate files").clicked() {
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                let (tx, rx) = mpsc::channel();
                                let cancel = Arc::new(AtomicBool::new(false));
                                let dir = self.duplicates_dir_input.clone();
                                let cancel_clone = cancel.clone();
                                std::thread::spawn(move || {
                                    crate::actions::find_duplicates_progress(
                                        &dir,
                                        tx,
                                        cancel_clone,
                                    );
                                });
                                self.worker_rx = Some(rx);
                                self.worker_cancel = Some(cancel);
                                self.progress_messages.clear();
                                self.is_busy = true;
                            }
                        }
                        if ui.button("22. Secure delete file").clicked() {
                            self.confirm_secure_open = true;
                            self.confirm_secure_target = self.secure_delete_input.clone();
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("23. Split file").clicked() {
                            let mb = self.split_chunk_input.parse::<u64>().unwrap_or(100);
                            match crate::actions::split_file_noninteractive(&self.split_input, mb) {
                                Ok(n) => {
                                    self.status = format!("Split into {} parts", n);
                                    self.files = read_files(&self.current_dir);
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                        }
                        if ui.button("24. Join file chunks").clicked() {
                            match crate::actions::join_files_noninteractive(
                                &self.join_base_input,
                                &self.join_output_input,
                            ) {
                                Ok(n) => {
                                    self.status = format!("Joined {} parts", n);
                                    self.files = read_files(&self.current_dir);
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("25. Exit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });

            // Advanced actions panel
            egui::CollapsingHeader::new("Advanced Actions")
                .default_open(false)
                .show(ui, |ui| {
                    // show availability of external 7z
                    if crate::archive::is_7z_available() {
                        ui.colored_label(
                            egui::Color32::LIGHT_GREEN,
                            "7z: available (will use 7z for many formats)",
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "7z: not found — using built-in handlers for zip/tar",
                        );
                    }
                    ui.horizontal(|ui| {
                        ui.label("Archive path:");
                        ui.text_edit_singleline(&mut self.archive_input);
                        if ui.button("List Archive").clicked() {
                            match crate::actions::archive_list_noninteractive(&self.archive_input) {
                                Ok(s) => {
                                    self.status = s;
                                    self.status_is_error = false
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Archive:");
                        ui.text_edit_singleline(&mut self.archive_input);
                        ui.label("Dest:");
                        ui.text_edit_singleline(&mut self.archive_dest_input);
                        ui.label("Pwd:");
                        ui.text_edit_singleline(&mut self.archive_password_input);
                        if ui.button("Extract").clicked() {
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                let pwd = if self.archive_password_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.archive_password_input.clone())
                                };
                                let (tx, rx) = mpsc::channel();
                                let cancel = Arc::new(AtomicBool::new(false));
                                let inp = self.archive_input.clone();
                                let dest = self.archive_dest_input.clone();
                                let cancel_clone = cancel.clone();
                                let pwd_move = pwd.clone();
                                std::thread::spawn(move || {
                                    crate::actions::archive_extract_progress(
                                        &inp,
                                        &dest,
                                        pwd_move.as_deref(),
                                        tx,
                                        cancel_clone,
                                    );
                                });
                                self.worker_rx = Some(rx);
                                self.worker_cancel = Some(cancel);
                                self.progress_messages.clear();
                                self.is_busy = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Sources (comma):");
                        ui.text_edit_singleline(&mut self.archive_sources_input);
                        ui.label("Out:");
                        ui.text_edit_singleline(&mut self.archive_output_input);
                        ui.label("Fmt:");
                        ui.text_edit_singleline(&mut self.archive_format_input);
                        if ui.button("Create Archive").clicked() {
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                let sources: Vec<String> = self
                                    .archive_sources_input
                                    .split(',')
                                    .map(|s| s.trim())
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect();
                                let fmt_owned = if self.archive_format_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.archive_format_input.clone())
                                };
                                let pwd_owned = if self.archive_password_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.archive_password_input.clone())
                                };
                                let (tx, rx) = mpsc::channel();
                                let cancel = Arc::new(AtomicBool::new(false));
                                let cancel_clone = cancel.clone();
                                let sources_clone = sources.clone();
                                let output = self.archive_output_input.clone();
                                std::thread::spawn(move || {
                                    crate::actions::archive_create_progress(
                                        &sources_clone,
                                        &output,
                                        fmt_owned.as_deref(),
                                        pwd_owned.as_deref(),
                                        tx,
                                        cancel_clone,
                                    );
                                });
                                self.worker_rx = Some(rx);
                                self.worker_cancel = Some(cancel);
                                self.progress_messages.clear();
                                self.is_busy = true;
                            }
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Hash file:");
                        ui.text_edit_singleline(&mut self.hash_input);
                        ui.label("Algo:");
                        ui.text_edit_singleline(&mut self.hash_algo_input);
                        if ui.button("Calculate").clicked() {
                            match crate::actions::calculate_hash_noninteractive(
                                &self.hash_input,
                                &self.hash_algo_input,
                            ) {
                                Ok(s) => {
                                    self.status = s;
                                    self.status_is_error = false
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Find duplicates in:");
                        ui.text_edit_singleline(&mut self.duplicates_dir_input);
                        if ui.button("Find Duplicates").clicked() {
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                let (tx, rx) = mpsc::channel();
                                let cancel = Arc::new(AtomicBool::new(false));
                                let dir = self.duplicates_dir_input.clone();
                                let cancel_clone = cancel.clone();
                                std::thread::spawn(move || {
                                    crate::actions::find_duplicates_progress(
                                        &dir,
                                        tx,
                                        cancel_clone,
                                    );
                                });
                                self.worker_rx = Some(rx);
                                self.worker_cancel = Some(cancel);
                                self.progress_messages.clear();
                                self.is_busy = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Secure delete file:");
                        ui.text_edit_singleline(&mut self.secure_delete_input);
                        if ui.button("Secure Delete").clicked() {
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                // open secure confirm that triggers background secure delete
                                self.confirm_secure_open = true;
                                self.confirm_secure_target = self.secure_delete_input.clone();
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Split file:");
                        ui.text_edit_singleline(&mut self.split_input);
                        ui.label("Chunk MB:");
                        ui.text_edit_singleline(&mut self.split_chunk_input);
                        if ui.button("Split").clicked() {
                            let mb = self.split_chunk_input.parse::<u64>().unwrap_or(100);
                            match crate::actions::split_file_noninteractive(&self.split_input, mb) {
                                Ok(n) => {
                                    self.status = format!("Split into {} parts", n);
                                    self.status_is_error = false
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                            self.files = read_files(&self.current_dir);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Join base:");
                        ui.text_edit_singleline(&mut self.join_base_input);
                        ui.label("Out:");
                        ui.text_edit_singleline(&mut self.join_output_input);
                        if ui.button("Join").clicked() {
                            match crate::actions::join_files_noninteractive(
                                &self.join_base_input,
                                &self.join_output_input,
                            ) {
                                Ok(n) => {
                                    self.status = format!("Joined {} parts", n);
                                    self.status_is_error = false
                                }
                                Err(e) => {
                                    self.status = e;
                                    self.status_is_error = true
                                }
                            }
                            self.files = read_files(&self.current_dir);
                        }
                    });
                });

            // Search bar
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_input);
                if ui.button("Go").clicked() {
                    let query = self.search_input.to_lowercase();
                    self.files = read_files(&self.current_dir)
                        .into_iter()
                        .filter(|f| f.display.to_lowercase().contains(&query))
                        .collect();
                    self.status = format!("Search results for '{}'", self.search_input);
                    self.status_is_error = false;
                }
                if ui.button("Clear").clicked() {
                    self.files = read_files(&self.current_dir);
                    self.search_input.clear();
                    self.status = "Search cleared".to_string();
                    self.status_is_error = false;
                }
            });

            // File list with metadata
            egui::Grid::new("file_grid").striped(true).show(ui, |ui| {
                ui.label("Name");
                ui.label("Size");
                ui.label("Modified");
                ui.label("Type");
                ui.end_row();

                // iterate over a clone so we can safely mutate self.files later
                for entry in self.files.clone() {
                    let selected =
                        self.selected.as_ref().map(|s| s.path.clone()) == Some(entry.path.clone());
                    let label = if entry.hidden {
                        format!("{} (hidden)", entry.display)
                    } else {
                        entry.display.clone()
                    };

                    let response = ui.selectable_label(selected, label);
                    ui.label(format!("{} bytes", entry.size));
                    ui.label(&entry.modified);
                    ui.label(if entry.is_dir { "Directory" } else { "File" });
                    ui.end_row();

                    // Double-click navigation into directories
                    if response.double_clicked() && entry.is_dir {
                        self.current_dir = entry.path.clone();
                        self.files = read_files(&self.current_dir);
                        self.selected = None;
                        self.status = format!("Entered directory {}", self.current_dir);
                        self.status_is_error = false;
                    } else if response.clicked() {
                        self.selected = Some(entry.clone());
                        self.rename_input = entry.display.clone();
                        self.status.clear();
                    }
                }
            });

            ui.separator();

            // Actions on selected file
            if let Some(selected_file) = self.selected.clone() {
                ui.label(format!("Selected: {}", selected_file.display));

                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        // open confirmation dialog
                        self.confirm_delete_open = true;
                        self.confirm_delete_target = selected_file.path.clone();
                        self.confirm_delete_is_dir = false;
                    }

                    ui.text_edit_singleline(&mut self.rename_input);
                    if ui.button("Rename").clicked() {
                        let new_path = Path::new(&self.current_dir).join(&self.rename_input);
                        let new_path_str = new_path.to_string_lossy().to_string();
                        match crate::actions::rename_file_noninteractive(
                            &selected_file.path,
                            &new_path_str,
                        ) {
                            Ok(_) => {
                                self.status = format!(
                                    "Renamed {} -> {}",
                                    selected_file.display, self.rename_input
                                );
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status =
                                    format!("Error renaming {}: {}", selected_file.display, e);
                                self.status_is_error = true;
                            }
                        }
                        self.files = read_files(&self.current_dir);
                        self.selected = None;
                    }
                });

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.move_input);
                    if ui.button("Move").clicked() {
                        match crate::actions::move_file_noninteractive(
                            &selected_file.path,
                            &self.move_input,
                        ) {
                            Ok(_) => {
                                self.status = format!(
                                    "Moved {} -> {}",
                                    selected_file.display, self.move_input
                                );
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status =
                                    format!("Error moving {}: {}", selected_file.display, e);
                                self.status_is_error = true;
                            }
                        }
                        self.files = read_files(&self.current_dir);
                        self.selected = None;
                    }
                });

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.copy_input);
                    if ui.button("Copy").clicked() {
                        match crate::actions::copy_file_noninteractive(
                            &selected_file.path,
                            &self.copy_input,
                        ) {
                            Ok(_) => {
                                self.status = format!(
                                    "Copied {} -> {}",
                                    selected_file.display, self.copy_input
                                );
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status =
                                    format!("Error copying {}: {}", selected_file.display, e);
                                self.status_is_error = true;
                            }
                        }
                        self.files = read_files(&self.current_dir);
                    }
                });
            }

            ui.separator();

            ui.separator();

            // Create / Delete panel
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_name_input);

                if ui.button("Create File").clicked() {
                    let path = Path::new(&self.current_dir)
                        .join(&self.new_name_input)
                        .to_string_lossy()
                        .to_string();
                    match crate::actions::create_file_noninteractive(&path) {
                        Ok(_) => {
                            self.status = format!("Created file {}", self.new_name_input);
                            self.status_is_error = false;
                        }
                        Err(e) => {
                            self.status =
                                format!("Error creating file {}: {}", self.new_name_input, e);
                            self.status_is_error = true;
                        }
                    }
                    self.files = read_files(&self.current_dir);
                    self.new_name_input.clear();
                }

                if ui.button("Create Directory").clicked() {
                    let path = Path::new(&self.current_dir)
                        .join(&self.new_name_input)
                        .to_string_lossy()
                        .to_string();
                    match crate::actions::create_directory_noninteractive(&path) {
                        Ok(_) => {
                            self.status = format!("Created directory {}", self.new_name_input);
                            self.status_is_error = false;
                        }
                        Err(e) => {
                            self.status =
                                format!("Error creating directory {}: {}", self.new_name_input, e);
                            self.status_is_error = true;
                        }
                    }
                    self.files = read_files(&self.current_dir);
                    self.new_name_input.clear();
                }

                if ui.button("Delete Directory").clicked() {
                    // open confirmation for directory delete
                    let path = Path::new(&self.current_dir)
                        .join(&self.new_name_input)
                        .to_string_lossy()
                        .to_string();
                    self.confirm_delete_open = true;
                    self.confirm_delete_target = path;
                    self.confirm_delete_is_dir = true;
                }
            });

            ui.separator();

            // Batch operations
            ui.horizontal(|ui| {
                ui.label("Batch (comma separated):");
                ui.text_edit_singleline(&mut self.batch_input);

                if ui.button("Batch Delete").clicked() {
                    // prepare batch confirmation
                    let names: Vec<String> = self
                        .batch_input
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|n| {
                            Path::new(&self.current_dir)
                                .join(n)
                                .to_string_lossy()
                                .to_string()
                        })
                        .collect();
                    self.confirm_batch_targets = names;
                    self.confirm_batch_open = true;
                }
            });

            // Refresh and Exit
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    self.files = read_files(&self.current_dir);
                    self.status = "Refreshed file list".to_string();
                    self.status_is_error = false;
                }
                if ui.button("Exit GUI").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.separator();

            // Colored status bar
            let color = if self.status_is_error {
                egui::Color32::RED
            } else {
                egui::Color32::GREEN
            };
            ui.colored_label(color, &self.status);
        });

        // Confirmation dialogs
        if self.confirm_delete_open {
            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete '{}'?",
                        self.confirm_delete_target
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            if self.confirm_delete_is_dir {
                                match crate::actions::delete_directory_noninteractive(
                                    &self.confirm_delete_target,
                                ) {
                                    Ok(_) => {
                                        self.status = format!(
                                            "Deleted directory {}",
                                            self.confirm_delete_target
                                        );
                                        self.status_is_error = false
                                    }
                                    Err(e) => {
                                        self.status = format!(
                                            "Error deleting directory {}: {}",
                                            self.confirm_delete_target, e
                                        );
                                        self.status_is_error = true
                                    }
                                }
                            } else {
                                match crate::actions::delete_file_noninteractive(
                                    &self.confirm_delete_target,
                                ) {
                                    Ok(_) => {
                                        self.status =
                                            format!("Deleted {}", self.confirm_delete_target);
                                        self.status_is_error = false
                                    }
                                    Err(e) => {
                                        self.status = format!(
                                            "Error deleting {}: {}",
                                            self.confirm_delete_target, e
                                        );
                                        self.status_is_error = true
                                    }
                                }
                            }
                            self.files = read_files(&self.current_dir);
                            self.selected = None;
                            self.confirm_delete_open = false;
                            self.confirm_delete_target.clear();
                        }
                        if ui.button("No").clicked() {
                            self.confirm_delete_open = false;
                            self.confirm_delete_target.clear();
                        }
                    });
                });
        }

        if self.confirm_batch_open {
            egui::Window::new("Confirm Batch Delete")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete {} files?",
                        self.confirm_batch_targets.len()
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            let results = crate::actions::batch_delete_noninteractive(
                                &self.confirm_batch_targets,
                            );
                            for (i, res) in results.iter().enumerate() {
                                match res {
                                    Ok(_) => {
                                        self.status =
                                            format!("Deleted {}", self.confirm_batch_targets[i]);
                                        self.status_is_error = false
                                    }
                                    Err(e) => {
                                        self.status = format!(
                                            "Error deleting {}: {}",
                                            self.confirm_batch_targets[i], e
                                        );
                                        self.status_is_error = true
                                    }
                                }
                            }
                            self.files = read_files(&self.current_dir);
                            self.confirm_batch_open = false;
                            self.confirm_batch_targets.clear();
                        }
                        if ui.button("No").clicked() {
                            self.confirm_batch_open = false;
                            self.confirm_batch_targets.clear();
                        }
                    });
                });
        }

        if self.confirm_secure_open {
            egui::Window::new("Confirm Secure Delete")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Securely delete '{}' ? This will overwrite the file.",
                        self.confirm_secure_target
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            // spawn background secure-delete worker
                            if self.is_busy {
                                self.status = "Already running an operation".to_string();
                            } else {
                                let (tx, rx) = mpsc::channel();
                                let cancel = Arc::new(AtomicBool::new(false));
                                let target = self.confirm_secure_target.clone();
                                let cancel_clone = cancel.clone();
                                std::thread::spawn(move || {
                                    crate::actions::secure_delete_progress(
                                        &target,
                                        tx,
                                        cancel_clone,
                                    );
                                });
                                self.worker_rx = Some(rx);
                                self.worker_cancel = Some(cancel);
                                self.progress_messages.clear();
                                self.is_busy = true;
                                // close confirm window
                                self.confirm_secure_open = false;
                                self.confirm_secure_target.clear();
                            }
                        }
                        if ui.button("No").clicked() {
                            self.confirm_secure_open = false;
                            self.confirm_secure_target.clear();
                        }
                    });
                });
        }

        // Poll worker receiver for messages
        if let Some(rx) = &self.worker_rx {
            // pull all pending messages
            while let Ok(msg) = rx.try_recv() {
                self.progress_messages.push(msg.clone());
                self.status = msg;
            }
            // if channel closed and we were busy, mark not busy
            if self.is_busy {
                // cannot directly detect closed here without blocking; we check if rx.try_recv returns Err(Disconnected)
                match rx.try_recv() {
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.is_busy = false;
                        self.worker_rx = None;
                        self.worker_cancel = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    Ok(msg) => {
                        self.progress_messages.push(msg.clone());
                        self.status = msg;
                    }
                }
            }
        }

        // Progress panel
        egui::TopBottomPanel::bottom("progress_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Progress:");
                if ui.button("Show Details").clicked() {
                    // toggle a large output — here we just leave messages visible via label
                }
                if self.is_busy {
                    if ui.button("Cancel").clicked() {
                        if let Some(cancel) = &self.worker_cancel {
                            cancel.store(true, Ordering::SeqCst);
                            self.status = "Cancellation requested".to_string();
                        }
                    }
                }
            });
            for m in self.progress_messages.iter().rev().take(8) {
                ui.label(m);
            }
        });
    }
}

fn read_files(dir: &str) -> Vec<FileEntry> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = metadata.len();
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
            let modified_str = format!("{}", duration.as_secs());

            let is_dir = metadata.is_dir();
            let name = entry.file_name().to_string_lossy().to_string();

            let mut hidden = name.starts_with('.');
            #[cfg(target_os = "windows")]
            {
                const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
                if metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
                    hidden = true;
                }
            }

            files.push(FileEntry {
                display: name,
                path: path.to_string_lossy().to_string(),
                size,
                modified: modified_str,
                is_dir,
                hidden,
            });
        }
    }
    sort_files(&mut files, SortMode::Name);
    files
}

fn sort_files(files: &mut Vec<FileEntry>, mode: SortMode) {
    match mode {
        SortMode::Name => {
            files.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()))
        }
        SortMode::Size => files.sort_by(|a, b| a.size.cmp(&b.size)),
        SortMode::Date => files.sort_by(|a, b| a.modified.cmp(&b.modified)),
    }
}
