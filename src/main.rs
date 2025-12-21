mod cli;
mod framebuffer;

use clap::Parser;
use framebuffer::FrameBuffer;

use std::io::*;
use std::net::TcpStream;

#[allow(unused)]
const X_SIZE: usize = 3840;
#[allow(unused)]
const Y_SIZE: usize = 2160;

#[derive(Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

fn get_image(filename: &String) -> Vec<Vec<RGB>> {
    let ppm_image_result = tinyppm::ppm_loader::read_image_data(filename);
    let ppm_image = match ppm_image_result {
        Ok(image) => image,
        _ => panic!("unable to read specified image file!"),
    };

    let mut frame: Vec<Vec<RGB>> =
        vec![vec![RGB { r: 0, g: 0, b: 0 }; ppm_image.width()]; ppm_image.height()];

    let mut x = 0;
    let mut y = 0;

    for px in ppm_image.pixels() {
        frame[y][x] = RGB {
            r: (*px >> 16u8 & 0xFF) as u8,
            g: (*px >> 8u8 & 0xFF) as u8,
            b: (*px >> 0u8 & 0xFF) as u8,
        };
        x += 1;
        if x >= ppm_image.width() {
            y += 1;
            x = 0;
        }
    }

    frame
}

fn show_picture(
    stream: &mut BufWriter<TcpStream>,
    fb: &mut FrameBuffer,
    buf: &Vec<Vec<RGB>>,
    scale: usize,
) -> std::io::Result<()> {
    let iter = buf.iter().flat_map(|x| {
        std::iter::repeat(x).take(scale.into()).flat_map(|x| {
            x.iter()
                .flat_map(|y| std::iter::repeat(*y).take(scale.into()))
        })
    });
    fb.fill(iter);

    fb.write_to(stream)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let cla = cli::Args::parse();

    let stream = TcpStream::connect(cla.address)?;
    stream.set_nodelay(false).expect("set_nodelay failed");
    let mut writer = BufWriter::with_capacity(1 << 22, stream);

    let mut frames: Vec<Vec<Vec<RGB>>> = Vec::new();
    for i in 1..13721 {
        let filename = format!("./images/bee_images_{:05}.ppm", i);
        let image = get_image(&String::from(filename));
        frames.push(image);
    }

    let width: usize = cla.scale * frames[0][0].len();
    let height: usize = cla.scale * frames[0].len();

    let mut fb = FrameBuffer::new(cla.x_pos, cla.y_pos, width, height);

    println!("printing");
    loop {
        for frame in &frames {
            for _ in 1..2 {
                show_picture(&mut writer, &mut fb, &frame, cla.scale)?;
            }
        }
    }
} // the stream is closed here
