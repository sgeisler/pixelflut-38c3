use std::net::TcpStream;
use std::io::*;
use std::cmp::max;
use lazy_static::lazy_static;

mod lut;

extern crate tinyppm;

const X_SIZE:usize = 3840;
const Y_SIZE:usize = 2160;

lazy_static! {
    static ref RGB_LUT:[[u8; 2]; 256] = lut::gen_rgb_lut();
    static ref POS_LUT:Vec<Box<[u8]>> = lut::gen_pos_lut(max(X_SIZE, Y_SIZE));
}

#[derive(Clone)] struct RGB {
    r:u8,
    g:u8,
    b:u8,
}

fn get_image(filename: &String) -> Vec<Vec<RGB>> {
    let ppm_image_result = tinyppm::ppm_loader::read_image_data(filename);
    let ppm_image = match ppm_image_result {
        Ok(image) => image,
        _ => panic!("unable to read specified image file!"),
    };

    let mut frame: Vec<Vec<RGB>> = vec![vec![RGB{r: 0, g: 0, b: 0}; ppm_image.width()]; ppm_image.height()];

    let mut x = 0;
    let mut y = 0;

    for px in ppm_image.pixels() {
        frame[y][x] = RGB{
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

#[inline] fn pixel(
    stream:&mut BufWriter<TcpStream>,
    x:u16,
    y:u16,
    px:&RGB
) -> std::io::Result<()> { 
/*
    let x_str = &POS_LUT[x as usize];
    let y_str = &POS_LUT[y as usize];
    let len = 12 + y_str.len() + x_str.len();

    let mut send_str = Vec::<u8>::with_capacity(len);

    send_str.extend(b"PX ");
    send_str.extend(&(POS_LUT[x as usize]));
    send_str.extend(b" ");
    send_str.extend(&(POS_LUT[y as usize]));
    send_str.extend(b" ");
    send_str.extend(&(RGB_LUT[px.r as usize]));
    send_str.extend(&(RGB_LUT[px.g as usize]));
    send_str.extend(&(RGB_LUT[px.b as usize]));
    send_str.extend(b"\n");
    stream.write_all(&send_str)?;
*/

    

    stream.write_all(b"PX ")?;
    stream.write_all(&(POS_LUT[x as usize]))?;
    stream.write_all(b" ")?;
    stream.write_all(&(POS_LUT[y as usize]))?;
    stream.write_all(b" ")?;
    stream.write_all(&(RGB_LUT[px.r as usize]))?;
    stream.write_all(&(RGB_LUT[px.g as usize]))?;
    stream.write_all(&(RGB_LUT[px.b as usize]))?;
    stream.write_all(b"\n")?;

    //println!("{}", send_str);
    //stream.write_all(send_str.as_bytes());
    Ok(())
}

fn show_picture(
    stream:&mut BufWriter<TcpStream>,
    x_pos:u16,
    y_pos:u16,
    buf:&Vec<Vec<RGB>>,
    scale:u16,
) -> std::io::Result<()> {
    let mut y = 0;
    for row in buf {
        let mut x = 0;
        for px in row {
            for x_i in 0..scale {
                for y_i in 0..scale {
                    pixel(stream, x_pos + x*scale + x_i, y_pos + y*scale + y_i, px)?;
                }
            }
            x += 1;
        }
        y += 1;
    }
    stream.flush()?;
    Ok(())
}

#[allow(unused)] fn flood_white(stream:&mut BufWriter<TcpStream>) -> std::io::Result<()> {
    for x in 1..X_SIZE {
        for y in 1..Y_SIZE {
            pixel(stream, x as u16, y as u16, &RGB{r:255, g:255, b:255})?;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("192.168.42.1:1337")?;
    stream.set_nodelay(false).expect("set_nodelay failed");
    let mut writer = BufWriter::with_capacity(1 << 22, stream);

    let mut frames : Vec<Vec<Vec<RGB>>> = Vec::new();
    for i in 1..13721 {
        let filename = format!("./images/bee_images_{:05}.ppm", i);
        let image = get_image(&String::from(filename));
        frames.push(image);
    }

    //loop {
    //    flood_white(&mut stream);
    //}

    println!("printing");
    loop {
        for frame in &frames {
            for _ in 1..2 {
                show_picture(&mut writer, 0, 0, &frame, 2)?;
            }
        }
    }
} // the stream is closed here
