use super::Color;
use indexmap::IndexMap;
use serde::ser::SerializeStruct;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Highlighting {
    Off,
    On { rules: IndexMap<Selector, Style> },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selector {
    pub kind: TokenKind,
    pub modifiers: Vec<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    pub font_styles: FontStyle,
    pub foreground: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    Wildcard,
    Specific(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontStyle {
    pub bold: FontStyleSetting,
    pub italic: FontStyleSetting,
    pub underline: FontStyleSetting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontStyleSetting {
    True,
    False,
    Inherit,
}

impl Serialize for Highlighting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Off => {
                let mut strukt = serializer.serialize_struct("SemanticHighlighting", 1)?;
                strukt.serialize_field("semanticHighlighting", &false)?;
                strukt.end()
            }

            Self::On { rules } => {
                let mut strukt = serializer.serialize_struct("SemanticHighlighting", 2)?;
                strukt.serialize_field("semanticHighlighting", &true)?;
                strukt.serialize_field("semanticTokenColors", &rules)?;
                strukt.end()
            }
        }
    }
}

impl Serialize for Selector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = String::new();

        match &self.kind {
            TokenKind::Wildcard => s.push('*'),
            TokenKind::Specific(kind) => s.push_str(kind),
        }

        for modifier in &self.modifiers {
            s.push('.');
            s.push_str(modifier);
        }

        if let Some(language) = &self.language {
            s.push(':');
            s.push_str(language);
        }

        serializer.serialize_str(&s)
    }
}

impl Serialize for Style {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut strukt = serializer.serialize_struct("Style", 1)?;
        strukt.serialize_field("foreground", &self.foreground)?;

        match self.font_styles.bold {
            FontStyleSetting::True => strukt.serialize_field("bold", &true)?,
            FontStyleSetting::False => strukt.serialize_field("bold", &false)?,
            FontStyleSetting::Inherit => {}
        }

        match self.font_styles.italic {
            FontStyleSetting::True => strukt.serialize_field("italic", &true)?,
            FontStyleSetting::False => strukt.serialize_field("italic", &false)?,
            FontStyleSetting::Inherit => {}
        }

        match self.font_styles.underline {
            FontStyleSetting::True => strukt.serialize_field("underline", &true)?,
            FontStyleSetting::False => strukt.serialize_field("underline", &false)?,
            FontStyleSetting::Inherit => {}
        }

        strukt.end()
    }
}
