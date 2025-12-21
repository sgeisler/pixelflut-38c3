use lazy_static::lazy_static;
use std::io::*;
use std::iter::zip;
use std::net::TcpStream;

mod lut;

extern crate tinyppm;

#[allow(unused)]
const X_SIZE: usize = 3840;
#[allow(unused)]
const Y_SIZE: usize = 2160;

lazy_static! {
    static ref RGB_LUT: [[u8; 2]; 256] = lut::gen_rgb_lut();
}

#[derive(Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

struct FrameBuffer {
    str: Vec<u8>,
    idx: Vec<usize>,
}

impl FrameBuffer {
    #[allow(unused)]
    fn new(x: usize, y: usize, w: usize, h: usize) -> FrameBuffer {
        let n = w * h;
        let mut fb = FrameBuffer {
            str: Vec::new(),
            idx: Vec::with_capacity(n),
        };

        println!("x: {x} y: {y} w: {w} h: {h}");

        for y_pos in y..y + h {
            for x_pos in x..x + w {
                let str1 = format!("PX {x_pos} {y_pos} ");

                fb.str.extend_from_slice(str1.as_bytes());
                fb.idx.push(fb.str.len());
                fb.str.extend_from_slice(b"xxxxxx\n");
            }
        }

        println!("idx length: {}", fb.idx.len());

        fb
    }

    #[inline]
    fn fill(&mut self, color_it: impl Iterator<Item = RGB>) {
        let zip_it = zip(self.idx.iter(), color_it);

        for (&idx, color) in zip_it {
            self.str[idx] = RGB_LUT[color.r as usize][0];
            self.str[idx + 1] = RGB_LUT[color.r as usize][1];
            self.str[idx + 2] = RGB_LUT[color.g as usize][0];
            self.str[idx + 3] = RGB_LUT[color.g as usize][1];
            self.str[idx + 4] = RGB_LUT[color.b as usize][0];
            self.str[idx + 5] = RGB_LUT[color.b as usize][1];
        }
    }
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

    //print!("{}", String::from_utf8_lossy(&fb.str));
    stream.write_all(&fb.str)?;
    Ok(())
}

/*#[allow(unused)]
fn flood_white(stream: &mut BufWriter<TcpStream>) -> std::io::Result<()> {
    for x in 1..X_SIZE {
        for y in 1..Y_SIZE {

        }
    }
    Ok(())
}*/

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("localhost:1337")?;
    stream.set_nodelay(false).expect("set_nodelay failed");
    let mut writer = BufWriter::with_capacity(1 << 22, stream);

    let mut frames: Vec<Vec<Vec<RGB>>> = Vec::new();
    for i in 1..13721 {
        let filename = format!("./images/bee_images_{:05}.ppm", i);
        let image = get_image(&String::from(filename));
        frames.push(image);
    }

    //loop {
    //    flood_white(&mut stream);
    //}
    const SCALE: usize = 2;

    let x = 10;
    let y = 50;

    let width: usize = SCALE * frames[0][0].len();
    let height: usize = SCALE * frames[0].len();

    let mut fb = FrameBuffer::new(x, y, width, height);

    println!("printing");
    loop {
        for frame in &frames {
            for _ in 1..2 {
                show_picture(&mut writer, &mut fb, &frame, SCALE)?;
            }
        }
    }
} // the stream is closed here
