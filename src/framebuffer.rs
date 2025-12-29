use std::{io::Write, iter::zip, vec};

use rand::seq::SliceRandom;

const HEX_LUT: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];

pub struct FrameBuffer {
    pub str: Vec<u8>,
    idx: Vec<usize>,
}

impl FrameBuffer {
    #[allow(unused)]
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> FrameBuffer {
        let n = w * h;
        let mut fb = FrameBuffer {
            str: Vec::new(),
            idx: vec![0; n],
        };

        println!("built framebuffer: x: {x} y: {y} w: {w} h: {h}");

        let mut positions = (y..y + h)
            .flat_map(|y_pos| (x..x + w).map(move |x_pos| (x_pos, y_pos)))
            .enumerate()
            .collect::<Vec<_>>();
        positions.shuffle(&mut rand::thread_rng());

        for (pixel_idx, (x_pos, y_pos)) in positions {
            let str1 = format!("PX {x_pos} {y_pos} ");

            fb.str.extend_from_slice(str1.as_bytes());
            fb.idx[pixel_idx] = fb.str.len();
            fb.str.extend_from_slice(b"xxxxxx\n");
        }

        fb
    }

    #[inline]
    pub fn fill(&mut self, color_it: impl Iterator<Item = super::RGB>) {
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

    pub fn write_to(&self, mut stream: impl Write) -> Result<(), std::io::Error> {
        stream.write_all(&self.str)?;
        Ok(())
    }
}
