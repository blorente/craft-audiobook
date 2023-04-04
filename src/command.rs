use std::{
    fmt::format,
    process::{Command, Output},
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use anyhow::Context;
use indicatif::ProgressBar;

pub struct AsyncCommand {
    program: String,
    args: Vec<String>,
}

impl AsyncCommand {
    pub fn new(args: Vec<String>) -> anyhow::Result<Self> {
        let error = anyhow!("Couldn't extract program from args: {:?}", &args);
        let mut args_iter = args.into_iter();
        let program = args_iter.next().ok_or(error)?;
        let args = args_iter.collect();
        Ok(Self { program, args })
    }

    pub async fn run(&self) -> anyhow::Result<Output> {
        // TODO BL: Log the command as it's running.
        let output = Command::new(&self.program)
            .args(&self.args)
            .output()
            .context("Failed to run command")?;

        // println!("Running {}", &self.message());
        Ok(output)
    }

    pub fn message(&self) -> String {
        format!("{} {}", self.program, self.args.join(" "))
    }
}
