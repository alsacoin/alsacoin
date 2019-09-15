use alsacoin::cli::CliDaemon;

pub fn main() {
    let matches = CliDaemon::args();
    println!("alsad matches: {:?}", matches)
}
