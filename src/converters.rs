use std::{
    collections::HashMap,
    process::{Command, Output},
};

use anyhow::Context;

use crate::command::AsyncCommand;

pub struct Converter;

const FFMPEG: &'static str = "ffmpeg";

impl Converter {
    pub fn convert_aiff_to_mp3(
        input_file: &str,
        output_file: &str,
        metadata: &HashMap<&str, &String>,
    ) -> anyhow::Result<AsyncCommand> {
        // TODO get rid of all these to strings
        let mut args = vec![FFMPEG.to_string()];
        // Overwrite existing files
        args.push("-y".to_string());
        args.push("-i".to_string());
        args.push(input_file.to_string());

        for (k, v) in metadata.iter() {
            args.push("-metadata".to_string());
            args.push(format!("{}={}", k, v));
        }
        args.push(output_file.to_string());
        AsyncCommand::new(args)
    }
}
