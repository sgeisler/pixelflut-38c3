use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// x position to print to
    #[arg(short)]
    pub x_pos: usize,

    /// y position to print to
    #[arg(short)]
    pub y_pos: usize,

    /// address to connect to with port
    #[arg(short, long)]
    pub address: String,

    /// scaling factor to use
    #[arg(short, long, default_value_t = 1)]
    pub scale: usize,
}
