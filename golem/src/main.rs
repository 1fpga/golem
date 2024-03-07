mod application;
mod data;
mod file_io;
mod hardware;
mod input;
mod macguiver;
mod main_inner;
mod platform;

fn main() {
    if let Err(e) = main_inner::main() {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}
