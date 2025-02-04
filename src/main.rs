use clap::Parser;
/// Flattens a folder structure
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Delete original files after flattening
    #[arg(short, long)]
    delete: String,
}

fn main() {
    let args = Args::parse();
}
