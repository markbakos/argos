fn main() {
    if argos_desktop::run().is_err() {
        eprintln!("Argos failed to start.");
        std::process::exit(1);
    }
}
