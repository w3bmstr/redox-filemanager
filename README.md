# redox-filemanager
A lightweight file manager for Redox OS, written in Rust.

# Redox File Manager

A lightweight file manager for Redox OS, written in Rust.  
This project aims to provide a simple GUI for browsing, opening, copying, and deleting files.

---

## 🚀 Getting Started

### Clone the repo
```bash
git clone https://github.com/w3bmstr/redox-filemanager.git
cd redox-filemanager
Build & run locally (Windows/PowerShell)
bash
cargo run
Cross-compile for Redox
bash
rustup target add x86_64-unknown-redox
cargo build --target x86_64-unknown-redox

📂 Project Structure
Code
src/
├── main.rs       # Entry point
├── ui.rs         # GUI layout
├── fs.rs         # Filesystem helpers
├── actions.rs    # User actions
└── error.rs      # Error handling

🛠️ Dependencies
OrbTk – GUI toolkit
Walkdir – Filesystem traversal

🎯 Roadmap
[x] Project skeleton

[ ] List files in a directory

[ ] Add basic GUI window

[ ] Implement open/delete/copy actions

[ ] Package for Redox OS
