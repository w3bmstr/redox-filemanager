use std::io;

mod actions;
mod archive;
mod error;
mod fs;
mod gui; // 👈 GUI module
mod navigation;
mod ui;

fn main() {
    println!("Redox File Manager starting...");
    ui::launch();

    loop {
        println!("\nChoose an action:");
        println!("1. List files");
        println!("2. Copy file");
        println!("3. Delete file");
        println!("4. Handle error");
        println!("5. Change directory");
        println!("6. Search files");
        println!("7. Batch delete files");
        println!("8. Rename file");
        println!("9. Move file");
        println!("10. Batch copy files");
        println!("11. Batch rename files");
        println!("12. Create file");
        println!("13. Create directory");
        println!("14. Delete directory");
        println!("15. Launch GUI");
        println!("17. Archive: List contents");
        println!("18. Archive: Extract");
        println!("19. Archive: Create");
        println!("20. Calculate file hash");
        println!("21. Find duplicate files");
        println!("22. Secure delete file");
        println!("23. Split file");
        println!("24. Join file chunks");
        println!("25. Exit");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => fs::list_files(),
            "2" => actions::copy_file(),
            "3" => actions::delete_file(), // 👈 now includes confirmation
            "4" => println!("Centralized error handling is active."),
            "5" => navigation::change_directory(),
            "6" => fs::search_files(),
            "7" => actions::batch_delete(),
            "8" => actions::rename_file(),
            "9" => actions::move_file(),
            "10" => actions::batch_copy(),
            "11" => actions::batch_rename(),
            "12" => actions::create_file(),
            "13" => actions::create_directory(),
            "14" => actions::delete_directory(), // 👈 now includes confirmation
            "15" => {
                if let Err(e) = gui::run_gui() {
                    println!("Failed to launch GUI: {}", e);
                }
            }
            "17" => actions::archive_list_cli(),
            "18" => actions::archive_extract_cli(),
            "19" => actions::archive_create_cli(),
            "20" => actions::calculate_hash(),
            "21" => actions::find_duplicates(),
            "22" => actions::secure_delete(),
            "23" => actions::split_file(),
            "24" => actions::join_files(),
            "25" => {
                println!("Exiting File Manager...");
                break;
            }
            _ => println!("Invalid choice, try again."),
        }
    }
}
