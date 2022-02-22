pub mod semantic;
pub mod textmate;

use indexmap::IndexMap;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub name: String,
    #[serde(rename = "tokenColors")]
    pub textmate_rules: Vec<textmate::Rule>,
    #[serde(flatten)]
    pub semantic_highlighting: semantic::Highlighting,
    #[serde(rename = "colors")]
    pub workbench_rules: IndexMap<Cow<'static, str>, Color>,
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
        if self.a == 0xFF {
            serializer.collect_str(&format_args!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b))
        } else {
            serializer.collect_str(&format_args!(
                "#{:02X}{:02X}{:02X}{:02X}",
                self.r, self.g, self.b, self.a
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};
    use indexmap::IndexMap;

    fn check(theme: Theme, expect: Expect) {
        expect.assert_eq(&crate::serialize_theme(&theme));
    }

    #[test]
    fn completely_empty() {
        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: semantic::Highlighting::Off,
                workbench_rules: IndexMap::new(),
            },
            expect![[r#"
                // Do not edit directly; this file is generated.
                {
                    "name": "My cool theme",
                    "tokenColors": [],
                    "semanticHighlighting": false,
                    "colors": {}
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
                workbench_rules: IndexMap::new(),
            },
            expect![[r#"
                // Do not edit directly; this file is generated.
                {
                    "name": "My cool theme",
                    "tokenColors": [],
                    "semanticHighlighting": true,
                    "semanticTokenColors": {},
                    "colors": {}
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
                workbench_rules: IndexMap::new(),
            },
            expect![[r##"
                // Do not edit directly; this file is generated.
                {
                    "name": "My cool theme",
                    "tokenColors": [
                        {
                            "scope": [
                                "entity.function.name"
                            ],
                            "settings": {
                                "foreground": "#9CDBDE"
                            }
                        }
                    ],
                    "semanticHighlighting": false,
                    "colors": {}
                }
            "##]],
        );
    }

    #[test]
    fn textmate_with_font_styles() {
        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![
                    textmate::Rule {
                        scope: vec!["storage".to_string()],
                        settings: textmate::RuleSettings {
                            foreground: Some(Color { r: 0, g: 0, b: 0, a: 255 }),
                            font_style: textmate::FontStyle::Set {
                                bold: true,
                                italic: true,
                                underline: false,
                            },
                        },
                    },
                    textmate::Rule {
                        scope: vec!["entity".to_string()],
                        settings: textmate::RuleSettings {
                            foreground: None,
                            font_style: textmate::FontStyle::Set {
                                bold: false,
                                italic: true,
                                underline: false,
                            },
                        },
                    },
                ],
                semantic_highlighting: semantic::Highlighting::Off,
                workbench_rules: IndexMap::new(),
            },
            expect![[r##"
                // Do not edit directly; this file is generated.
                {
                    "name": "My cool theme",
                    "tokenColors": [
                        {
                            "scope": [
                                "storage"
                            ],
                            "settings": {
                                "foreground": "#000000",
                                "fontStyle": "italic bold"
                            }
                        },
                        {
                            "scope": [
                                "entity"
                            ],
                            "settings": {
                                "fontStyle": "italic"
                            }
                        }
                    ],
                    "semanticHighlighting": false,
                    "colors": {}
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
                modifiers: vec![semantic::Identifier::new("unsafe").unwrap()],
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
                kind: semantic::TokenKind::Wildcard,
                modifiers: vec![semantic::Identifier::new("mutable").unwrap()],
                language: None,
            },
            semantic::Style {
                foreground: None,
                font_style: semantic::FontStyle {
                    bold: semantic::FontStyleSetting::Inherit,
                    italic: semantic::FontStyleSetting::Inherit,
                    underline: semantic::FontStyleSetting::True,
                },
            },
        );

        rules.insert(
            semantic::Selector {
                kind: semantic::TokenKind::Specific(semantic::Identifier::new("variable").unwrap()),
                modifiers: Vec::new(),
                language: Some(semantic::Identifier::new("rust").unwrap()),
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
                kind: semantic::TokenKind::Specific(semantic::Identifier::new("function").unwrap()),
                modifiers: vec![
                    semantic::Identifier::new("declaration").unwrap(),
                    semantic::Identifier::new("public").unwrap(),
                ],
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
                workbench_rules: IndexMap::new(),
            },
            expect![[r##"
                // Do not edit directly; this file is generated.
                {
                    "name": "My cool theme",
                    "tokenColors": [],
                    "semanticHighlighting": true,
                    "semanticTokenColors": {
                        "*.unsafe": {
                            "foreground": "#FF0000",
                            "bold": true
                        },
                        "*.mutable": {
                            "underline": true
                        },
                        "variable:rust": {
                            "foreground": "#E0E0C9"
                        },
                        "function.declaration.public": {
                            "foreground": "#9CDBDE"
                        }
                    },
                    "colors": {}
                }
            "##]],
        );
    }

    #[test]
    fn workbench_rules() {
        let mut workbench_rules = IndexMap::new();
        workbench_rules
            .insert(Cow::Borrowed("editor.foreground"), Color { r: 255, g: 0, b: 0, a: 255 });

        check(
            Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: semantic::Highlighting::Off,
                workbench_rules,
            },
            expect![[r##"
                // Do not edit directly; this file is generated.
                {
                    "name": "My cool theme",
                    "tokenColors": [],
                    "semanticHighlighting": false,
                    "colors": {
                        "editor.foreground": "#FF0000"
                    }
                }
            "##]],
        );
    }
}
