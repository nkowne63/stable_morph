use base64::{engine::general_purpose, Engine};
use image::{io::Reader as ImageReader, ImageResult};
use std::io::Cursor;

pub fn png_to_base64(path: String) -> String {
    let img = ImageReader::open(path).unwrap().decode().unwrap();
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();
    general_purpose::STANDARD.encode(&bytes)
}

pub fn base64_to_png(base64: String, path: String) -> ImageResult<String> {
    let bytes = general_purpose::STANDARD.decode(base64).unwrap();
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;
    img.save(path.clone())?;
    Ok(path)
}
