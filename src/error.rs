use std::io;

/// Centralized error handler: takes any `io::Error` and prints a friendly message.
pub fn handle_error(err: io::Error, context: &str) {
    match err.kind() {
        io::ErrorKind::NotFound => {
            println!("Error: {} not found.", context);
        }
        io::ErrorKind::PermissionDenied => {
            println!("Error: Permission denied while accessing {}.", context);
        }
        io::ErrorKind::AlreadyExists => {
            println!("Error: {} already exists.", context);
        }
        io::ErrorKind::InvalidInput => {
            println!("Error: Invalid input provided for {}.", context);
        }
        io::ErrorKind::UnexpectedEof => {
            println!("Error: Unexpected end of file while reading {}.", context);
        }
        io::ErrorKind::WriteZero => {
            println!("Error: Failed to write data to {}.", context);
        }
        io::ErrorKind::Interrupted => {
            println!("Error: Operation interrupted while working with {}.", context);
        }
        _ => {
            println!("Unexpected error with {}: {}", context, err);
        }
    }
}
