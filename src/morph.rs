pub struct InitImage {
    pub prompts: Vec<String>,
    pub negative_prompt: String,
    pub width: u32,
    pub height: u32,
    pub seed: Option<u32>,
}

pub struct MorphPrompts {
    pub prompts: Vec<String>,
    pub denoising_strength: f64,
}
