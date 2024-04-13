pub fn png_to_base64(path: &str) -> String {
    let img = ImageReader::open(path).unwrap().decode().unwrap();
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();
    general_purpose::STANDARD.encode(&bytes)
}

use base64::{engine::general_purpose, Engine};
use image::{io::Reader as ImageReader, ImageResult};
use std::io::Cursor;

pub fn base64_to_png(base64: String, path: String) -> ImageResult<String> {
    let bytes = general_purpose::STANDARD.decode(base64).unwrap();
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;
    img.save(path.clone())?;
    Ok(path)
}

#[derive(serde::Deserialize)]
struct Img2ImgResponse {
    images: Vec<String>,
}

pub fn apply_img2img_morphing(image_path: &str, strength: f64, output_path: &str) {
    use dotenvy::dotenv;
    dotenv().unwrap();
    use std::env;
    let base64 = png_to_base64(image_path);
    let client = reqwest::blocking::Client::new();
    let basepath = env::var("SDWEBUI_ENDPOINT").unwrap();
    let url = format!("{}/sdapi/v1/img2img", basepath);
    let json = &serde_json::json!({
        "init_images": [base64],
        "prompt": env::var("TEST_IMG2IMG_PROMPT").unwrap(),
        "negative_prompt": env::var("DEFAULT_NEGATIVE_PROMPT").unwrap(),
        "denoising_strength": strength,
        "width": 1024,
        "height": 1024,
        "sampler_name": "DPM++ 2M Karras",
        "save_images": true,
        "send_images": true,
        "steps": 20
    });
    let length = json.to_string().len();
    let request = client
        .post(&url)
        .header(
            "CF-Access-Client-Id",
            env::var("CF_ACCESS_CLIENT_ID").unwrap(),
        )
        .header(
            "CF-Access-Client-Secret",
            env::var("CF_ACCESS_CLIENT_SECRET").unwrap(),
        )
        .header("Host", env::var("HOST").unwrap())
        .header("Content-Length", length.to_string())
        .json(json);
    // println!("{:#?}", request);
    let response = request.send().unwrap();
    // println!("{:#?}", response.text().unwrap());
    // panic!("stop");
    let response: Img2ImgResponse = response.json().unwrap();
    let base64 = &response.images[0];
    base64_to_png(base64.clone(), output_path.to_string()).unwrap();
}

pub fn morphing(strength: f64) {
    let steps = 7;
    let mut image_path = "test.tmp.png".to_string();
    for i in 0..steps {
        let output_path = format!("output_{}.tmp.png", i);
        println!("{} start", i);
        println!("image_path: {}", image_path);
        println!("output_path: {}", output_path);
        apply_img2img_morphing(&image_path, strength, &output_path);
        image_path = output_path;
        println!("{} done", i)
    }
    println!("Morphing done");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_to_base64() {
        let path = "test.tmp.png";
        println!("{}", png_to_base64(path));
    }

    #[test]
    fn test_base64_to_png() {
        let path = "test.tmp.png";
        let base64 = png_to_base64(path);
        base64_to_png(base64, "test_output.tmp.png".to_string()).unwrap();
    }

    #[test]
    fn test_apply_img2img_morphing() {
        let image_path = "test.tmp.png";
        let strength = 0.6;
        apply_img2img_morphing(image_path, strength, "output.tmp.png");
    }

    #[test]
    fn test_morphing() {
        morphing(0.5);
    }
}
