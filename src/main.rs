use indicatif::ProgressBar;
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_dir},
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Context;
use structopt::*;

use crate::{converters::Converter, tts::TTS};

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

    println!("Converting {} files.", all_files.len());
    let pb = Arc::new(Mutex::new(ProgressBar::new((all_files.len() + 1) as u64)));
    pb.lock().expect("TODO").inc(1);

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
        let pb = pb.clone();

        let pathstr = path.to_str().unwrap();

        let tts = TTS::new().expect("TODO");
        let aiff = tts.say(&pathstr, &outdir).await.expect("TODO").path;

        let metadata = HashMap::from([
            ("title", &chapter_name),
            ("album", &audiobook_name),
            ("author", &audiobook_author),
        ]);
        let mp3out = format!("{}/{}.mp3", &outdir, &chapter_name);
        let _ = Converter::convert_aiff_to_mp3(&aiff, &mp3out, &metadata).expect("TODO");
        pb.lock().expect("TODO").inc(1);
    }
    Ok(())
}
