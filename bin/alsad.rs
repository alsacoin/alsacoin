use alsacoin::cli::CliDaemon;

pub fn main() {
    CliDaemon::run().unwrap();
    CliDaemon::clean().unwrap()
}
