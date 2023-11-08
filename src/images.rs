use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use arboard::{Clipboard, ImageData};
use eframe::{
    egui::{
        self,
        load::{Bytes, TexturePoll},
        paint_texture_at, DroppedFile, Image, ImageSource, Key, Modifiers, Response, Sense,
        Spinner, TextStyle, TextureOptions, Ui,
    },
    emath::Align2,
    epaint::{Color32, ColorImage, Pos2, Rect, Rounding, Stroke, Vec2},
    Frame,
};
use image::ImageFormat;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

#[derive(Clone)]
struct Img<'img> {
    fmt: ImageFormat,
    source: ImageSource<'img>,
}
impl std::fmt::Debug for Img<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Img")
            .field("fmt", &self.fmt)
            .field(
                "source",
                match &self.source {
                    ImageSource::Uri(uri) => uri,
                    ImageSource::Texture(i) => i,
                    ImageSource::Bytes { uri, .. } => uri,
                },
            )
            .finish()
    }
}

impl<'i> Img<'i> {
    #[inline]
    fn from_bytes(fmt: ImageFormat, uri: Option<String>, bytes: Vec<u8>) -> Self {
        Self {
            fmt,
            source: ImageSource::Bytes {
                uri: Cow::Owned(
                    uri.unwrap_or(format!("bytes://bytes_image.{}", fmt.extensions_str()[0])),
                ),
                bytes: Bytes::Shared(bytes.into()),
            },
        }
    }

    #[allow(unused)]
    fn from_uri(fmt: ImageFormat, uri: String) -> Self {
        Self {
            fmt,
            source: ImageSource::Uri(uri.into()),
        }
    }

    fn from_path(fmt: ImageFormat, path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();
        match std::fs::read(path) {
            Ok(bytes) => Some(Self::from_bytes(
                fmt,
                Some(format!("bytes:/{}", path.display())),
                bytes,
            )),
            Err(err) => {
                log::error!("ERROR: Failed to read contents of image - {err}");
                None
            }
        }
    }

    #[inline]
    fn from_paths<I>(paths: I) -> Vec<Self>
    where
        I: IntoParallelIterator<Item = PathBuf>,
    {
        paths
            .into_par_iter()
            .filter_map(filter_map_images_file)
            .filter_map(|(fmt, path)| Self::from_path(fmt, path))
            .collect()
    }
}

#[derive(Debug)]
pub struct IVImages<'img> {
    images_sources: Vec<Img<'img>>,
    texture_options: TextureOptions,
    rect: Rect,
    size: Option<Vec2>,
    zoom: Vec2,
    drag: Vec2,
    showed_idx: usize,
}

impl<'img> IVImages<'img> {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        let images_sources = Img::from_paths(paths);
        Self {
            images_sources,
            texture_options: TextureOptions::NEAREST,
            size: None,
            rect: Rect::ZERO,
            zoom: Vec2::splat(1f32),
            drag: Vec2::ZERO,
            showed_idx: 0,
        }
    }

    pub fn extend_from_dropfile<I>(&mut self, paths: I)
    where
        I: IntoParallelIterator<Item = DroppedFile>,
    {
        self.images_sources.extend(
            paths
                .into_par_iter()
                .filter_map(|x| {
                    x.path
                        .and_then(filter_map_images_file)
                        .and_then(|(fmt, path)| Img::from_path(fmt, path))
                })
                .collect::<Vec<_>>(),
        )
    }
    pub fn extend_from_image_data(&mut self, img_data: ImageData<'_>) {
        use std::io::Cursor;
        let w = img_data.width as u32;
        let h = img_data.height as u32;
        let mut buffer = Cursor::new(Vec::with_capacity((w * h) as usize));
        if let Err(err) = image::write_buffer_with_format(
            &mut buffer,
            &img_data.bytes,
            w,
            h,
            image::ColorType::Rgba8,
            image::ImageOutputFormat::Png,
        ) {
            log::error!("Failed to convert image from raw rgba to png - (Reason: {err})");
            return;
        }
        self.images_sources
            .push(Img::from_bytes(ImageFormat::Png, None, buffer.into_inner()))
    }

    #[allow(unused)]
    pub fn set_size(&mut self, size: Vec2) {
        self.size = Some(size);
    }

    #[inline]
    pub fn set_next(&mut self) {
        self.showed_idx = self
            .showed_idx
            .saturating_add(1)
            .min(self.images_sources.len() - 1);
        log::debug!("setting next index on: {}", self.showed_idx);
    }
    #[inline]
    pub fn set_prev(&mut self) {
        self.showed_idx = self.showed_idx.saturating_sub(1);
        log::debug!("setting prev index on: {}", self.showed_idx);
    }

    pub fn copy_to_clipboard(&self, frame: &Frame, clipboard: &mut Clipboard, ss: ColorImage) {
        let ss = ss.region(&self.rect, frame.info().native_pixels_per_point);
        if let Err(err) = clipboard.set_image(ImageData {
            width: ss.width(),
            height: ss.height(),
            bytes: ss.as_raw().into(),
        }) {
            log::error!("Failed to copy image to clipboard: (Reason: {err})")
        }
    }

    fn button(&mut self, ui: &mut Ui, rect: Rect, pos: Pos2, align: Align2, text: &str) -> bool {
        let mut clicked = false;
        let btn_rect = ui.painter().text(
            pos,
            align,
            text,
            TextStyle::Heading.resolve(ui.style()),
            ui.visuals().text_color(),
        );
        if ui.rect_contains_pointer(btn_rect) {
            ui.painter().rect(
                rect,
                Rounding::ZERO,
                Color32::from_black_alpha(100),
                Stroke::NONE,
            );
            if ui.input(|x| x.pointer.primary_pressed()) {
                clicked = true;
            }
        }
        clicked
    }
}

