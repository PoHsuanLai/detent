//! The files the window reads and writes, and the work on them that blocks: the settings files,
//! docket's agents and places, the person's keymap and style sheets, and the installed apps'
//! desktop entries. The types are `detent-files`', named here so the window reaches the files only
//! through this door; the commands that block run on a blocking thread, so the window keeps
//! drawing.

use crate::client::Client;
use detent_files::{StyleSheet, published_classes};
use detent_model::{Candidate, OriginRule, ProgramBook};
use std::path::PathBuf;

pub use detent_files::{
    AgentsTool, ConfigHome, DataHome, ExtensionHome, FileValues, FilesError, FirstRunMark,
    KeymapFiles, StateHome, add_agent, installed_agents, load, load_agents, load_places,
    remove_agent, save, save_agent_choice, save_places,
};

/// What the person's style files hold, with the classes the installed apps publish.
#[derive(Debug, Clone, Default)]
pub struct StyleInputs {
    /// Each style file, with its owner.
    pub sheets: Vec<StyleSheet>,
    /// The classes each installed app publishes, which a style may use without a note.
    pub apps: Vec<Vec<String>>,
}

impl Client {
    /// The coding assistants on offer to download; the empty sentence when none could be asked.
    pub async fn list_agents(&self, data: &DataHome) -> Result<Vec<Candidate>, String> {
        let Some(program) = self.agents_program.clone() else {
            return Err(String::new());
        };
        let tool = AgentsTool::new(program, data);
        tokio::task::spawn_blocking(move || tool.list())
            .await
            .unwrap_or_else(|_| Err(String::new()))
    }

    /// Downloads the coding assistant `id` at `version`, calling `begun` once the program has
    /// started; the program's sentence when it fails, empty when there is none.
    pub async fn download_agent(
        &self,
        data: &DataHome,
        id: String,
        version: String,
        begun: impl FnOnce() + Send + 'static,
    ) -> Result<(), String> {
        let Some(program) = self.agents_program.clone() else {
            return Err(String::new());
        };
        let tool = AgentsTool::new(program, data);
        tokio::task::spawn_blocking(move || tool.install(&id, &version, begun))
            .await
            .unwrap_or_else(|_| Err(String::new()))
    }

    /// The installed apps' desktop entries in `dirs`, judged by `rule`; empty when they could not
    /// be read.
    pub async fn scan_programs(&self, dirs: Vec<PathBuf>, rule: OriginRule) -> ProgramBook {
        tokio::task::spawn_blocking(move || detent_files::scan_programs(&dirs, &rule))
            .await
            .unwrap_or_default()
    }

    /// The person's style files and the classes the installed apps publish, read off the window's
    /// thread.
    pub async fn read_styles(
        &self,
        home: ConfigHome,
        data: DataHome,
        system: Vec<PathBuf>,
    ) -> StyleInputs {
        tokio::task::spawn_blocking(move || StyleInputs {
            sheets: detent_files::read_style_sheets(&home),
            apps: published_classes(&data, &system),
        })
        .await
        .unwrap_or_default()
    }
}
