#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod button;
mod images;

use app::IVApp;
use clap::Parser;
use eframe::{
    epaint::{vec2, Vec2},
    NativeOptions,
};
use log::LevelFilter;
use std::path::PathBuf;

use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Debug, Clone, Parser)]
pub struct CmdLine {
    files: Option<Vec<PathBuf>>,

    #[clap(short, long, default_value_t = false)]
    recursive: bool,
    #[clap(short = 'd', long = "verbose", default_value_t = false)]
    verbose: bool,
}

impl CmdLine {
    fn get_files(self) -> Vec<PathBuf> {
        self.files
            .unwrap_or_else(|| get_lists_curr_dir(self.recursive))
    }
}

const INIT_SIZE_WINDOW: Vec2 = vec2(720.0, 480.0);

fn main() -> anyhow::Result<()> {
    let cmd = CmdLine::parse();
    let level = if cmd.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };
    env_logger::builder().filter_level(level).init();

    let no = NativeOptions {
        drag_and_drop_support: true,
        centered: true,
        initial_window_size: Some(INIT_SIZE_WINDOW),
        min_window_size: Some(INIT_SIZE_WINDOW),
        ..Default::default()
    };
    eframe::run_native("IVRZ", no, Box::new(|cc| IVApp::new(cc, cmd.get_files())))
        .map_err(|err| anyhow::anyhow!("Failed to run naitve window - {err}"))
}

pub fn get_lists_curr_dir(recursive: bool) -> Vec<PathBuf> {
    let curr_dir = match std::env::current_dir() {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Failed to get current directory - {err}");
            return vec![];
        }
    };
    read_dir_rec(curr_dir, recursive)
}

fn read_dir_rec(dir: PathBuf, recursive: bool) -> Vec<PathBuf> {
    let readdir = match std::fs::read_dir(&dir) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Failed to read directory {} - {err}", dir.display());
            return vec![];
        }
    };

    readdir
        .par_bridge()
        .filter_map(|result_direntry| match result_direntry {
            Ok(entry) => {
                if entry.file_type().is_ok_and(|x| x.is_dir() && recursive) {
                    Some(read_dir_rec(entry.path(), recursive))
                } else {
                    Some(vec![entry.path()])
                }
            }
            Err(err) => {
                log::error!("Failed to readdir on path - (Reason: {err})");
                None
            }
        })
        .flatten()
        .collect::<Vec<_>>()
}
