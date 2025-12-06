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

✅ Current Functionality
🔧 File Operations
List files → display contents of the current directory.

Copy file → duplicate a file to another location.

Delete file → remove a file.

Rename file → change a file’s name.

Move file → relocate a file to another directory.

Batch copy files → copy multiple files at once.

Batch delete files → delete multiple files at once.

Batch rename files → rename multiple files at once.

Create file → generate a new file.

📂 Directory Operations
Change directory → navigate into another folder.

Create directory → make a new folder.

Delete directory → remove a folder.

🔍 Search & Sort
Search files → find files by name.

Sort files → by name, size, and date (already implemented).

⚙️ Other Features
Handle error → placeholder for error handling logic.

Launch GUI → placeholder for graphical interface.

Exit → quit the program.

🎯 What This Means
You already have a solid CLI file manager with:

Core file manipulation (create, copy, move, delete, rename).

Directory management.

Sorting and searching.

Batch operations for efficiency.

A placeholder for GUI expansion.

That’s a strong foundation — basically the essentials of a file explorer, but in Rust.
