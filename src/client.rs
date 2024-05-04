use once_cell::sync::Lazy;
use reqwest::{Client, Error, RequestBuilder};
use serde::Deserialize;
use serde_json::{json, Value};

pub struct Txt2ImgRequest {
    pub prompt: String,
    pub negative_prompt: String,
    pub width: u32,
    pub height: u32,
    pub seed: Option<u32>,
}

pub struct Img2ImgRequest {
    pub init_images: Vec<String>,
    pub prompt: String,
    pub negative_prompt: String,
    pub denoising_strength: f64,
    pub width: u32,
    pub height: u32,
    pub seed: Option<u32>,
}

#[derive(Deserialize, Clone)]
pub struct SdwebClientInfo {
    pub basepath: String,
    pub cf_access_client_id: Option<String>,
    pub cf_access_client_secret: Option<String>,
    pub host: String,
}

#[derive(Clone)]
pub struct SdwebClient {
    pub client: Client,
    pub info: SdwebClientInfo,
}

#[derive(Deserialize, Debug)]
pub struct ImagesResponse {
    pub images: Vec<String>,
}

impl SdwebClient {
    pub fn new(info: SdwebClientInfo) -> Self {
        let client = Client::new();
        Self { client, info }
    }

    fn post_client(&self, url: String, json: Value) -> RequestBuilder {
        let length = json.to_string().len();
        let mut request = self
            .client
            .post(&url)
            .header("Host", &self.info.host)
            .header("Content-Length", length.to_string())
            .json(&json);
        if let Some(cf_access_client_id) = &self.info.cf_access_client_id {
            request = request.header("CF-Access-Client-Id", cf_access_client_id);
        }
        if let Some(cf_access_client_secret) = &self.info.cf_access_client_secret {
            request = request.header("CF-Access-Client-Secret", cf_access_client_secret);
        }
        request
    }

    pub async fn img2img(&self, img2img: Img2ImgRequest) -> Result<ImagesResponse, Error> {
        let url = format!("{}/sdapi/v1/img2img", self.info.basepath);
        let json = json!({
            "init_images": img2img.init_images,
            "prompt": img2img.prompt,
            "negative_prompt": img2img.negative_prompt,
            "denoising_strength": img2img.denoising_strength,
            "width": img2img.width,
            "height": img2img.height,
            "save_images": true,
            "send_images": true,
            "seed": img2img.seed,
            "steps": 20
        });
        let request = self.post_client(url, json);
        let response = request.send().await?;
        let text = response.text().await;
        let response: ImagesResponse = serde_json::from_str(&text.unwrap()).unwrap();
        Ok(response)
    }
    pub async fn txt2img(&self, txt2img: Txt2ImgRequest) -> Result<ImagesResponse, Error> {
        let url = format!("{}/sdapi/v1/txt2img", self.info.basepath);
        let json = json!({
            "prompt": txt2img.prompt,
            "negative_prompt": txt2img.negative_prompt,
            "width": txt2img.width,
            "height": txt2img.height,
            "seed": txt2img.seed,
        });
        let request = self.post_client(url, json);
        let response = request.send().await?;
        let response: ImagesResponse = response.json().await?;
        Ok(response)
    }
}

pub static SD_WEB_ENV: Lazy<SdwebClientInfo> = Lazy::new(|| {
    use dotenvy::dotenv;
    use std::env::var;
    dotenv().unwrap();
    SdwebClientInfo {
        basepath: var("SDWEBUI_ENDPOINT").unwrap(),
        cf_access_client_id: var("CF_ACCESS_CLIENT_ID").ok(),
        cf_access_client_secret: var("CF_ACCESS_CLIENT_SECRET").ok(),
        host: var("HOST").unwrap(),
    }
});
