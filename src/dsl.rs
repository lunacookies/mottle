use crate::proto;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::ops::{BitOr, BitXor, Shr};

#[derive(Debug, Default)]
pub struct ThemeBuilder {
    pub textmate_rules: Vec<proto::textmate::Rule>,
    pub semantic_rules: IndexMap<proto::semantic::Selector, proto::semantic::Style>,
    pub workbench_rules: IndexMap<Cow<'static, str>, proto::Color>,
}

impl ThemeBuilder {
    pub fn a(
        &mut self,
        selectors: impl Into<SemanticOrTextMateSelectors>,
        style: impl Into<Style>,
    ) {
        let mut textmate_scopes = Vec::new();
        let mut semantic_selectors = Vec::new();
        let selectors = selectors.into();
        let style = style.into();

        for selector in selectors.0 {
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

    pub fn w(&mut self, selector: WorkbenchSelectors, color: u32) {
        let [r, g, b, a] = color.to_be_bytes();

        for selector in selector.0 {
            self.workbench_rules.insert(selector.into(), proto::Color { r, g, b, a });
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

pub fn tm(scope: impl Into<String>) -> SemanticOrTextMateSelectors {
    SemanticOrTextMateSelectors(vec![Selector::TextMate(scope.into())])
}

pub fn s(token_kind: impl IntoTokenKind) -> SemanticOrTextMateSelectors {
    SemanticOrTextMateSelectors(vec![Selector::Semantic(proto::semantic::Selector {
        kind: token_kind.into_token_kind(),
        modifiers: Vec::new(),
        language: None,
    })])
}

pub fn w(selector: impl Into<String>) -> WorkbenchSelectors {
    WorkbenchSelectors(vec![selector.into()])
}

pub trait IntoTokenKind {
    fn into_token_kind(self) -> proto::semantic::TokenKind;
}

impl IntoTokenKind for &'static str {
    fn into_token_kind(self) -> proto::semantic::TokenKind {
        proto::semantic::TokenKind::Specific(proto::semantic::Identifier::new(self).unwrap())
    }
}

impl IntoTokenKind for String {
    fn into_token_kind(self) -> proto::semantic::TokenKind {
        proto::semantic::TokenKind::Specific(proto::semantic::Identifier::new(self).unwrap())
    }
}

impl IntoTokenKind for char {
    fn into_token_kind(self) -> proto::semantic::TokenKind {
        assert_eq!(self, '*', "only use a `char` semantic token kind with '*' to signify wildcard");
        proto::semantic::TokenKind::Wildcard
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticOrTextMateSelectors(Vec<Selector>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selector {
    TextMate(String),
    Semantic(proto::semantic::Selector),
}

impl BitOr for SemanticOrTextMateSelectors {
    type Output = Self;

    fn bitor(mut self, mut rhs: Self) -> Self::Output {
        self.0.append(&mut rhs.0);
        self
    }
}

impl Shr<&'static str> for SemanticOrTextMateSelectors {
    type Output = Self;

    fn shr(mut self, rhs: &'static str) -> Self::Output {
        for selector in &mut self.0 {
            match selector {
                Selector::Semantic(semantic) => {
                    semantic.modifiers.push(proto::semantic::Identifier::new(rhs).unwrap());
                }
                Selector::TextMate(_) => panic!("cannot add a modifier to TextMate selector"),
            }
        }

        self
    }
}

impl BitXor<&'static str> for SemanticOrTextMateSelectors {
    type Output = Self;

    fn bitxor(mut self, rhs: &'static str) -> Self::Output {
        for selector in &mut self.0 {
            match selector {
                Selector::Semantic(semantic) => {
                    semantic.language = Some(proto::semantic::Identifier::new(rhs).unwrap());
                }
                Selector::TextMate(_) => panic!("cannot add set language of TextMate selector"),
            }
        }

        self
    }
}

pub struct WorkbenchSelectors(Vec<String>);

impl BitOr for WorkbenchSelectors {
    type Output = Self;

    fn bitor(mut self, mut rhs: Self) -> Self::Output {
        self.0.append(&mut rhs.0);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    foreground: Option<proto::Color>,
    font_style: Option<FontStyle>,
}

impl From<u32> for Style {
    fn from(rgba: u32) -> Self {
        let [r, g, b, a] = rgba.to_be_bytes();
        Self { foreground: Some(proto::Color { r, g, b, a }), font_style: None }
    }
}

impl From<FontStyle> for Style {
    fn from(font_style: FontStyle) -> Self {
        Self { foreground: None, font_style: Some(font_style) }
    }
}

impl From<(u32, FontStyle)> for Style {
    fn from((rgba, font_style): (u32, FontStyle)) -> Self {
        let [r, g, b, a] = rgba.to_be_bytes();
        Self { foreground: Some(proto::Color { r, g, b, a }), font_style: Some(font_style) }
    }
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

        t.a(tm("keyword.operator"), 0xF92672FF);

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

        t.a(tm("keyword.operator") | tm("punctuation") | tm("keyword.other"), 0xF92672FF);

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

        t.a(s("string"), 0xD49E9EFF);

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

        t.a(s("number") | s("boolean") | s("enumMember"), 0xB5CEA8FF);

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

        t.a(
            s("parameter") | s("variable") >> "declaration" >> "static" | s("function"),
            0xD0AAFCFF,
        );

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

        t.a(s("variable") | tm("variable"), 0xFFFFFFFF);

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

        t.a(tm("keyword") | s("keyword"), (0xEADFAFFF, FontStyle::Bold));

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

        t.a(tm("markup.underline") | s('*') >> "mutable", FontStyle::Underline);

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

        t.a(s("variable") >> "constant" ^ "rust", 0xFF0000FF);

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

        t.w(w("editor.background"), 0x111111FF);
        t.w(w("editor.foreground") | w("foreground"), 0xBCBCBCFF);

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
