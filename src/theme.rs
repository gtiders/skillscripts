use anyhow::Result;
use chromata::{Theme, base24};
use ratatui::style::Color;
use serde::Serialize;

pub(crate) const DEFAULT_NAME: &str = "Catppuccin Frappe";

#[derive(Serialize)]
pub(crate) struct ThemeInfo<'a> {
    name: &'a str,
    variant: &'a str,
}

pub(crate) fn available() -> Vec<ThemeInfo<'static>> {
    base24::THEMES
        .iter()
        .map(|theme| ThemeInfo {
            name: theme.name.as_ref(),
            variant: if theme.is_dark() { "dark" } else { "light" },
        })
        .collect()
}

pub(crate) fn resolve(name: &str) -> Result<&'static Theme> {
    base24::THEMES
        .iter()
        .copied()
        .find(|theme| theme.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "unknown picker theme `{name}`; run `sks themes` to list available themes"
            )
        })
}

pub(crate) fn configured(name: Option<&str>) -> Result<&'static Theme> {
    resolve(name.unwrap_or(DEFAULT_NAME))
}

#[derive(Clone, Copy)]
pub(crate) struct Colors {
    pub(crate) background: Color,
    pub(crate) panel: Color,
    pub(crate) card: Color,
    pub(crate) selected: Color,
    pub(crate) text: Color,
    pub(crate) muted: Color,
    pub(crate) border: Color,
    pub(crate) card_border: Color,
    pub(crate) accent: Color,
    pub(crate) path_label: Color,
    pub(crate) command_label: Color,
    pub(crate) comment_label: Color,
    pub(crate) tags_label: Color,
    pub(crate) help_key: Color,
    pub(crate) match_text: Color,
    pub(crate) match_background: Color,
}

pub(crate) fn colors(theme: &Theme) -> Colors {
    let accent = theme.accent();
    let panel = theme.bg.lerp(theme.fg, 0.04);
    let border = theme.fg.lerp(theme.bg, 0.55);
    Colors {
        background: theme.bg.into(),
        panel: panel.into(),
        card: theme.bg.lerp(theme.fg, 0.08).into(),
        selected: theme
            .selection
            .unwrap_or(theme.bg.lerp(accent, 0.25))
            .into(),
        text: theme.fg.into(),
        muted: theme.gutter.unwrap_or(theme.fg.lerp(theme.bg, 0.4)).into(),
        border: border.into(),
        card_border: theme.statusbar_fg.unwrap_or(border).into(),
        accent: accent.into(),
        path_label: theme.info.or(theme.blue).unwrap_or(accent).into(),
        command_label: theme.success.or(theme.green).unwrap_or(accent).into(),
        comment_label: theme.keyword.or(theme.purple).unwrap_or(accent).into(),
        tags_label: theme.warning.or(theme.orange).unwrap_or(accent).into(),
        help_key: theme.cyan.unwrap_or(accent).into(),
        match_text: theme.bg.into(),
        match_background: theme.yellow.unwrap_or(accent).into(),
    }
}
