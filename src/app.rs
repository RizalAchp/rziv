use std::path::PathBuf;

use arboard::Clipboard;
use eframe::{
    egui::{
        panel::TopBottomSide, Button, CentralPanel, Id, Key, KeyboardShortcut, LayerId, Modifiers,
        Order, TextStyle, TopBottomPanel, Ui, WidgetText, Window,
    },
    emath::Align2,
    epaint::Color32,
};

use crate::{button::ButtonKind, images::IVImages};

trait CustomMod {
    const CTRL_SHIFT: Modifiers = Modifiers {
        alt: false,
        ctrl: true,
        shift: true,
        mac_cmd: false,
        command: false,
    };
}
impl CustomMod for Modifiers {}

#[rustfmt::skip]
const SHORTCUTS_AND_BUTTONS: &[(ButtonKind, KeyboardShortcut, &str)] = &[
    (ButtonKind::Open,      KeyboardShortcut::new(Modifiers::CTRL,       Key::O), "Open image in disk"),
    (ButtonKind::Save,      KeyboardShortcut::new(Modifiers::CTRL,       Key::S), "Save Image in same path"),
    (ButtonKind::SaveAs,    KeyboardShortcut::new(Modifiers::CTRL_SHIFT, Key::S), "Save Image in disk with opened filemanager"),
    (ButtonKind::Copy,      KeyboardShortcut::new(Modifiers::CTRL,       Key::C), "Copy image from clipboard"),
    (ButtonKind::Paste,     KeyboardShortcut::new(Modifiers::CTRL,       Key::P), "Paste Image to clipboard"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum IVAppEvent {
    Noop,
    Open,
    Save,
    SaveAs,
    Copy,
    Paste,
}
impl From<ButtonKind> for IVAppEvent {
    fn from(value: ButtonKind) -> Self {
        use IVAppEvent::*;
        match value {
            ButtonKind::Open => Open,
            ButtonKind::Save => Save,
            ButtonKind::SaveAs => SaveAs,
            ButtonKind::Copy => Copy,
            ButtonKind::Paste => Paste,
            _ => Noop,
        }
    }
}

pub struct IVApp<'a> {
    images: IVImages<'a>,
    cb_ctx: Option<Clipboard>,
    kind_event: Option<IVAppEvent>,
}

pub fn bar_button_active(ui: &mut Ui, kind: ButtonKind, sc: KeyboardShortcut, desc: &str) -> bool {
    let clicked = ui
        .add(
            Button::new(WidgetText::from(kind.name()).strong().raised())
                .shortcut_text(ui.ctx().format_shortcut(&sc)),
        )
        .on_hover_text(kind.name_button_popup(desc))
        .clicked();
    clicked || ui.input_mut(|i| i.count_and_consume_key(sc.modifiers, sc.key) > 0)
}

impl<'a> IVApp<'a> {
    pub fn new(cc: &eframe::CreationContext, imgfiles: Vec<PathBuf>) -> Box<Self> {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let cb_ctx = match Clipboard::new() {
            Ok(ok) => Some(ok),
            Err(err) => {
                log::error!("Failed to get system clipboard - (Reason: {err})");
                None
            }
        };
        log::debug!("image_files: {imgfiles:?}");
        log::debug!("integration_info: {:#?}", cc.integration_info);
        Box::new(Self {
            images: IVImages::new(imgfiles),
            cb_ctx,
            kind_event: None,
        })
    }

    fn on_paste_event(&mut self) {
        if let Some(ref mut clipboard_ctx) = self.cb_ctx {
            match clipboard_ctx.get_image() {
                Ok(img) => self.images.extend_from_image_data(img),
                Err(err) => {
                    log::error!("Failed to get paste image - (Reason: {err})");
                    match clipboard_ctx.get_text() {
                        Ok(text) => {
                            log::debug!("Got paste item: {text}");
                        }
                        Err(err) => {
                            log::error!("Failed to get paste text - (Reason: {err})")
                        }
                    }
                }
            }
            clipboard_ctx
                .clear()
                .unwrap_or_else(|err| log::error!("Failed to clear clipboard - (Reason: {err})"));
        }
    }
}

impl<'a> eframe::App for IVApp<'a> {
    fn on_close_event(&mut self) -> bool {
        true
    }

    fn warm_up_enabled(&self) -> bool {
        true
    }

    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        if let Some(ev) = self.kind_event {
            use IVAppEvent as IVE;
            match ev {
                IVE::Noop => (),
                IVE::Open => {}
                IVE::Save => {}
                IVE::SaveAs => {}
                IVE::Copy => frame.request_screenshot(),
                IVE::Paste => self.on_paste_event(),
            }
            self.kind_event = None;
        }

        CentralPanel::default().show(ctx, |ui| {
            preview_files_being_dropped(ui.ctx());
            ui.input_mut(|input| {
                if !input.raw.dropped_files.is_empty() {
                    self.images
                        .extend_from_dropfile(input.raw.dropped_files.clone());
                    input.raw.dropped_files.clear();
                }
            });
            self.images.draw(ui);
        });

        TopBottomPanel::new(TopBottomSide::Bottom, "iv_toppanel").show_animated(ctx, true, |ui| {
            eframe::egui::menu::bar(ui, |uibar| {
                uibar.heading("IV - Image Viewer");
                uibar.separator();
                for (kind, sc, desc) in SHORTCUTS_AND_BUTTONS {
                    if bar_button_active(uibar, *kind, *sc, desc) {
                        self.kind_event = Some(From::from(*kind));
                    }
                }
            });
        });

        if cfg!(debug_assertions) {
            Window::new("Settins").default_open(false).show(ctx, |ui| {
                ctx.settings_ui(ui);
                ui.separator();
                ctx.inspection_ui(ui);
            });
        }
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], frame: &eframe::Frame) {
        if let Some(ss) = frame.screenshot() {
            log::debug!(
                "Got screenshot: ( width: {}, height: {} )",
                ss.width(),
                ss.height()
            );
            if let Some(ref mut clipboard_ctx) = self.cb_ctx {
                self.images.copy_to_clipboard(frame, clipboard_ctx, ss);
            }
        }
    }
}

fn preview_files_being_dropped(ctx: &eframe::egui::Context) -> bool {
    use std::fmt::Write as _;
    let Some(text) = ctx.input(|i| {
        if !i.raw.hovered_files.is_empty() {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            Some(text)
        } else {
            None
        }
    }) else {
        return false;
    };

    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
    let screen_rect = ctx.screen_rect();
    painter.rect_filled(
        screen_rect,
        0.0,
        eframe::epaint::Color32::from_black_alpha(192),
    );
    painter.text(
        screen_rect.center(),
        Align2::CENTER_CENTER,
        text,
        TextStyle::Heading.resolve(&ctx.style()),
        Color32::WHITE,
    );
    true
}
