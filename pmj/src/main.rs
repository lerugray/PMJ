mod app;
mod combat;
mod game;
mod input;
mod map;
mod mct;
mod ui;
mod units;

fn main() {
    if let Err(e) = app::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
