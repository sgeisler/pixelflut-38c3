pub fn gen_pos_lut(max:usize) -> Vec<Box<[u8]>> {
    let lookup:Vec<Box<[u8]>> = (0..=max).map(|x| format!("{}", x).as_bytes().into()).collect();
    lookup
}

pub fn gen_rgb_lut() -> [[u8; 2]; 256] {
    let lookup:[[u8; 2]; 256] = (0..256).map(|x| format!("{:02x}", x).as_bytes().try_into().unwrap()).collect::<Vec<_>>().try_into().unwrap();
    lookup
}

