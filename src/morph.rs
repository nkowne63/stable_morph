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

#[derive(serde::Deserialize, Debug)]
struct Img2ImgResponse {
    images: Vec<String>,
}

pub fn apply_img2img_morphing(image_path: &str, strength: f64, output_path: &str, seed: u32) {
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
        "width": env::var("DEFAULT_WIDTH").unwrap().parse::<u32>().unwrap(),
        "height": env::var("DEFAULT_HEIGHT").unwrap().parse::<u32>().unwrap(),
        "save_images": true,
        "send_images": true,
        "steps": 20,
        "seed": seed
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

pub fn morphing(
    init_strength: f64,
    final_strength: f64,
    start_path: &str,
    steps: u32,
    naming: impl Fn(u32) -> String,
) {
    let mut image_path = start_path.to_string();
    for i in 0..steps {
        let output_path = naming(i);
        let strength =
            init_strength + (final_strength - init_strength) / (steps as f64) * (i as f64);
        apply_img2img_morphing(&image_path, strength, &output_path, 100);
        println!("{} / {} done > {}", i + 1, steps, output_path);
        image_path = output_path;
    }
    println!("Morphing done");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_to_base64() {
        let path = "test.png";
        png_to_base64(path);
        // println!("{}", png_to_base64(path));
    }

    #[test]
    fn test_base64_to_png() {
        let path = "test.png";
        let base64 = png_to_base64(path);
        base64_to_png(base64, "test_output.tmp.png".to_string()).unwrap();
    }

    #[test]
    fn test_apply_img2img_morphing() {
        let image_path = "test.png";
        let strength = 0.8;
        apply_img2img_morphing(image_path, strength, "output.tmp.png", 100);
    }

    #[test]
    fn test_morphing() {
        use dotenvy::dotenv;
        dotenv().unwrap();
        use std::env;
        let init_num = env::var("INIT_NUM").unwrap().parse::<u32>().unwrap();
        let init_strength = env::var("INIT_STRENGTH").unwrap().parse::<f64>().unwrap();
        let final_strength = env::var("FINAL_STRENGTH").unwrap().parse::<f64>().unwrap();
        let steps = env::var("STEPS").unwrap().parse::<u32>().unwrap();
        let init_image = format!("output_{}.tmp.png", init_num);
        let naming_format = |i: u32| format!("output_{}.tmp.png", i + init_num + 1);
        morphing(
            init_strength,
            final_strength,
            &init_image,
            steps,
            naming_format,
        );
    }
}
