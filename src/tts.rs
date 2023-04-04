use std::{
    env::temp_dir,
    io::{Bytes, Read},
    path::PathBuf,
    process::Command,
};

use anyhow::anyhow;

pub(crate) struct TTS {}

pub(crate) struct AiffSoundFile {
    // TODO yes, this should be a path, sue me.
    pub path: String,
}

impl TTS {
    pub fn new() -> anyhow::Result<Self> {
        // let tts = Tts::default()?;
        let tmp_dir = temp_dir();
        Ok(Self {})
    }

    pub async fn say(&self, input_file: &str, outdir: &str) -> anyhow::Result<AiffSoundFile> {
        let outfile = format!("{}/testfile.aiff", outdir);
        let output = Command::new("say")
            .arg(&format!("--input-file={}", input_file))
            .arg(&format!("--output-file={}", outfile))
            .output()
            .expect("failed to execute process");

        assert!(output.status.success());
        Ok(AiffSoundFile { path: outfile })
    }
}
