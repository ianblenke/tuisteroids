use tuisteroids::leaderboard;

fn main() {
    let args = std::env::args().skip(1);
    let options = match leaderboard::parse_args(args, &leaderboard::default_player()) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{}", message);
            std::process::exit(2);
        }
    };
    if let Err(e) = tuisteroids::game::run(options) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
