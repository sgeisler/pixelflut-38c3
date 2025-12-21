mod cli;
mod ffmpeg;
mod framebuffer;

use clap::Parser;
use framebuffer::FrameBuffer;

use std::io::*;
use std::net::TcpStream;

#[allow(unused)]
const X_SIZE: usize = 3840;
#[allow(unused)]
const Y_SIZE: usize = 2160;

type Frame = Vec<Vec<RGB>>;

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

fn show_picture(
    stream: &mut BufWriter<TcpStream>,
    fb: &mut FrameBuffer,
    buf: &Frame,
) -> std::io::Result<()> {
    let iter = buf.iter().flat_map(|x| x.iter().copied());
    fb.fill(iter);

    fb.write_to(stream)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let cla = cli::Args::parse();

    let stream = TcpStream::connect(cla.address)?;
    stream.set_nodelay(false).expect("set_nodelay failed");
    let mut writer = BufWriter::with_capacity(1 << 22, stream);

    let frames = ffmpeg::extract_frames(&cla.path, cla.scale)?;

    let width: usize = frames[0][0].len();
    let height: usize = frames[0].len();

    let mut fb = FrameBuffer::new(cla.x_pos, cla.y_pos, width, height);

    println!("start printing");

    loop {
        let receiver = ffmpeg::open_stream(cla.path.clone(), cla.scale)?;
        let mut id = 0;
        while let Ok(frame) = receiver.recv() {
            id += 1;
            if id % 100 == 0 {
                println!("Frame {id}");
            }
            show_picture(&mut writer, &mut fb, &frame)?;
        }
    }
} // the stream is closed here
