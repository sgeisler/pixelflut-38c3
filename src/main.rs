use std::io::*;
use std::iter::zip;
use std::net::TcpStream;

extern crate tinyppm;

const HEX_LUT: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];

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
            self.str[idx] = HEX_LUT[(color.r >> 4 & 0xF) as usize];
            self.str[idx + 1] = HEX_LUT[(color.r & 0xF) as usize];
            self.str[idx + 2] = HEX_LUT[(color.g >> 4 & 0xF) as usize];
            self.str[idx + 3] = HEX_LUT[(color.g & 0xF) as usize];
            self.str[idx + 4] = HEX_LUT[(color.b >> 4 & 0xF) as usize];
            self.str[idx + 5] = HEX_LUT[(color.b & 0xF) as usize];
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
