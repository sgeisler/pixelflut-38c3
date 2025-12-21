pub fn gen_rgb_lut() -> [[u8; 2]; 256] {
    let lookup: [[u8; 2]; 256] = (0..256)
        .map(|x| format!("{:02x}", x).as_bytes().try_into().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    lookup
}
