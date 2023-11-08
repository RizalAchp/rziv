use paste::paste;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonKind {
    Ok,
    Cancel,
    Apply,
    Reset,
    Open,
    Save,
    SaveAs,
    Close,
    Delete,
    Play,
    Pause,
    Stop,
    Record,
    Next,
    Previous,
    FullScreen,
    Random,
    Edit,
    Favorite,
    Unfavorite,
    Mute,
    Unmute,
    Lock,
    Unlock,
    Refresh,
    New,
    Copy,
    Paste,
    Cut,
    No,
}

impl std::fmt::Display for ButtonKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Ok => "\u{2714}",
            Self::Cancel => "\u{1F6AB}",
            Self::Apply => "\u{2714}",
            Self::Reset => "\u{1F504}",
            Self::Open => "\u{1F5C1}",
            Self::Save => "\u{1F4BE}",
            Self::SaveAs => "\u{1F4BE}",
            Self::Close => "\u{1F5D9}",
            Self::Delete => "\u{1F5D1}",
            Self::Play => "\u{25B6}",
            Self::Pause => "\u{23F8}",
            Self::Stop => "\u{23F9}",
            Self::Record => "\u{23FA}",
            Self::Next => "\u{23ED}",
            Self::Previous => "\u{23EE}",
            Self::FullScreen => "\u{26F6}",
            Self::Random => "\u{1F3B2}",
            Self::Edit => "\u{270F}",
            Self::Favorite => "\u{2605}",
            Self::Unfavorite => "\u{2606}",
            Self::Mute => "\u{1F507}",
            Self::Unmute => "\u{1F50A}",
            Self::Lock => "\u{1F512}",
            Self::Unlock => "\u{1F513}",
            Self::Refresh => "\u{1F503}",
            Self::New => "\u{1F5CB}",
            Self::Copy => "\u{1F5D0}",
            Self::Paste => "\u{1F4CB}",
            Self::Cut => "\u{2702}",
            Self::No => "\u{2718}",
        };
        f.write_str(s)
    }
}

impl ButtonKind {
    #[inline(always)]
    pub fn name_button_popup(self, desc: &'_ str) -> String {
        format!("clickk '{self}' to {desc}")
    }
}

impl Default for ButtonKind {
    fn default() -> Self {
        Self::Ok
    }
}

macro_rules!  standart_button {
    ($traits:ident {$( $name: ident),*}) => {
        pub trait $traits {
            fn button_ext(&mut self, button_kind: ButtonKind) -> eframe::egui::Button;
            fn small_button_ext(&mut self, button_kind: ButtonKind) -> eframe::egui::Button;
        paste!($(
            #[allow(unused)]
            #[inline(always)]
            fn [<$name:lower _button>](&mut self) -> eframe::egui::Button {
                self.button_ext(ButtonKind::$name)
            }
            #[allow(unused)]
            #[inline(always)]
            fn [<small_ $name:lower _button>](&mut self) -> eframe::egui::Button {
                self.small_button_ext(ButtonKind::$name)
            }
        )*);
        }
    };
}

standart_button!(ButtonExt {
    Ok,
    Cancel,
    Apply,
    Reset,
    Open,
    Save,
    SaveAs,
    Close,
    Delete,
    Play,
    Pause,
    Stop,
    Record,
    Next,
    Previous,
    FullScreen,
    Random,
    Edit,
    Favorite,
    Unfavorite,
    Mute,
    Unmute,
    Lock,
    Unlock,
    Refresh,
    New,
    Copy,
    Paste,
    Cut,
    No
});

impl ButtonExt for eframe::egui::Ui {
    #[allow(unused)]
    #[inline(always)]
    fn small_button_ext(&mut self, button_kind: ButtonKind) -> eframe::egui::Button {
        eframe::egui::Button::new(button_kind.to_string())
    }
    #[allow(unused)]
    #[inline(always)]
    fn button_ext(&mut self, button_kind: ButtonKind) -> eframe::egui::Button {
        eframe::egui::Button::new(button_kind.to_string())
    }
}
