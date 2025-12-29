mod cli;
mod ffmpeg;
mod framebuffer;

use clap::Parser;
use framebuffer::FrameBuffer;
use image::ImageReader;

use core::f64;
use std::io::*;
use std::net::TcpStream;
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

#[allow(unused)]
const X_SIZE: usize = 3840;
#[allow(unused)]
const Y_SIZE: usize = 2160;

pub struct Frame {
    img: Vec<Vec<RGB>>,
    frame_duration: f64,
}

#[derive(Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        return RGB { r, g, b };
    }
}

fn show_frame(
    stream: &mut BufWriter<TcpStream>,
    fb: &mut FrameBuffer,
    buf: &Frame,
) -> std::io::Result<()> {
    let iter = buf.img.iter().flat_map(|x| x.iter().copied());

    fb.fill(iter);

    let frame_start_ts = SystemTime::now();

    loop {
        fb.write_to(&mut *stream)?;

        let elapsed = SystemTime::now()
            .duration_since(frame_start_ts)
            .expect("system time error");

        if elapsed.as_secs_f64() > buf.frame_duration {
            break;
        }
    }

    Ok(())
}

fn render_video(common_args: &cli::CommonArgs, args: &cli::VideoArgs) -> anyhow::Result<()> {
    let stream = TcpStream::connect(&common_args.address)?;
    stream.set_nodelay(false).expect("set_nodelay failed");
    let mut writer = BufWriter::with_capacity(1 << 22, stream);

    println!("start printing");

    loop {
        let frame_size = Arc::<OnceLock<(usize, usize)>>::default();
        let receiver = ffmpeg::open_stream(args.path.clone(), args.scale, frame_size.clone())?;

        let (width, height) = *frame_size.wait();

        let mut fb = FrameBuffer::new(common_args.x_pos, common_args.y_pos, width, height);

        let mut id = 0;
        while let Ok(frame) = receiver.recv() {
            id += 1;
            if id % 100 == 0 {
                println!("Frame {id}");
            }
            show_frame(&mut writer, &mut fb, &frame)?;
        }
    }
}

fn render_image(common_args: &cli::CommonArgs, args: &cli::ImageArgs) -> anyhow::Result<()> {
    let stream: TcpStream = TcpStream::connect(&common_args.address)?;
    // stream.set_nodelay(false).expect("set_nodelay failed");
    let mut writer = BufWriter::with_capacity(1 << 22, stream);

    let image = ImageReader::open(&args.path)?.decode()?;
    let width = image.width();
    let height = image.height();

    let frame = image
        .to_rgb8()
        .as_raw()
        .chunks(3)
        .map(|pixel| RGB::new(pixel[0], pixel[1], pixel[2]))
        .collect::<Vec<_>>();

    let mut fb = FrameBuffer::new(
        common_args.x_pos,
        common_args.y_pos,
        width as usize,
        height as usize,
    );

    fb.fill(frame.into_iter());

    loop {
        fb.write_to(&mut writer)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cla = cli::Args::parse();

    match cla.mode {
        cli::Mode::Video(args) => render_video(&cla.common, &args)?,
        cli::Mode::Image(args) => render_image(&cla.common, &args)?,
    }

    Ok(())
} // the stream is closed here
