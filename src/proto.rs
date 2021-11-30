pub mod semantic;
pub mod textmate;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub name: String,
    #[serde(rename = "tokenColors")]
    pub textmate_rules: Vec<textmate::Rule>,
    #[serde(flatten)]
    pub semantic_highlighting: semantic::Highlighting,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer
            .collect_str(&format_args!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a))
    }
}
