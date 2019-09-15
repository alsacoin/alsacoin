use alsacoin::cli::CliClient;

pub fn main() {
    let matches = CliClient::args();
    println!("alsac matches: {:?}", matches)
}
