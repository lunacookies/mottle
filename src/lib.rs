pub mod dsl;
pub mod proto;

use serde::Serialize;
use serde_json::ser::PrettyFormatter;
use serde_json::Serializer;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

pub fn save_theme(theme: &proto::Theme) -> Result<(), SaveThemeError> {
    let themes_dir = prepare_themes_dir()?;
    let theme_path = themes_dir.join(format!("{}-color-theme.json", theme.name));

    fs::write(&theme_path, serialize_theme(theme))
        .map_err(|e| SaveThemeError::WriteTheme(e, theme_path))?;

    Ok(())
}

pub fn serialize_theme(theme: &proto::Theme) -> String {
    let mut v = Vec::new();
    let mut serializer = Serializer::with_formatter(&mut v, PrettyFormatter::with_indent(b"    "));
    theme.serialize(&mut serializer).unwrap();

    String::from_utf8(v).unwrap()
}

fn prepare_themes_dir() -> Result<&'static Path, SaveThemeError> {
    let themes_dir = Path::new("themes");

    if !themes_dir.exists() {
        fs::create_dir(themes_dir).map_err(SaveThemeError::CreateThemesDir)?;
    } else if !themes_dir.is_dir() {
        return Err(SaveThemeError::ThemesDirIsNotDir);
    }

    Ok(themes_dir)
}

#[derive(Debug, Error)]
pub enum SaveThemeError {
    #[error("failed creating `themes/` directory")]
    CreateThemesDir(#[source] io::Error),
    #[error("`themes/` already exists and is not a directory")]
    ThemesDirIsNotDir,
    #[error("failed writing theme to `{1}`")]
    WriteTheme(#[source] io::Error, PathBuf),
}
