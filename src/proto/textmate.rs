use super::Color;
use serde::ser::SerializeStruct;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub scope: Vec<String>,
    pub settings: RuleSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleSettings {
    pub foreground: Color,
    pub font_style: FontStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontStyle {
    Inherit,
    Set { bold: bool, italic: bool, underline: bool },
}

impl Serialize for RuleSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.font_style {
            FontStyle::Inherit => {
                let mut strukt = serializer.serialize_struct("Settings", 1)?;
                strukt.serialize_field("foreground", &self.foreground)?;
                strukt.end()
            }

            FontStyle::Set { bold, italic, underline } => {
                let mut strukt = serializer.serialize_struct("Settings", 2)?;

                strukt.serialize_field("foreground", &self.foreground)?;

                let mut s = String::new();

                if italic {
                    if !s.is_empty() {
                        s.push(' ');
                    }
                    s.push_str("italic");
                }

                if bold {
                    if !s.is_empty() {
                        s.push(' ');
                    }
                    s.push_str("bold");
                }

                if underline {
                    if !s.is_empty() {
                        s.push(' ');
                    }
                    s.push_str("underline");
                }

                strukt.serialize_field("fontStyle", &s)?;

                strukt.end()
            }
        }
    }
}
