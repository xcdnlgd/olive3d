use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
};

pub fn save_buffer_to_ppm_file(
    buffer: &[u32],
    width: u32,
    height: u32,
    stride: u32,
    path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut file = BufWriter::new(file);
    write!(file, "P6\n{} {} 255\n", width, height)?;
    for y in 0..height {
        for x in 0..width {
            let pixel = buffer[(y * stride + x) as usize];
            let rgb: [u8; 3] = [
                ((pixel) & 0xFF) as u8,
                ((pixel >> 8) & 0xFF) as u8,
                ((pixel >> (8 * 2)) & 0xFF) as u8,
            ];
            file.write_all(&rgb)?;
        }
    }
    Ok(())
}

#[derive(PartialEq)]
enum NowReading {
    MagicNumber,
    Width,
    Height,
    MaxVal,
    Data,
}

pub fn load_ppm_file_to_buffer(path: impl AsRef<Path>) -> Image {
    let file = File::open(path).unwrap();
    let mut now_reading = NowReading::MagicNumber;
    let mut file = BufReader::new(file);
    let mut magic_number;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut max_val: u32 = 0;
    while now_reading != NowReading::Data {
        let mut line = String::new();
        file.read_line(&mut line).unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        for &part in parts.iter() {
            match now_reading {
                NowReading::MagicNumber => {
                    magic_number = part.to_string();
                    if magic_number != "P6" {
                        panic!("unsupported {magic_number}");
                    }
                    now_reading = NowReading::Width;
                }
                NowReading::Width => {
                    width = part.parse().unwrap();
                    now_reading = NowReading::Height;
                }
                NowReading::Height => {
                    height = part.parse().unwrap();
                    now_reading = NowReading::MaxVal;
                }
                NowReading::MaxVal => {
                    max_val = part.parse().unwrap();
                    now_reading = NowReading::Data;
                }
                NowReading::Data => {
                    panic!()
                }
            }
        }
    }
    println!("load ppm, width: {width}, height: {height}, max_val: {max_val}");
    let mut buffer: Vec<u32> = Vec::with_capacity((width * height) as usize);
    let mut rgb = [0u8; 3];
    while file.read_exact(&mut rgb).is_ok() {
        let mut pixel: u32 = 0xff000000;
        for (i, &x) in rgb.iter().enumerate() {
            pixel |= ((x as u32) & 0xff) << (8 * i);
        }
        buffer.push(pixel);
    }
    assert_eq!(buffer.len(), (width * height) as usize);
    Image {
        buffer,
        width,
        height,
    }
}

pub struct Image {
    pub buffer: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl Image {
    pub fn vflip(&mut self) {
        for y in 0..self.height / 2 {
            for x in 0..self.width {
                let a = x + y * self.width;
                let b = x + (self.height - 1 - y) * self.width;
                self.buffer.swap(a as usize, b as usize);
            }
        }
    }
}
