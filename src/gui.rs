use eframe::egui;
use std::fs;
use std::path::Path;
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
                    let selected = self.selected.as_ref().map(|s| s.path.clone()) == Some(entry.path.clone());
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
                        match fs::remove_file(&selected_file.path) {
                            Ok(_) => {
                                self.status = format!("Deleted {}", selected_file.display);
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status = format!("Error deleting {}: {}", selected_file.display, e);
                                self.status_is_error = true;
                            }
                        }
                        self.files = read_files(&self.current_dir);
                        self.selected = None;
                    }

                    ui.text_edit_singleline(&mut self.rename_input);
                    if ui.button("Rename").clicked() {
                        let new_path = Path::new(&self.current_dir).join(&self.rename_input);
                        match fs::rename(&selected_file.path, &new_path) {
                            Ok(_) => {
                                self.status = format!("Renamed {} → {}", selected_file.display, self.rename_input);
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status = format!("Error renaming {}: {}", selected_file.display, e);
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
                        match fs::rename(&selected_file.path, &self.move_input) {
                            Ok(_) => {
                                self.status = format!("Moved {} → {}", selected_file.display, self.move_input);
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status = format!("Error moving {}: {}", selected_file.display, e);
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
                        match fs::copy(&selected_file.path, &self.copy_input) {
                            Ok(_) => {
                                self.status = format!("Copied {} → {}", selected_file.display, self.copy_input);
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status = format!("Error copying {}: {}", selected_file.display, e);
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
        match std::fs::File::create(Path::new(&self.current_dir).join(&self.new_name_input)) {
            Ok(_) => {
                self.status = format!("Created file {}", self.new_name_input);
                self.status_is_error = false;
            }
            Err(e) => {
                self.status = format!("Error creating file {}: {}", self.new_name_input, e);
                self.status_is_error = true;
            }
        }
        self.files = read_files(&self.current_dir);
        self.new_name_input.clear();
    }

    if ui.button("Create Directory").clicked() {
        match fs::create_dir(Path::new(&self.current_dir).join(&self.new_name_input)) {
            Ok(_) => {
                self.status = format!("Created directory {}", self.new_name_input);
                self.status_is_error = false;
            }
            Err(e) => {
                self.status = format!("Error creating directory {}: {}", self.new_name_input, e);
                self.status_is_error = true;
            }
        }
        self.files = read_files(&self.current_dir);
        self.new_name_input.clear();
    }

    if ui.button("Delete Directory").clicked() {
        match fs::remove_dir(Path::new(&self.current_dir).join(&self.new_name_input)) {
            Ok(_) => {
                self.status = format!("Deleted directory {}", self.new_name_input);
                self.status_is_error = false;
            }
            Err(e) => {
                self.status = format!("Error deleting directory {}: {}", self.new_name_input, e);
                self.status_is_error = true;
            }
        }
        self.files = read_files(&self.current_dir);
        self.new_name_input.clear();
    }
});

ui.separator();

// Batch operations
ui.horizontal(|ui| {
    ui.label("Batch (comma separated):");
    ui.text_edit_singleline(&mut self.batch_input);

    if ui.button("Batch Delete").clicked() {
        let names: Vec<&str> = self.batch_input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        for name in names {
            let path = Path::new(&self.current_dir).join(name);
            match fs::remove_file(&path) {
                Ok(_) => {
                    self.status = format!("Deleted {}", name);
                    self.status_is_error = false;
                }
                Err(e) => {
                    self.status = format!("Error deleting {}: {}", name, e);
                    self.status_is_error = true;
                }
            }
        }
        self.files = read_files(&self.current_dir);
        self.batch_input.clear();
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
        SortMode::Name => files.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase())),
        SortMode::Size => files.sort_by(|a, b| a.size.cmp(&b.size)),
        SortMode::Date => files.sort_by(|a, b| a.modified.cmp(&b.modified)),
    }
}
