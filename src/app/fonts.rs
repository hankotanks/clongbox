use std::{fmt, io, borrow};

use include_dir::Dir;
use once_cell::sync::{Lazy, OnceCell};

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Font {
    GentiumPlus,
    Andika,
    CharisSIL,
    DoulosSIL,
}

impl Font {
    const fn as_filename(&self) -> &str {
        // NOTE: These filenames MUST match the contents of /assets/fonts
        match self {
            Font::GentiumPlus => "GentiumPlus.ttf",
            Font::Andika => "Andika.ttf",
            Font::CharisSIL => "CharisSIL.ttf",
            Font::DoulosSIL => "DoulosSIL.ttf",
        }
    }
}

impl fmt::Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Font::GentiumPlus => "Gentium Plus",
            Font::Andika => "Andika",
            Font::CharisSIL => "Charis SIL",
            Font::DoulosSIL => "Doulos SIL",
        })
    }
}

static FONT_DATA: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/fonts");

fn load_font_data(selection: Font) -> anyhow::Result<Vec<u8>> {
    let glob = format!("**/{}", selection.as_filename());

    let error = io::Error::new(
        io::ErrorKind::NotFound, 
        "Unable to load required fonts."
    );

    let file = glob.as_str();
    let file = FONT_DATA.find(file)?.next().unwrap();
    let file = file.as_file().ok_or(error)?;

    Ok(file.contents().to_vec())
}

static FONT_FAMILY: Lazy<egui::FontFamily> = Lazy::new(|| {
    egui::FontFamily::Name("IPA".into())
});

#[allow(dead_code)]
pub static FONT_ID: Lazy<egui::FontId> = Lazy::new(|| egui::FontId {
    size: 16.,
    family: FONT_FAMILY.to_owned()
});

static FONT_SCALING_DATA: OnceCell<rusttype::Font<'static>> = OnceCell::new();

#[allow(dead_code)]
pub fn ipa_text_width<'a, I>(text: I) -> f32
    where I: Into<borrow::Cow<'a, str>> {
    let text: borrow::Cow<'_, str> = text.into();

    let scale = rusttype::Scale::uniform(FONT_ID.size);

    let font = unsafe { FONT_SCALING_DATA.get_unchecked() };

    font
        .layout(text.as_ref(), scale, rusttype::point(0.0, 0.0))
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .last()
        .unwrap_or(0.0)
}

pub fn load_fonts(selection: Font) -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    match load_font_data(selection) {
        Ok(loaded_font_data) => {
            fonts.font_data.insert(
                format!("{}", selection),
                egui::FontData::from_owned(loaded_font_data)
            );
        
            fonts.families.insert(
                FONT_FAMILY.to_owned(), 
                vec![format!("{}", selection)]
            );
        },
        Err(..) => {
            // Fall back on default fonts if necessary
            // This is very undesirable behavior, most IPA symbols won't render
            fonts.families.insert(
                FONT_FAMILY.to_owned(),
                vec!["Hack".to_owned(), "Ubuntu-Light".to_owned()]
            );
        },
    }

    let font_name = &fonts.families[&FONT_FAMILY][0];

    let font_data = &fonts.font_data[font_name];
    let font_data = font_data.font.to_vec();

    let font_scaling_data = rusttype::Font::try_from_vec(font_data).unwrap();

    FONT_SCALING_DATA.set(font_scaling_data).unwrap();

    fonts
}