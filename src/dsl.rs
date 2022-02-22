use crate::proto;
use indexmap::IndexMap;
use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct ThemeBuilder {
    pub textmate_rules: Vec<proto::textmate::Rule>,
    pub semantic_rules: IndexMap<proto::semantic::Selector, proto::semantic::Style>,
    pub workbench_rules: IndexMap<Cow<'static, str>, proto::Color>,
}

impl ThemeBuilder {
    pub fn a(&mut self, selectors: impl IntoIterator<Item = Selector>, style: impl Into<Style>) {
        let mut textmate_scopes = Vec::new();
        let mut semantic_selectors = Vec::new();
        let style = style.into();

        for selector in selectors {
            match selector {
                Selector::TextMate(scope) => textmate_scopes.push(scope),
                Selector::Semantic(selector) => semantic_selectors.push(selector),
            }
        }

        if !textmate_scopes.is_empty() {
            self.textmate_rules.push(proto::textmate::Rule {
                scope: textmate_scopes,
                settings: style_to_textmate_rule_settings(style),
            });
        }

        let semantic_style = proto::semantic::Style {
            foreground: style.foreground,
            font_style: match style.font_style {
                Some(font_style) => {
                    let mut s = proto::semantic::FontStyle {
                        bold: proto::semantic::FontStyleSetting::Inherit,
                        italic: proto::semantic::FontStyleSetting::Inherit,
                        underline: proto::semantic::FontStyleSetting::Inherit,
                    };

                    *match font_style {
                        FontStyle::Bold => &mut s.bold,
                        FontStyle::Italic => &mut s.italic,
                        FontStyle::Underline => &mut s.underline,
                    } = proto::semantic::FontStyleSetting::True;

                    s
                }
                None => proto::semantic::FontStyle {
                    bold: proto::semantic::FontStyleSetting::Inherit,
                    italic: proto::semantic::FontStyleSetting::Inherit,
                    underline: proto::semantic::FontStyleSetting::Inherit,
                },
            },
        };

        for selector in semantic_selectors {
            self.semantic_rules.insert(selector, semantic_style);
        }

        fn style_to_textmate_rule_settings(style: Style) -> proto::textmate::RuleSettings {
            let font_style = match style.font_style {
                Some(font_style) => {
                    let mut s = (false, false, false);

                    *match font_style {
                        FontStyle::Bold => &mut s.0,
                        FontStyle::Italic => &mut s.1,
                        FontStyle::Underline => &mut s.2,
                    } = true;

                    proto::textmate::FontStyle::Set { bold: s.0, italic: s.1, underline: s.2 }
                }
                None => proto::textmate::FontStyle::Inherit,
            };

            proto::textmate::RuleSettings { foreground: style.foreground, font_style }
        }
    }

    pub fn w<'a>(&mut self, selector: impl IntoIterator<Item = &'a str>, color: impl Into<Color>) {
        let Color(color) = color.into();

        for selector in selector {
            self.workbench_rules.insert(Cow::Owned(selector.to_string()), color);
        }
    }

    pub fn build(self, name: impl Into<String>) -> proto::Theme {
        proto::Theme {
            name: name.into(),
            textmate_rules: self.textmate_rules,
            semantic_highlighting: proto::semantic::Highlighting::On { rules: self.semantic_rules },
            workbench_rules: self.workbench_rules,
        }
    }
}

pub fn tm(scope: impl Into<String>) -> Selector {
    Selector::TextMate(scope.into())
}

