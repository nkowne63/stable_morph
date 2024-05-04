use crate::{
    client::{SdwebClient, SdwebClientInfo},
    images::{path_modifier, png_to_base64},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct DenoisingSteps {
    pub init: f64,
    pub end: f64,
    pub steps: f64,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct MorphPreset {
    pub prompt_chunks: PromptChunks,
    pub denoising_steps: DenoisingSteps,
    pub seed: Option<u32>,
}

#[derive(Serialize)]
pub struct Instruction {
    pub client_info: SdwebClientInfo,
    pub init_image_path: String,
    pub common_negative_prompt: String,
    pub morph_presets: Vec<MorphPreset>,
}

pub async fn morph(instruction: Instruction) {
    // let client = SdwebClient::new(instruction.client_info);
    // let mut origin_image_base64 = png_to_base64(instruction.init_image_path.clone());
    let mut previous_prompt_chunks = PromptChunks { chunks: Vec::new() };
    // for each presets
    let mut preset_idx = 0;
    for preset in instruction.morph_presets {
        let common_chunks = preset.prompt_chunks.common(&previous_prompt_chunks);
        let new_chunks = preset.prompt_chunks.diff_by(&previous_prompt_chunks);
        let old_chunks = previous_prompt_chunks.diff_by(&preset.prompt_chunks);
        // for each chunk transition
        let mut chunk_transition_idx = 0;
        let mut chunk_transition = (Vec::new(), old_chunks);
        while chunk_transition.1.len() > 0 && chunk_transition.0.len() < new_chunks.len() {
            let new_chunk = new_chunks.get(chunk_transition.0.len()).clone();
            let old_chunk = chunk_transition.1.get(0).clone();
            if let Some(new_chunk) = new_chunk {
                chunk_transition.0.push(new_chunk.clone());
            }
            if let Some(_) = old_chunk {
                chunk_transition.1.remove(0);
            }
            // for each denoising strength
            let mut strength = preset.denoising_steps.init;
            let seed = preset.seed.unwrap_or(rand::random());
            let mut strength_idx = 0;
            while strength <= preset.denoising_steps.end {
                let mut target_prompts = common_chunks.clone();
                target_prompts.extend(chunk_transition.0.clone());
                target_prompts.extend(chunk_transition.1.clone());
                let target_prompts = target_prompts.join(",");
                let output_path = path_modifier(
                    instruction.init_image_path.clone(),
                    &format!(
                        "_morph_{}_{}_{}",
                        preset_idx, chunk_transition_idx, strength_idx
                    ),
                );
                // indicator
                println!(
                    "start preset index: {}, chunk transition index: {}, strength index: {}",
                    preset_idx, chunk_transition_idx, strength_idx
                );
                println!(
                    "run img2img with prompts: {}, strength: {}, seed: {}",
                    target_prompts, strength, seed
                );
                println!("output path: {}", output_path);
                // run img2img
                // update strength
                strength += preset.denoising_steps.steps;
                strength_idx += 1;
            }
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
}
