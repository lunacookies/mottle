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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};
    use indexmap::IndexMap;

    fn check(theme: Theme, expect: Expect) {
        expect.assert_eq(&format!("{}\n", crate::format_theme(&theme)));
    }

    #[test]
    fn completely_empty() {
        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: semantic::Highlighting::Off,
            },
            expect![[r#"
                {
                    "name": "My cool theme",
                    "tokenColors": [],
                    "semanticHighlighting": false
                }
            "#]],
        );
    }

    #[test]
    fn empty_semantic() {
        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: semantic::Highlighting::On { rules: IndexMap::new() },
            },
            expect![[r#"
                {
                    "name": "My cool theme",
                    "tokenColors": [],
                    "semanticHighlighting": true,
                    "semanticTokenColors": {}
                }
            "#]],
        );
    }

    #[test]
    fn basic_textmate() {
        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![textmate::Rule {
                    scope: vec!["entity.function.name".to_string()],
                    settings: textmate::RuleSettings {
                        foreground: Some(Color { r: 156, g: 219, b: 222, a: 255 }),
                        font_style: textmate::FontStyle::Inherit,
                    },
                }],
                semantic_highlighting: semantic::Highlighting::Off,
            },
            expect![[r##"
                {
                    "name": "My cool theme",
                    "tokenColors": [
                        {
                            "scope": [
                                "entity.function.name"
                            ],
                            "settings": {
                                "foreground": "#9CDBDEFF"
                            }
                        }
                    ],
                    "semanticHighlighting": false
                }
            "##]],
        );
    }

    #[test]
    fn textmate_with_font_styles() {
        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![textmate::Rule {
                    scope: vec!["storage".to_string()],
                    settings: textmate::RuleSettings {
                        foreground: Some(Color { r: 0, g: 0, b: 0, a: 255 }),
                        font_style: textmate::FontStyle::Set {
                            bold: true,
                            italic: true,
                            underline: false,
                        },
                    },
                }],
                semantic_highlighting: semantic::Highlighting::Off,
            },
            expect![[r##"
                {
                    "name": "My cool theme",
                    "tokenColors": [
                        {
                            "scope": [
                                "storage"
                            ],
                            "settings": {
                                "foreground": "#000000FF",
                                "fontStyle": "italic bold"
                            }
                        }
                    ],
                    "semanticHighlighting": false
                }
            "##]],
        );
    }

    #[test]
    fn semantic() {
        let mut rules = IndexMap::new();

        rules.insert(
            semantic::Selector {
                kind: semantic::TokenKind::Wildcard,
                modifiers: vec!["unsafe".to_string()],
                language: None,
            },
            semantic::Style {
                foreground: Some(Color { r: 255, g: 0, b: 0, a: 255 }),
                font_style: semantic::FontStyle {
                    bold: semantic::FontStyleSetting::True,
                    italic: semantic::FontStyleSetting::Inherit,
                    underline: semantic::FontStyleSetting::Inherit,
                },
            },
        );

        rules.insert(
            semantic::Selector {
                kind: semantic::TokenKind::Specific("variable".to_string()),
                modifiers: Vec::new(),
                language: Some("rust".to_string()),
            },
            semantic::Style {
                foreground: Some(Color { r: 224, g: 224, b: 201, a: 255 }),
                font_style: semantic::FontStyle {
                    bold: semantic::FontStyleSetting::Inherit,
                    italic: semantic::FontStyleSetting::Inherit,
                    underline: semantic::FontStyleSetting::Inherit,
                },
            },
        );

        rules.insert(
            semantic::Selector {
                kind: semantic::TokenKind::Specific("function".to_string()),
                modifiers: vec!["declaration".to_string(), "public".to_string()],
                language: None,
            },
            semantic::Style {
                foreground: Some(Color { r: 156, g: 219, b: 222, a: 255 }),
                font_style: semantic::FontStyle {
                    bold: semantic::FontStyleSetting::Inherit,
                    italic: semantic::FontStyleSetting::Inherit,
                    underline: semantic::FontStyleSetting::Inherit,
                },
            },
        );

        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: semantic::Highlighting::On { rules },
            },
            expect![[r##"
                {
                    "name": "My cool theme",
                    "tokenColors": [],
                    "semanticHighlighting": true,
                    "semanticTokenColors": {
                        "*.unsafe": {
                            "foreground": "#FF0000FF",
                            "bold": true
                        },
                        "variable:rust": {
                            "foreground": "#E0E0C9FF"
                        },
                        "function.declaration.public": {
                            "foreground": "#9CDBDEFF"
                        }
                    }
                }
            "##]],
        );
    }
}
