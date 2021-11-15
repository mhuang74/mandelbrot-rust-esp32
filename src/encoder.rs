/// BMP encoder, courtesy of https://github.com/image-rs/image/blob/master/src/codecs/bmp/encoder.rs
/// Patched to work on esp32 without alloc error

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Write};
use anyhow::{Result};
use log::*;

const BITMAPFILEHEADER_SIZE: u32 = 14;
const BITMAPINFOHEADER_SIZE: u32 = 40;
const BITMAPV4HEADER_SIZE: u32 = 108;


/// encode grayscale Bitmap image
pub fn encode_grayscale(
    writer: &mut dyn Write,
    image: &[u8],
    width: u32,
    height: u32,
) -> Result<()> {


    let bmp_header_size = BITMAPFILEHEADER_SIZE;

    let (dib_header_size, written_pixel_size, palette_color_count) = (BITMAPINFOHEADER_SIZE, 1, 256);
    let row_pad_size = (4 - (width * written_pixel_size) % 4) % 4; // each row must be padded to a multiple of 4 bytes
    let image_size = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(written_pixel_size))
        .and_then(|v| v.checked_add(height * row_pad_size)).unwrap();
    let palette_size = palette_color_count * 4; // all palette colors are BGRA
    let file_size = bmp_header_size + dib_header_size + palette_size + image_size;

    info!("image_size: {}\t palette_size: {}\t file_size: {}\t row_pad_size: {}", image_size, palette_size, file_size, row_pad_size);

    // write BMP header
    writer.write_u8(b'B')?;
    writer.write_u8(b'M')?;
    writer.write_u32::<LittleEndian>(file_size)?; // file size
    writer.write_u16::<LittleEndian>(0)?; // reserved 1
    writer.write_u16::<LittleEndian>(0)?; // reserved 2
    writer
        .write_u32::<LittleEndian>(bmp_header_size + dib_header_size + palette_size)?; // image data offset

    // write DIB header
    writer.write_u32::<LittleEndian>(dib_header_size)?;
    writer.write_i32::<LittleEndian>(width as i32)?;
    writer.write_i32::<LittleEndian>(height as i32)?;
    writer.write_u16::<LittleEndian>(1)?; // color planes
    writer
        .write_u16::<LittleEndian>((written_pixel_size * 8) as u16)?; // bits per pixel
    if dib_header_size >= BITMAPV4HEADER_SIZE {
        // Assume BGRA32
        writer.write_u32::<LittleEndian>(3)?; // compression method - bitfields
    } else {
        writer.write_u32::<LittleEndian>(0)?; // compression method - no compression
    }
    writer.write_u32::<LittleEndian>(image_size)?;
    writer.write_i32::<LittleEndian>(0)?; // horizontal ppm
    writer.write_i32::<LittleEndian>(0)?; // vertical ppm
    writer.write_u32::<LittleEndian>(palette_color_count)?;
    writer.write_u32::<LittleEndian>(0)?; // all colors are important
    if dib_header_size >= BITMAPV4HEADER_SIZE {
        // Assume BGRA32
        writer.write_u32::<LittleEndian>(0xff << 16)?; // red mask
        writer.write_u32::<LittleEndian>(0xff << 8)?; // green mask
        writer.write_u32::<LittleEndian>(0xff)?; // blue mask
        writer.write_u32::<LittleEndian>(0xff << 24)?; // alpha mask
        writer.write_u32::<LittleEndian>(0x73524742)?; // colorspace - sRGB

        // endpoints (3x3) and gamma (3)
        for _ in 0..12 {
            writer.write_u32::<LittleEndian>(0)?;
        }
    }

    //// write image data
    
    // write grayscale palette
    for val in 0u8..=128 {
        // each color is written as BGRA, where A is always 0 and since only grayscale is being written, B = G = R = index
        // !! NOTE: change from write_all to write() to avoid alloc errors on esp32
        writer.write(&[val, val, val, 0])?;
    }
    
    // write bitmap

    
    for row in (0..height).rev() {
        // from the bottom up
        let row_start = row * width;
        for col in 0..width {
            let pixel_start = (row_start + col) as usize;
            // color value is equal to the palette index
            writer.write_u8(image[pixel_start])?;
            // alpha is never written as it's not widely supported
        }

        // write_row_pad
        for _ in 0..row_pad_size {
            writer.write_u8(0)?;
        }
    }

    Ok(())
}




