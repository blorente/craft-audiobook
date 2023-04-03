use std::{
    collections::HashMap,
    process::{Command, Output},
};

use anyhow::Context;

pub struct Converter;

const FFMPEG: &'static str = "ffmpeg";

impl Converter {
    pub fn convert_aiff_to_mp3(
        input_file: &str,
        output_file: &str,
        metadata: &HashMap<&str, &String>,
    ) -> anyhow::Result<Output> {
        let mut binding = Command::new(FFMPEG);
        let command = binding.arg("-i").arg(input_file);

        for (k, v) in metadata.iter() {
            command.arg("-metadata").arg(&format!("{}={}", k, v));
        }
        command
            .arg(output_file)
            .output()
            .context("failed to convert aiff to mp3")
        // .and_then(|_| Ok(()))
    }
}
