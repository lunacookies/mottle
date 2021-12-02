use super::{
    FontStyle, SemanticOrTextMateSelector, SemanticOrTextMateSelectors, Style, TextMateScopes,
};
use crate::proto;
use indexmap::IndexMap;

pub trait SemanticHighlightingState {
    fn into_proto(self) -> proto::semantic::Highlighting;

    type Selectors;
    fn add_rule(
        &mut self,
        selectors: Self::Selectors,
        style: Style,
        textmate_rules: &mut Vec<proto::textmate::Rule>,
    );
}

#[non_exhaustive]
pub struct SemanticHighlightingEnabled {
    pub rules: IndexMap<proto::semantic::Selector, proto::semantic::Style>,
}

#[non_exhaustive]
pub struct SemanticHighlightingDisabled;

impl SemanticHighlightingState for SemanticHighlightingEnabled {
    fn into_proto(self) -> proto::semantic::Highlighting {
        proto::semantic::Highlighting::On { rules: self.rules }
    }

    type Selectors = SemanticOrTextMateSelectors;

    fn add_rule(
        &mut self,
        selectors: Self::Selectors,
        style: Style,
        textmate_rules: &mut Vec<proto::textmate::Rule>,
    ) {
        let mut textmate_scopes = Vec::new();
        let mut semantic_selectors = Vec::new();

        for selector in selectors.0 {
            match selector {
                SemanticOrTextMateSelector::TextMate(scope) => textmate_scopes.push(scope),
                SemanticOrTextMateSelector::Semantic(selector) => {
                    semantic_selectors.push(selector.0)
                }
            }
        }

        if !textmate_scopes.is_empty() {
            textmate_rules.push(proto::textmate::Rule {
                scope: textmate_scopes,
                settings: style_to_textmate_rule_settings(style),
            });
        }

        let semantic_style = proto::semantic::Style {
            foreground: style.foreground,
            font_style: match style.font_style {
                Some(font_style) => {
                    let mut s = proto::semantic::FontStyle {
                        bold: proto::semantic::FontStyleSetting::False,
                        italic: proto::semantic::FontStyleSetting::False,
                        underline: proto::semantic::FontStyleSetting::False,
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
            self.rules.insert(selector, semantic_style);
        }
    }
}

impl SemanticHighlightingState for SemanticHighlightingDisabled {
    fn into_proto(self) -> proto::semantic::Highlighting {
        proto::semantic::Highlighting::Off
    }

    type Selectors = TextMateScopes;

    fn add_rule(
        &mut self,
        selectors: Self::Selectors,
        style: Style,
        textmate_rules: &mut Vec<proto::textmate::Rule>,
    ) {
        textmate_rules.push(proto::textmate::Rule {
            scope: selectors.0,
            settings: style_to_textmate_rule_settings(style),
        });
    }
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
