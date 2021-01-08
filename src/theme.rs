mod builder;
pub use builder::{Scope, ThemeBuilder};

use crate::style::{Color, Style};
use json::ser::PrettyFormatter;
use json::Serializer;
use serde::Serialize;
use std::{fs, io};

pub struct Theme {
    name: String,
    ty: Type,
    workspace_rules: Vec<WorkspaceRule>,
    semantic_rules: Vec<Rule>,
    textmate_rules: Vec<Rule>,
}

impl Theme {
    pub fn save(self) -> io::Result<()> {
        self.save_with_indent("\t")
    }

    pub fn save_with_indent(self, indent: &str) -> io::Result<()> {
        let path = format!("themes/{}-color-theme.json", self.name);
        let json: json::Value = self.into();

        let serialized_json = {
            let mut buf = "// Do not edit directly; this file is generated.\n"
                .as_bytes()
                .to_vec();

            let pretty_formatter = PrettyFormatter::with_indent(indent.as_bytes());
            let mut serializer = Serializer::with_formatter(&mut buf, pretty_formatter);

            json.serialize(&mut serializer).unwrap();
            buf.push(b'\n');

            buf
        };

        fs::write(path, serialized_json)?;

        Ok(())
    }
}

impl From<Theme> for json::Value {
    fn from(theme: Theme) -> Self {
        let mut map = json::Map::new();

        map.insert("name".to_string(), Self::String(theme.name));
        map.insert("type".to_string(), theme.ty.into());

        map.insert(
            "colors".to_string(),
            json::Value::Object(
                theme
                    .workspace_rules
                    .into_iter()
                    .map(|rule| (rule.scope_name, rule.color.into()))
                    .collect(),
            ),
        );

        map.insert("semanticHighlighting".to_string(), Self::Bool(true));

        map.insert("semanticTokenColors".to_string(), {
            json::Value::Object(
                theme
                    .semantic_rules
                    .into_iter()
                    .map(|rule| (rule.scope_name, rule.style.as_json_value(false)))
                    .collect(),
            )
        });

        map.insert(
            "tokenColors".to_string(),
            theme
                .textmate_rules
                .into_iter()
                .map(Rule::into_textmate_json_value)
                .collect(),
        );

        Self::Object(map)
    }
}

pub enum Type {
    #[allow(dead_code)]
    Light,
    Dark,
}

impl From<Type> for json::Value {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Light => Self::String("light".to_string()),
            Type::Dark => Self::String("dark".to_string()),
        }
    }
}

struct Rule {
    scope_name: String,
    style: Style,
}

impl Rule {
    fn into_textmate_json_value(self) -> json::Value {
        let mut map = json::Map::new();

        map.insert("scope".to_string(), json::Value::String(self.scope_name));
        map.insert("settings".to_string(), self.style.as_json_value(true));

        json::Value::Object(map)
    }
}

struct WorkspaceRule {
    scope_name: String,
    color: Color,
}
