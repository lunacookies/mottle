use tincture::{ColorSpace, Hex, LinearRgb, Oklab, Oklch, Srgb};

#[derive(Clone, Copy)]
pub struct Style {
    color: Option<Color>,
    font_style: Option<FontStyle>,
}

impl<C: Into<Color>> From<C> for Style {
    fn from(color: C) -> Self {
        Self {
            color: Some(color.into()),
            font_style: None,
        }
    }
}

impl From<FontStyle> for Style {
    fn from(font_style: FontStyle) -> Self {
        Self {
            color: None,
            font_style: Some(font_style),
        }
    }
}

impl<C: Into<Color>> From<(C, FontStyle)> for Style {
    fn from((color, font_style): (C, FontStyle)) -> Self {
        Self {
            color: Some(color.into()),
            font_style: Some(font_style),
        }
    }
}

impl Style {
    pub fn as_json_value(&self, in_textmate_rule: bool) -> json::Value {
        let mut map = json::Map::new();

        if let Some(ref color) = self.color {
            map.insert("foreground".to_string(), (*color).into());
        }

        if self.font_style != Some(FontStyle::Inherit) {
            if in_textmate_rule {
                let font_style = self.font_style.map_or_else(
                    || json::Value::String(String::new()),
                    |font_style| font_style.into(),
                );

                map.insert("fontStyle".to_string(), font_style);
            } else {
                let key_value_pairs = match self.font_style {
                    Some(FontStyle::Bold) => vec![("bold", json::Value::Bool(true))],
                    Some(FontStyle::Italic) => vec![("italic", json::Value::Bool(true))],
                    Some(FontStyle::BoldItalic) => vec![
                        ("bold", json::Value::Bool(true)),
                        ("italic", json::Value::Bool(true)),
                    ],
                    Some(FontStyle::Underline) => vec![("underline", json::Value::Bool(true))],
                    Some(FontStyle::Inherit) => unreachable!(),
                    None => vec![("fontStyle", json::Value::String(String::new()))],
                };

                for (key, value) in key_value_pairs {
                    map.insert(key.to_string(), value);
                }
            }
        }

        json::Value::Object(map)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum FontStyle {
    Bold,
    Italic,
    BoldItalic,
    Underline,
    Inherit,
}

impl From<FontStyle> for json::Value {
    fn from(font_style: FontStyle) -> Self {
        match font_style {
            FontStyle::Bold => Self::String("bold".to_string()),
            FontStyle::Italic => Self::String("italic".to_string()),
            FontStyle::BoldItalic => Self::String("bold italic".to_string()),
            FontStyle::Underline => Self::String("underline".to_string()),
            FontStyle::Inherit => Self::String(String::new()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub hex: u32,
    pub alpha: Option<u8>,
}

impl From<Oklab> for Color {
    fn from(oklab: Oklab) -> Self {
        Self {
            hex: oklab_to_hex(oklab),
            alpha: None,
        }
    }
}

impl From<(Oklab, u8)> for Color {
    fn from((oklab, alpha): (Oklab, u8)) -> Self {
        Self {
            hex: oklab_to_hex(oklab),
            alpha: Some(alpha),
        }
    }
}

impl From<Oklch> for Color {
    fn from(oklch: Oklch) -> Self {
        Self {
            hex: oklch_to_hex(oklch),
            alpha: None,
        }
    }
}

impl From<(Oklch, u8)> for Color {
    fn from((oklch, alpha): (Oklch, u8)) -> Self {
        Self {
            hex: oklch_to_hex(oklch),
            alpha: Some(alpha),
        }
    }
}

impl From<Color> for json::Value {
    fn from(color: Color) -> Self {
        let hex = if let Some(alpha) = color.alpha {
            format!("#{:06X}{:02X}", color.hex, alpha)
        } else {
            format!("#{:06X}", color.hex)
        };

        Self::String(hex)
    }
}

fn oklab_to_hex(oklab: Oklab) -> u32 {
    let linear_rgb: LinearRgb = tincture::convert(oklab);
    let srgb = Srgb::from(linear_rgb);
    assert!(srgb.in_bounds());

    srgb.hex()
}

fn oklch_to_hex(oklch: Oklch) -> u32 {
    let oklab = Oklab::from(oklch);
    oklab_to_hex(oklab)
}
