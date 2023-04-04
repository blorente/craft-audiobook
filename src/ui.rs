use anyhow::anyhow;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::command::AsyncCommand;

#[derive(Clone)]
pub struct ConsoleUi {
    inner: Arc<Mutex<ConsoleUiInner>>,
}

impl ConsoleUi {
    pub fn new() -> anyhow::Result<Self> {
        let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {wide_msg}").unwrap();

        let bar = MultiProgress::new();
        Ok(Self {
            inner: Arc::new(Mutex::new(ConsoleUiInner {
                bar,
                bars: HashMap::new(),
                style: spinner_style,
            })),
        })
    }

    fn with_inner<F, Ret>(&self, mut cb: F) -> anyhow::Result<Ret>
    where
        F: FnMut(&mut ConsoleUiInner) -> anyhow::Result<Ret>,
    {
        let mut guard = self
            .inner
            .lock()
            .map_err(|err| anyhow!("Error locking the progress bar: {:?}", err))?;
        cb(&mut *guard)
    }
    pub fn start_command(&self, message: String) -> anyhow::Result<UiCommandId> {
        self.with_inner(|inner| inner.start_command(&message))
    }

    pub fn finish_command(&self, command: UiCommandId) -> anyhow::Result<()> {
        self.with_inner(|inner| inner.finish_command(command))
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct UiCommandId(usize);

pub struct ConsoleUiInner {
    bar: MultiProgress,
    style: ProgressStyle,
    bars: HashMap<UiCommandId, ProgressBar>,
}
impl ConsoleUiInner {
    pub fn start_command(&mut self, message: &String) -> anyhow::Result<UiCommandId> {
        let id = UiCommandId(self.bars.len());
        let new_bar = self.bar.add(ProgressBar::new(1));
        new_bar.set_style(self.style.clone());
        new_bar.set_prefix(format!("[{}/?]", id.0));
        // TODO remove this clone somehow
        new_bar.set_message(message.clone());
        new_bar.inc(1);
        self.bars.insert(id, new_bar);
        Ok(id)
    }

    pub fn finish_command(&mut self, command: UiCommandId) -> anyhow::Result<()> {
        let mut bar = self
            .bars
            .remove(&command)
            .ok_or(anyhow!("UiCommandId {:?} not found", command))?;
        bar.set_prefix("[OK]");
        bar.finish();
        Ok(())
    }
}