impl IVImages<'_> {
    pub fn draw(&mut self, ui: &mut Ui) -> Response {
        let res = ui.allocate_rect(ui.min_rect(), Sense::click_and_drag());

        ui.input_mut(|i| {
            let reset = i.key_released(egui::Key::Num0);
            let zoom_delta = i.zoom_delta();
            if i.consume_key(Modifiers::CTRL, Key::J) {
                self.set_prev();
            } else if i.consume_key(Modifiers::CTRL, Key::K) {
                self.set_next();
            } else if zoom_delta > 1.0 && !reset {
                self.zoom += Vec2::splat(zoom_delta * 0.1);
            } else if zoom_delta < 1.0 && !reset {
                self.zoom -= Vec2::splat(zoom_delta * 0.1);
            }

            if reset {
                self.zoom = Vec2::splat(1.0);
                self.drag = Vec2::ZERO;
            }
        });

        let Some(Img { fmt: _, source }) = self.images_sources.get(self.showed_idx) else {
            return res;
        };
        let image = Image::new(source.clone()).texture_options(self.texture_options);

        let size = self.size.unwrap_or(res.rect.size()) * self.zoom;
        let tlr = image.load_for_size(ui.ctx(), size);
        let ui_size = image.calc_size(size, tlr.as_ref().ok().and_then(|t| t.size()));
        // drag mouse capability
        self.rect = Rect::from_center_size(res.rect.center(), ui_size);
        if res.dragged() {
            self.drag += res.drag_delta();
        }
        self.rect = self.rect.translate(self.drag);
        match tlr {
            Ok(TexturePoll::Ready { texture }) => {
                paint_texture_at(ui.painter(), self.rect, image.image_options(), &texture);
            }
            Ok(TexturePoll::Pending { .. }) => {
                Spinner::new().paint_at(ui, self.rect);
            }
            Err(_) => {
                let font_id = TextStyle::Body.resolve(ui.style());
                ui.painter().text(
                    self.rect.center(),
                    Align2::CENTER_CENTER,
                    "⚠",
                    font_id,
                    ui.visuals().error_fg_color,
                );
            }
        }
        if res.hovered() {
            if self.button(
                ui,
                res.rect,
                res.rect.left_center(),
                Align2::LEFT_CENTER,
                "\u{23EE}",
            ) {
                self.set_prev();
            }
            if self.button(
                ui,
                res.rect,
                res.rect.right_center(),
                Align2::RIGHT_CENTER,
                "\u{23ED}",
            ) {
                self.set_next();
            }
        }
        res
    }
}

#[inline]
fn filter_map_images_file(path: PathBuf) -> Option<(ImageFormat, PathBuf)> {
    let ext = path.extension()?;
    let fmt = image::ImageFormat::from_extension(ext)?;
    Some((fmt, path))
}
