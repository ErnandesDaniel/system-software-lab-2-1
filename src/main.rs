fn main() {
    if let Err(e) = mylang_parser::run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
