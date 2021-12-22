pub mod semantic_highlighting_typestate;

use self::semantic_highlighting_typestate::{
    SemanticHighlightingDisabled, SemanticHighlightingEnabled, SemanticHighlightingState,
};
use crate::proto;
use indexmap::IndexMap;
use std::ops::{BitOr, Shr};

#[derive(Debug)]
pub struct ThemeBuilder<S> {
    pub textmate_rules: Vec<proto::textmate::Rule>,
    pub semantic_highlighting: S,
}

impl ThemeBuilder<SemanticHighlightingEnabled> {
    pub fn new_with_semantic_highlighting() -> Self {
        Self {
            textmate_rules: Vec::new(),
            semantic_highlighting: SemanticHighlightingEnabled { rules: IndexMap::new() },
        }
    }
}

impl ThemeBuilder<SemanticHighlightingDisabled> {
    pub fn new_without_semantic_highlighting() -> Self {
        Self { textmate_rules: Vec::new(), semantic_highlighting: SemanticHighlightingDisabled }
    }
}

impl<S> ThemeBuilder<S>
where
    S: SemanticHighlightingState,
{
    pub fn a(&mut self, selectors: impl Into<S::Selectors>, style: impl Into<Style>) {
        self.semantic_highlighting.add_rule(
            selectors.into(),
            style.into(),
            &mut self.textmate_rules,
        );
    }

    pub fn build(self, name: impl Into<String>) -> proto::Theme {
        proto::Theme {
            name: name.into(),
            textmate_rules: self.textmate_rules,
            semantic_highlighting: self.semantic_highlighting.into_proto(),
        }
    }
}

impl ThemeBuilder<SemanticHighlightingEnabled> {
    pub fn tm(&self, scope: impl Into<String>) -> SemanticOrTextMateSelectors {
        SemanticOrTextMateSelectors(vec![SemanticOrTextMateSelector::TextMate(scope.into())])
    }

    pub fn s(&self, token_kind: impl IntoTokenKind) -> SemanticSelector {
        SemanticSelector(proto::semantic::Selector {
            kind: token_kind.into_token_kind(),
            modifiers: Vec::new(),
            language: None,
        })
    }
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

impl ThemeBuilder<SemanticHighlightingDisabled> {
    pub fn tm(&self, scope: impl Into<String>) -> TextMateScopes {
        TextMateScopes(vec![scope.into()])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticOrTextMateSelectors(Vec<SemanticOrTextMateSelector>);

impl From<SemanticSelector> for SemanticOrTextMateSelectors {
    fn from(semantic_selector: SemanticSelector) -> Self {
        Self(vec![SemanticOrTextMateSelector::Semantic(semantic_selector)])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticOrTextMateSelector {
    TextMate(String),
    Semantic(SemanticSelector),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextMateScopes(Vec<String>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticSelector(proto::semantic::Selector);

impl BitOr for TextMateScopes {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.0.extend_from_slice(&rhs.0);
        self
    }
}

impl BitOr<SemanticSelector> for SemanticOrTextMateSelectors {
    type Output = SemanticOrTextMateSelectors;

    fn bitor(mut self, rhs: SemanticSelector) -> Self::Output {
        self.0.push(SemanticOrTextMateSelector::Semantic(rhs));
        self
    }
}

impl BitOr for SemanticSelector {
    type Output = SemanticOrTextMateSelectors;

    fn bitor(self, rhs: Self) -> Self::Output {
        SemanticOrTextMateSelectors(vec![
            SemanticOrTextMateSelector::Semantic(self),
            SemanticOrTextMateSelector::Semantic(rhs),
        ])
    }
}

impl BitOr<SemanticOrTextMateSelectors> for SemanticSelector {
    type Output = SemanticOrTextMateSelectors;

    fn bitor(self, mut rhs: SemanticOrTextMateSelectors) -> Self::Output {
        let mut selectors =
            SemanticOrTextMateSelectors(vec![SemanticOrTextMateSelector::Semantic(self)]);
        selectors.0.append(&mut rhs.0);

        selectors
    }
}

impl Shr<&'static str> for SemanticSelector {
    type Output = Self;

    fn shr(mut self, rhs: &'static str) -> Self::Output {
        self.0.modifiers.push(proto::semantic::Identifier::new(rhs).unwrap());
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

    #[test]
    fn empty_with_semantic_highlighting() {
        let t = ThemeBuilder::new_with_semantic_highlighting();

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::On { rules: IndexMap::new() }
            }
        );
    }

    #[test]
    fn empty_without_semantic_highlighting() {
        let t = ThemeBuilder::new_without_semantic_highlighting();

        assert_eq!(
            t.build("My cool theme"),
            proto::Theme {
                name: "My cool theme".to_string(),
                textmate_rules: Vec::new(),
                semantic_highlighting: proto::semantic::Highlighting::Off
            }
        );
    }

    #[test]
    fn add_single_textmate_scope_with_rgba_u32() {
        let mut t = ThemeBuilder::new_without_semantic_highlighting();

        t.a(t.tm("keyword.operator"), 0xF92672FF);

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
                semantic_highlighting: proto::semantic::Highlighting::Off
            }
        );
    }

    #[test]
    fn add_multiple_textmate_scopes_with_rgba_u32() {
        let mut t = ThemeBuilder::new_without_semantic_highlighting();

        t.a(t.tm("keyword.operator") | t.tm("punctuation") | t.tm("keyword.other"), 0xF92672FF);

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
                semantic_highlighting: proto::semantic::Highlighting::Off
            }
        );
    }

