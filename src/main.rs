#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod button;
mod images;
mod logger;

use app::IVApp;
use clap::Parser;
use eframe::{epaint, NativeOptions};
use std::path::PathBuf;

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
        let Some(mut files) = self.files else {
            return get_lists_curr_dir(self.recursive);
        };
        if files.len() == 1 {
            let file = files[0].clone();
            if file.is_dir() {
                return read_dir_rec(file, self.recursive);
            } else if let Some(par) = file.parent().map(|p| p.to_path_buf()) {
                files.extend(read_dir_rec(par, self.recursive));
                files.dedup();
            }
            files
        } else {
            files
        }
    }
}

const INIT_SIZE_WINDOW: epaint::Vec2 = epaint::vec2(720.0, 480.0);

fn main() -> anyhow::Result<()> {
    let cmd = CmdLine::parse();
    logger::init_logger(cmd.verbose);

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
