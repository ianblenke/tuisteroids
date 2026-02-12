fn main() {
    if let Err(e) = tuisteroids::game::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