pub fn s(s: &str) -> Selector {
    return match parse(s) {
        Ok(s) => s,
        Err(e) => panic!("Failed to parse semantic selector ‘{s}’: {e}"),
    };

    fn parse(s: &str) -> Result<Selector, Cow<'static, str>> {
        let (s, language) = match s.rfind(':') {
            Some(idx) if idx == s.len() - 1 => {
                return Err("expected language name after ‘:’".into())
            }
            Some(idx) => {
                let language = s[idx + 1..].to_owned();
                let language = proto::semantic::Identifier::new(language)?;

                (&s[..idx], Some(language))
            }
            None => (s, None),
        };

        let mut components = s.split('.');

        let kind = match components.next() {
            Some("*") => proto::semantic::TokenKind::Wildcard,
            Some(kind) => {
                let kind = proto::semantic::Identifier::new(kind.to_owned())?;
                proto::semantic::TokenKind::Specific(kind)
            }
            None => return Err("expected semantic token kind".into()),
        };

        let modifiers = components
            .map(|m| proto::semantic::Identifier::new(m.to_owned()))
            .collect::<Result<_, _>>()?;

        Ok(Selector::Semantic(proto::semantic::Selector { kind, modifiers, language }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selector {
    TextMate(String),
    Semantic(proto::semantic::Selector),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    foreground: Option<proto::Color>,
    font_style: Option<FontStyle>,
}

impl<C> From<C> for Style
where
    C: Into<Color>,
{
    fn from(c: C) -> Self {
        let Color(c) = c.into();
        Self { foreground: Some(c), font_style: None }
    }
}

impl From<FontStyle> for Style {
    fn from(font_style: FontStyle) -> Self {
        Self { foreground: None, font_style: Some(font_style) }
    }
}

impl<C> From<(C, FontStyle)> for Style
where
    C: Into<Color>,
{
    fn from((c, font_style): (C, FontStyle)) -> Self {
        let Color(c) = c.into();
        Self { foreground: Some(c), font_style: Some(font_style) }
    }
}

pub struct Color(proto::Color);

impl From<u32> for Color {
    fn from(rgb: u32) -> Self {
        let (r, g, b) = rgb_from_u32(rgb);
        Self(proto::Color { r, g, b, a: 0xFF })
    }
}

impl From<(u32, u8)> for Color {
    fn from((rgb, a): (u32, u8)) -> Self {
        let (r, g, b) = rgb_from_u32(rgb);
        Self(proto::Color { r, g, b, a })
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self(proto::Color { r, g, b, a: 0xFF })
    }
}

impl From<((u8, u8, u8), u8)> for Color {
    fn from(((r, g, b), a): ((u8, u8, u8), u8)) -> Self {
        Self(proto::Color { r, g, b, a })
    }
}

fn rgb_from_u32(rgb: u32) -> (u8, u8, u8) {
    let [hi, r, g, b] = rgb.to_be_bytes();
    assert_eq!(hi, 0);

    (r, g, b)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontStyle {
    Bold,
    Italic,
    Underline,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::borrow::Cow;

    #[test]
    fn empty() {
        let t = ThemeBuilder::default();

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::On { rules: IndexMap::new() },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn add_single_textmate_scope_with_rgba_u32() {
        let mut t = ThemeBuilder::default();

        t.a([tm("keyword.operator")], 0xF92672);

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![proto::textmate::Rule {
                    scope: vec!["keyword.operator".to_string()],
                    settings: proto::textmate::RuleSettings {
                        foreground: Some(proto::Color { r: 0xF9, g: 0x26, b: 0x72, a: 0xFF }),
                        font_style: proto::textmate::FontStyle::Inherit
                    }
                }],
                semantic_highlighting: proto::semantic::Highlighting::On { rules: IndexMap::new() },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn add_multiple_textmate_scopes_with_rgba_u32() {
        let mut t = ThemeBuilder::default();

        t.a([tm("keyword.operator"), tm("punctuation"), tm("keyword.other")], 0xF92672);

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![proto::textmate::Rule {
                    scope: vec![
                        "keyword.operator".to_string(),
                        "punctuation".to_string(),
                        "keyword.other".to_string()
                    ],
                    settings: proto::textmate::RuleSettings {
                        foreground: Some(proto::Color { r: 0xF9, g: 0x26, b: 0x72, a: 0xFF }),
                        font_style: proto::textmate::FontStyle::Inherit
                    }
                }],
                semantic_highlighting: proto::semantic::Highlighting::On { rules: IndexMap::new() },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn add_single_semantic_selector_with_rgba_u32() {
        let mut t = ThemeBuilder::default();

        t.a([s("string")], 0xD49E9E);

        let mut rules = IndexMap::new();

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("string").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            proto::semantic::Style {
                foreground: Some(proto::Color { r: 0xD4, g: 0x9E, b: 0x9E, a: 0xFF }),
                font_style: proto::semantic::FontStyle {
                    bold: proto::semantic::FontStyleSetting::Inherit,
                    italic: proto::semantic::FontStyleSetting::Inherit,
                    underline: proto::semantic::FontStyleSetting::Inherit,
                },
            },
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::On { rules },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn add_multiple_semantic_selectors_with_rgba_u32() {
        let mut t = ThemeBuilder::default();

        t.a([s("number"), s("boolean"), s("enumMember")], 0xB5CEA8);

        let mut rules = IndexMap::new();

        let style = proto::semantic::Style {
            foreground: Some(proto::Color { r: 0xB5, g: 0xCE, b: 0xA8, a: 0xFF }),
            font_style: proto::semantic::FontStyle {
                bold: proto::semantic::FontStyleSetting::Inherit,
                italic: proto::semantic::FontStyleSetting::Inherit,
                underline: proto::semantic::FontStyleSetting::Inherit,
            },
        };

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("number").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            style,
        );

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("boolean").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            style,
        );

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("enumMember").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            style,
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::On { rules },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn add_semantic_selector_with_modifiers() {
        let mut t = ThemeBuilder::default();

        t.a([s("parameter"), s("variable.declaration.static"), s("function")], 0xD0AAFC);

        let mut rules = IndexMap::new();

        let style = proto::semantic::Style {
            foreground: Some(proto::Color { r: 0xD0, g: 0xAA, b: 0xFC, a: 0xFF }),
            font_style: proto::semantic::FontStyle {
                bold: proto::semantic::FontStyleSetting::Inherit,
                italic: proto::semantic::FontStyleSetting::Inherit,
                underline: proto::semantic::FontStyleSetting::Inherit,
            },
        };

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("parameter").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            style,
        );

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("variable").unwrap(),
                ),
                modifiers: vec![
                    proto::semantic::Identifier::new("declaration").unwrap(),
                    proto::semantic::Identifier::new("static").unwrap(),
                ],
                language: None,
            },
            style,
        );

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("function").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            style,
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::On { rules },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn add_semantic_and_textmate_selectors() {
        let mut t = ThemeBuilder::default();

        t.a([s("variable"), tm("variable")], 0xFFFFFF);

        let mut rules = IndexMap::new();

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("variable").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            proto::semantic::Style {
                foreground: Some(proto::Color { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF }),
                font_style: proto::semantic::FontStyle {
                    bold: proto::semantic::FontStyleSetting::Inherit,
                    italic: proto::semantic::FontStyleSetting::Inherit,
                    underline: proto::semantic::FontStyleSetting::Inherit,
                },
            },
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![proto::textmate::Rule {
                    scope: vec!["variable".to_string()],
                    settings: proto::textmate::RuleSettings {
                        foreground: Some(proto::Color { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF }),
                        font_style: proto::textmate::FontStyle::Inherit
                    }
                }],
                semantic_highlighting: proto::semantic::Highlighting::On { rules },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn rgba_u32_and_font_style() {
        let mut t = ThemeBuilder::default();

        t.a([tm("keyword"), s("keyword")], (0xEADFAF, FontStyle::Bold));

        let mut rules = IndexMap::new();

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("keyword").unwrap(),
                ),
                modifiers: Vec::new(),
                language: None,
            },
            proto::semantic::Style {
                foreground: Some(proto::Color { r: 0xEA, g: 0xDF, b: 0xAF, a: 0xFF }),
                font_style: proto::semantic::FontStyle {
                    bold: proto::semantic::FontStyleSetting::True,
                    italic: proto::semantic::FontStyleSetting::Inherit,
                    underline: proto::semantic::FontStyleSetting::Inherit,
                },
            },
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![proto::textmate::Rule {
                    scope: vec!["keyword".to_string()],
                    settings: proto::textmate::RuleSettings {
                        foreground: Some(proto::Color { r: 0xEA, g: 0xDF, b: 0xAF, a: 0xFF }),
                        font_style: proto::textmate::FontStyle::Set {
                            bold: true,
                            italic: false,
                            underline: false
                        }
                    }
                }],
                semantic_highlighting: proto::semantic::Highlighting::On { rules },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn font_style() {
        let mut t = ThemeBuilder::default();

        t.a([tm("markup.underline"), s("*.mutable")], FontStyle::Underline);

        let mut rules = IndexMap::new();

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Wildcard,
                modifiers: vec![proto::semantic::Identifier::new("mutable").unwrap()],
                language: None,
            },
            proto::semantic::Style {
                foreground: None,
                font_style: proto::semantic::FontStyle {
                    bold: proto::semantic::FontStyleSetting::Inherit,
                    italic: proto::semantic::FontStyleSetting::Inherit,
                    underline: proto::semantic::FontStyleSetting::True,
                },
            },
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: vec![proto::textmate::Rule {
                    scope: vec!["markup.underline".to_string()],
                    settings: proto::textmate::RuleSettings {
                        foreground: None,
                        font_style: proto::textmate::FontStyle::Set {
                            bold: false,
                            italic: false,
                            underline: true
                        }
                    }
                }],
                semantic_highlighting: proto::semantic::Highlighting::On { rules },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn semantic_language() {
        let mut t = ThemeBuilder::default();

        t.a([s("variable.constant:rust")], 0xFF0000);

        let mut rules = IndexMap::new();

        rules.insert(
            proto::semantic::Selector {
                kind: proto::semantic::TokenKind::Specific(
                    proto::semantic::Identifier::new("variable").unwrap(),
                ),
                modifiers: vec![proto::semantic::Identifier::new("constant").unwrap()],
                language: Some(proto::semantic::Identifier::new("rust").unwrap()),
            },
            proto::semantic::Style {
                foreground: Some(proto::Color { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }),
                font_style: proto::semantic::FontStyle {
                    bold: proto::semantic::FontStyleSetting::Inherit,
                    italic: proto::semantic::FontStyleSetting::Inherit,
                    underline: proto::semantic::FontStyleSetting::Inherit,
                },
            },
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::On { rules },
                workbench_rules: IndexMap::new(),
            }
        );
    }

    #[test]
    fn workbench_rules() {
        let mut t = ThemeBuilder::default();

        t.w(["editor.background"], 0x111111);
        t.w(["editor.foreground", "foreground"], 0xBCBCBC);

        let mut workbench_rules = IndexMap::new();
        workbench_rules.insert(
            Cow::Borrowed("editor.background"),
            proto::Color { r: 0x11, g: 0x11, b: 0x11, a: 0xFF },
        );
        workbench_rules.insert(
            Cow::Borrowed("editor.foreground"),
            proto::Color { r: 0xBC, g: 0xBC, b: 0xBC, a: 0xFF },
        );
        workbench_rules.insert(
            Cow::Borrowed("foreground"),
            proto::Color { r: 0xBC, g: 0xBC, b: 0xBC, a: 0xFF },
        );

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::On { rules: IndexMap::new() },
                workbench_rules,
            }
        );
    }
}
