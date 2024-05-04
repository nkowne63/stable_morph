use crate::{
    client::{Img2ImgRequest, SdwebClient, SdwebClientInfo},
    images::{base64_to_png, path_modifier, png_to_base64},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DenoisingSteps {
    pub init: f64,
    pub end: f64,
    pub steps: f64,
}

#[derive(Deserialize)]
pub struct PromptChunks {
    pub chunks: Vec<String>,
}

impl PromptChunks {
    pub fn diff_by(&self, other: &PromptChunks) -> Vec<String> {
        let mut diff: Vec<String> = Vec::new();
        for chunk in &self.chunks {
            if !other.chunks.contains(chunk) {
                diff.push(chunk.clone());
            }
        }
        diff
    }
    pub fn common(&self, other: &PromptChunks) -> Vec<String> {
        let mut common: Vec<String> = Vec::new();
        for chunk in &self.chunks {
            if other.chunks.contains(chunk) {
                common.push(chunk.clone());
            }
        }
        common
    }
}

#[derive(Deserialize)]
pub struct MorphPreset {
    pub prompt_chunks: PromptChunks,
    pub denoising_steps: DenoisingSteps,
    pub seed: Option<u32>,
}

#[derive(Deserialize, Clone)]
pub struct RestartIndex {
    pub preset: usize,
    pub chunk_transition: usize,
    pub strength: usize,
}

#[derive(Deserialize)]
pub struct Instruction {
    pub client_info: SdwebClientInfo,
    pub init_image_path: String,
    pub common_negative_prompt: String,
    pub morph_presets: Vec<MorphPreset>,
    pub restart_index: Option<RestartIndex>,
}

pub async fn morph(instruction: Instruction) {
    let client = SdwebClient::new(instruction.client_info);
    let mut last_output_path = instruction.init_image_path.clone();
    let mut origin_image_base64 = png_to_base64(last_output_path.clone());
    let mut previous_prompt_chunks = PromptChunks { chunks: Vec::new() };
    let image_size = crate::images::size(instruction.init_image_path.clone());
    let restart_index = instruction.restart_index.clone();
    // for each presets
    let mut preset_idx = 0;
    for preset in instruction.morph_presets {
        let common_chunks = preset.prompt_chunks.common(&previous_prompt_chunks);
        let new_chunks = preset.prompt_chunks.diff_by(&previous_prompt_chunks);
        let old_chunks = previous_prompt_chunks.diff_by(&preset.prompt_chunks);
        // for each chunk transition
        let mut chunk_transition_idx = 0;
        let mut chunk_transition = (Vec::new(), old_chunks);
        while chunk_transition.1.len() > 0 || chunk_transition.0.len() < new_chunks.len() {
            let new_chunk = new_chunks.get(chunk_transition.0.len()).clone();
            let old_chunk = chunk_transition.1.get(0).clone();
            if preset_idx == 0 {
                chunk_transition.0 = new_chunks.clone();
                // println!("{:?}", chunk_transition.0);
                // panic!("stop")
            } else if let Some(new_chunk) = new_chunk {
                chunk_transition.0.push(new_chunk.clone());
            }
            if let Some(_) = old_chunk {
                chunk_transition.1.remove(0);
            }
            // for each denoising strength
            let mut strength = preset.denoising_steps.init;
            let seed = preset.seed.unwrap_or(rand::random());
            let mut strength_idx = 0;
            let mut waitings = Vec::new();
            while strength <= preset.denoising_steps.end {
                let mut target_prompts = common_chunks.clone();
                target_prompts.extend(chunk_transition.0.clone());
                target_prompts.extend(chunk_transition.1.clone());
                let target_prompts = target_prompts.join(",");
                let output_path = path_modifier(
                    instruction.init_image_path.clone(),
                    &format!("_{}_{}_{}", preset_idx, chunk_transition_idx, strength_idx),
                );
                // indicator
                println!(
                    "start\n\tpreset index: {}\n\tchunk transition index: {}\n\tstrength index: {}",
                    preset_idx, chunk_transition_idx, strength_idx
                );
                println!(
                    "run img2img with\n\tprompts: {}\n\tstrength: {}\n\tseed: {}",
                    target_prompts, strength, seed
                );
                println!("output path: {}\n", output_path);
                // run img2img
                let job_strength = strength.clone();
                let job_output_path = output_path.clone();
                let job_origin_image_base64 = origin_image_base64.clone();
                let job_client = client.clone();
                let job_common_negative_prompt = instruction.common_negative_prompt.clone();
                if !(restart_index.is_some()
                    && !(restart_index.clone().unwrap().preset <= preset_idx
                        || restart_index.clone().unwrap().chunk_transition <= chunk_transition_idx
                        || restart_index.clone().unwrap().strength <= strength_idx))
                {
                    let job = async move {
                        let response = job_client
                            .img2img(Img2ImgRequest {
                                init_images: vec![job_origin_image_base64],
                                prompt: target_prompts,
                                negative_prompt: job_common_negative_prompt,
                                denoising_strength: job_strength,
                                width: image_size.0,
                                height: image_size.1,
                                seed: Some(seed),
                            })
                            .await
                            .expect("img2img failed");
                        let output_image_base64 = response.images[0].clone();
                        // save output image
                        base64_to_png(output_image_base64, job_output_path)
                            .expect("save image failed");
                    };
                    // job.await;
                    waitings.push(job);
                } else {
                    println!("skip img2img")
                }
                // update strength
                strength += preset.denoising_steps.steps;
                strength = (strength * 1000.0).round() / 1000.0;
                strength_idx += 1;
                // update last output path
                last_output_path = output_path.clone();
            }
            futures::future::join_all(waitings).await;
            // update origin image
            origin_image_base64 = png_to_base64(last_output_path.clone());
            chunk_transition_idx += 1;
        }
        previous_prompt_chunks = preset.prompt_chunks;
        preset_idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_chunks_diff_by() {
        let chunks1 = PromptChunks {
            chunks: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let chunks2 = PromptChunks {
            chunks: vec!["b".to_string(), "c".to_string(), "d".to_string()],
        };
        let diff = chunks1.diff_by(&chunks2);
        assert_eq!(diff, vec!["a".to_string()]);
    }

    #[test]
    fn test_prompt_chunks_common() {
        let chunks1 = PromptChunks {
            chunks: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let chunks2 = PromptChunks {
            chunks: vec!["b".to_string(), "c".to_string(), "d".to_string()],
        };
        let common = chunks1.common(&chunks2);
        assert_eq!(common, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn test_join_str() {
        let chunks = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let joined = chunks.join(",");
        assert_eq!(joined, "a,b,c");
    }
}
