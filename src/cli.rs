use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    /// What's to be rendered
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Parser, Debug)]
pub(crate) struct CommonArgs {
    /// x position to print to
    #[arg(short)]
    pub x_pos: usize,

    /// y position to print to
    #[arg(short)]
    pub y_pos: usize,

    /// address to connect to with port
    #[arg(short, long)]
    pub address: String,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Mode {
    Video(VideoArgs),
    Image(ImageArgs),
}

#[derive(Parser, Debug)]
pub(crate) struct VideoArgs {
    /// scaling factor to use
    #[arg(short, long, default_value_t = 1.0)]
    pub scale: f64,

    /// path to video
    pub path: String,
}

#[derive(Parser, Debug)]
pub(crate) struct ImageArgs {
    /// path to image
    pub path: String,
}
