fn main() {
    // Loads agent/.env if present (searches upward from the working directory).
    let _ = dotenvy::dotenv();

    if let Err(error) = eury_desktop::run() {
        eprintln!("Eury Agent failed to start: {error}");
        std::process::exit(1);
    }
}