    #[test]
    fn add_single_semantic_selector_with_rgba_u32() {
        let mut t = ThemeBuilder::new_with_semantic_highlighting();

        t.a(t.s("string"), 0xD49E9EFF);

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
                semantic_highlighting: proto::semantic::Highlighting::On { rules }
            }
        );
    }

    #[test]
    fn add_multiple_semantic_selectors_with_rgba_u32() {
        let mut t = ThemeBuilder::new_with_semantic_highlighting();

        t.a(t.s("number") | t.s("boolean") | t.s("enumMember"), 0xB5CEA8FF);

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
                semantic_highlighting: proto::semantic::Highlighting::On { rules }
            }
        );
    }

    #[test]
    fn add_semantic_selector_with_modifiers() {
        let mut t = ThemeBuilder::new_with_semantic_highlighting();

        t.a(
            t.s("parameter") | t.s("variable") >> "declaration" >> "static" | t.s("function"),
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
                semantic_highlighting: proto::semantic::Highlighting::On { rules }
            }
        );
    }

    #[test]
    fn add_semantic_and_textmate_selectors() {
        let mut t = ThemeBuilder::new_with_semantic_highlighting();

        t.a(t.s("variable") | t.tm("variable"), 0xFFFFFFFF);

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
                semantic_highlighting: proto::semantic::Highlighting::On { rules }
            }
        );
    }

    #[test]
    fn rgba_u32_and_font_style() {
        let mut t = ThemeBuilder::new_with_semantic_highlighting();

        t.a(t.tm("keyword") | t.s("keyword"), (0xEADFAFFF, FontStyle::Bold));

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
                    italic: proto::semantic::FontStyleSetting::False,
                    underline: proto::semantic::FontStyleSetting::False,
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
                semantic_highlighting: proto::semantic::Highlighting::On { rules }
            }
        );
    }

    #[test]
    fn font_style() {
        let mut t = ThemeBuilder::new_with_semantic_highlighting();

        t.a(t.tm("markup.underline") | t.s('*') >> "mutable", FontStyle::Underline);

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
                    bold: proto::semantic::FontStyleSetting::False,
                    italic: proto::semantic::FontStyleSetting::False,
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
                semantic_highlighting: proto::semantic::Highlighting::On { rules }
            }
        );
    }
}
