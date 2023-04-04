use indicatif::ProgressBar;
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_dir, remove_file},
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

    #[structopt(long, short)]
    zip: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let srcdir = opt.srcdir;
    let outdir = opt.outdir;
    let audiobook_name = opt.name;
    let audiobook_author = opt.author;
    let zipit = opt.zip;

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
    all_files.sort_by(|a, b| a.file_name().partial_cmp(&b.file_name()).unwrap());

    println!("== Converting {} files. ==", all_files.len());
    let ui = ConsoleUi::new()?;

    let mut set: JoinSet<anyhow::Result<UiCommandId>> = JoinSet::new();
    for (idx, file) in all_files.iter().enumerate() {
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
            let track = format!("{}", idx);
            let metadata = HashMap::from([
                ("title", &chapter_name),
                ("album", &audiobook_name),
                ("artist", &audiobook_author),
                ("track", &track),
            ]);
            let convert_cmd = Converter::convert_aiff_to_mp3(&aiff, &mp3out, &metadata)?;
            convert_cmd.run().await?;

            remove_file(aiff)?;
            Ok(ui_id)
        });
    }

    while let Some(res) = set.join_next().await {
        let ui_id = res??;
        ui.finish_command(ui_id)?;
    }

    if zipit {
        let zip_out = format!("{}/{}.zip", &outdir, &audiobook_name);
        let ui_id = ui.start_command(format!("Zipping into {}", &zip_out))?;
        let args = vec!["zip", "-r", &zip_out, &outdir]
            .into_iter()
            .map(String::from)
            .collect();
        let zip_cmd = AsyncCommand::new(args)?;
        zip_cmd.run().await?;
        ui.finish_command(ui_id)?;
    }
    println!("== All Done! ==");

    Ok(())
}
