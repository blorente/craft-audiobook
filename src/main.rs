use indicatif::ProgressBar;
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_dir},
    sync::{Arc, Mutex},
    thread::{self},
};
use tokio::task::{JoinHandle, JoinSet};
use ui::UiCommandId;

use anyhow::Context;
use structopt::*;

use crate::{command::AsyncCommand, converters::Converter, tts::TTS, ui::ConsoleUi};

mod command;
mod converters;
mod tts;
mod ui;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, short)]
    srcdir: String,

    #[structopt(long, short)]
    outdir: String,

    #[structopt(long, short)]
    name: String,

    #[structopt(long, short)]
    author: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let srcdir = opt.srcdir;
    let outdir = opt.outdir;
    let audiobook_name = opt.name;
    let audiobook_author = opt.author;

    create_dir_all(&outdir)?;

    let mut all_files = vec![];
    for file in read_dir(&srcdir)? {
        if let Ok(file) = file {
            if let Ok(filetype) = file.file_type() {
                if filetype.is_file() {
                    all_files.push(file);
                }
            }
        }
    }

    println!("== Converting {} files. ==", all_files.len());
    let ui = ConsoleUi::new()?;

    let mut set: JoinSet<anyhow::Result<UiCommandId>> = JoinSet::new();
    for file in all_files.iter() {
        let path = file.path();
        let basename = path
            .file_name()
            .context("Error getting the file name for file")?
            .to_str()
            .context("Error turning the file name to a string")?;

        let extension = path.extension().context("TODO")?.to_str().context("TODO")?;
        let dotted_extension = format!(".{}", extension);
        let chapter_name = basename.replace(&dotted_extension, "");

        let outdir = outdir.clone();
        let audiobook_author = audiobook_author.clone();
        let audiobook_name = audiobook_name.clone();

        let ui = ui.clone();

        // TODO split into and then futures, so that I can have finer grained UI control
        set.spawn(async move {
            let pathstr = path.to_str().unwrap();
            let mp3out = format!("{}/{}.mp3", &outdir, &chapter_name);
            let ui_id = ui.start_command(format!("Converting {} to {}", &chapter_name, &mp3out))?;

            let tts = TTS::new().expect("TODO");
            let aiff = tts.say(&pathstr, &outdir).await.expect("TODO").path;

            let metadata = HashMap::from([
                ("title", &chapter_name),
                ("album", &audiobook_name),
                ("author", &audiobook_author),
            ]);
            let convert_cmd = Converter::convert_aiff_to_mp3(&aiff, &mp3out, &metadata)?;
            convert_cmd.run().await?;
            Ok(ui_id)
        });
    }

    while let Some(res) = set.join_next().await {
        let ui_id = res??;
        ui.finish_command(ui_id)?;
    }
    println!("== All Done! ==");

    Ok(())
}
