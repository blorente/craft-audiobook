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
        let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let bar = MultiProgress::new();
        Ok(Self {
            inner: Arc::new(Mutex::new(ConsoleUiInner {
                bar,
                bars: HashMap::new(),
                style: spinner_style,
            })),
        })
    }

    fn with_inner<F>(&self, mut cb: F) -> anyhow::Result<()>
    where
        F: FnMut(&mut ConsoleUiInner) -> anyhow::Result<()>,
    {
        let mut guard = self
            .inner
            .lock()
            .map_err(|err| anyhow!("Error locking the progress bar: {:?}", err))?;
        cb(&mut *guard)
    }
    pub fn start_command(&self, command: &AsyncCommand) -> anyhow::Result<()> {
        self.with_inner(|inner| inner.start_command(command))
    }

    pub fn finish_command(&self, command: &AsyncCommand) -> anyhow::Result<()> {
        self.with_inner(|inner| inner.finish_command(command))
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct UiCommandId(usize);

pub struct ConsoleUiInner {
    bar: MultiProgress,
    style: ProgressStyle,
    bars: HashMap<UiCommandId, ProgressBar>,
}
impl ConsoleUiInner {
    pub fn start_command(&mut self, command: &AsyncCommand) -> anyhow::Result<UiCommandId> {
        let new_bar = self.bar.add(ProgressBar::new());
        let id = UiCommandId(self.bars.len());
        self.bars.insert(id, new_bar);
        Ok(id)
    }

    pub fn finish_command(&mut self, command: UiCommandId) -> anyhow::Result<()> {
        Ok(())
    }
}
