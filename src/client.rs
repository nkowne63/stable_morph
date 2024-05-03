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

pub struct SdwebClientInfo {
    pub basepath: String,
    pub cf_access_client_id: Option<String>,
    pub cf_access_client_secret: Option<String>,
    pub host: String,
}

pub struct SdwebClient {
    pub client: reqwest::Client,
    pub info: SdwebClientInfo,
}

#[derive(serde::Deserialize, Debug)]
pub struct Img2ImgResponse {
    pub images: Vec<String>,
}

impl SdwebClient {
    pub fn new(info: SdwebClientInfo) -> Self {
        let client = reqwest::Client::new();
        Self { client, info }
    }

    fn post_client(&self, url: String, json: Value) -> reqwest::RequestBuilder {
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

    pub async fn img2img(
        &self,
        img2img: &Img2ImgRequest,
    ) -> Result<Img2ImgResponse, reqwest::Error> {
        let url = format!("{}/sdapi/v1/img2img", self.info.basepath);
        let json = serde_json::json!({
            "init_images": img2img.init_images,
            "prompt": img2img.prompt,
            "negative_prompt": img2img.negative_prompt,
            "denoising_strength": img2img.denoising_strength,
            "width": img2img.width,
            "height": img2img.height,
            "save_images": true,
            "send_images": true,
            "seed": img2img.seed
        });
        let request = self.post_client(url, json);
        let response = request.send().await?;
        let response: Img2ImgResponse = response.json().await?;
        Ok(response)
    }
    pub async fn txt2img(
        &self,
        txt2img: &Txt2ImgRequest,
    ) -> Result<Img2ImgResponse, reqwest::Error> {
        let url = format!("{}/sdapi/v1/txt2img", self.info.basepath);
        let json = serde_json::json!({
            "prompt": txt2img.prompt,
            "negative_prompt": txt2img.negative_prompt,
            "width": txt2img.width,
            "height": txt2img.height,
            "seed": txt2img.seed
        });
        let request = self.post_client(url, json);
        let response = request.send().await?;
        let response: Img2ImgResponse = response.json().await?;
        Ok(response)
    }
}

use once_cell::sync::Lazy;
use serde_json::Value;

pub static SD_WEB_ENV: Lazy<SdwebClientInfo> = Lazy::new(|| {
    use dotenvy::dotenv;
    dotenv().unwrap();
    SdwebClientInfo {
        basepath: std::env::var("SDWEBUI_ENDPOINT").unwrap(),
        cf_access_client_id: std::env::var("CF_ACCESS_CLIENT_ID").ok(),
        cf_access_client_secret: std::env::var("CF_ACCESS_CLIENT_SECRET").ok(),
        host: std::env::var("HOST").unwrap(),
    }
});
