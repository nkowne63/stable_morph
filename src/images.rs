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

pub fn size(path: String) -> (u32, u32) {
    let img = ImageReader::open(path).unwrap().decode().unwrap();
    (img.width(), img.height())
}

pub fn path_modifier(path: String, modifier: &str) -> String {
    let parts: Vec<&str> = path.split(".").collect();
    let mut new_path = String::from(parts[0]);
    new_path.push_str(modifier);
    for part in parts[1..].iter() {
        new_path.push_str(".");
        new_path.push_str(part);
    }
    new_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_modifier() {
        let path = String::from("test.tmp.png");
        let modifier = "_mod";
        let new_path = path_modifier(path, modifier);
        assert_eq!(new_path, "test_mod.tmp.png");
    }
}
