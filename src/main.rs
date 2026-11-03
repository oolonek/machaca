fn main() {
    if let Err(error) = machaca::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
