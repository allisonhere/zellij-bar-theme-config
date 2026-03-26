use crate::theme::{RgbColor, Theme, ThemeComponent, ThemeComponentType};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("KDL parse error: {0}")]
    KdlParse(String),
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
}

pub struct ConfigManager {
    config_dir: PathBuf,
    themes_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zellij");
        let themes_dir = config_dir.join("themes");

        Self {
            config_dir,
            themes_dir,
        }
    }

    pub fn ensure_themes_dir(&self) -> Result<(), ConfigError> {
        if !self.themes_dir.exists() {
            fs::create_dir_all(&self.themes_dir)?;
        }
        Ok(())
    }

    pub fn list_themes(&self) -> Result<Vec<String>, ConfigError> {
        let mut themes = Vec::new();

        if self.themes_dir.exists() {
            for entry in fs::read_dir(&self.themes_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "kdl") {
                    if let Some(stem) = path.file_stem() {
                        themes.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(themes)
    }

    pub fn load_theme(&self, name: &str) -> Result<Theme, ConfigError> {
        let kdl_path = self.themes_dir.join(format!("{}.kdl", name));

        if kdl_path.exists() {
            let content = fs::read_to_string(&kdl_path)?;
            parse_theme_kdl(&content, name)
        } else {
            Ok(Theme::default())
        }
    }

    pub fn save_theme(&self, theme: &Theme) -> Result<(), ConfigError> {
        self.ensure_themes_dir()?;
        let kdl_path = self.themes_dir.join(format!("{}.kdl", theme.name));
        let content = theme_to_kdl(theme);
        fs::write(kdl_path, content)?;
        Ok(())
    }

    pub fn rename_theme(&self, old_name: &str, new_name: &str) -> Result<(), ConfigError> {
        let old_path = self.themes_dir.join(format!("{}.kdl", old_name));
        let new_path = self.themes_dir.join(format!("{}.kdl", new_name));
        fs::rename(&old_path, &new_path)?;
        Ok(())
    }

    pub fn delete_theme(&self, name: &str) -> Result<(), ConfigError> {
        let path = self.themes_dir.join(format!("{}.kdl", name));
        fs::remove_file(&path)?;
        Ok(())
    }

    pub fn apply_theme_to_zellij(&self, theme: &Theme) -> Result<(), ConfigError> {
        self.save_theme(theme)?;

        let config_path = self.config_dir.join("config.kdl");
        let existing = if config_path.exists() {
            fs::read_to_string(&config_path)?
        } else {
            String::new()
        };

        let theme_line = format!("theme \"{}\"", theme.name);
        let updated = if existing.lines().any(|l| l.trim_start().starts_with("theme ")) {
            existing
                .lines()
                .map(|l| {
                    if l.trim_start().starts_with("theme ") {
                        theme_line.as_str()
                    } else {
                        l
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        } else if existing.is_empty() {
            format!("{}\n", theme_line)
        } else {
            format!("{}\n{}\n", existing.trim_end(), theme_line)
        };

        fs::write(config_path, updated)?;
        Ok(())
    }

}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_theme_kdl(content: &str, name: &str) -> Result<Theme, ConfigError> {
    let doc: kdl::KdlDocument = content
        .parse()
        .map_err(|e: kdl::KdlError| ConfigError::KdlParse(e.to_string()))?;

    let theme_node =
        if let Some(themes_node) = doc.nodes().iter().find(|n| n.name().value() == "themes") {
            themes_node
                .children()
                .as_ref()
                .and_then(|children| children.nodes().iter().find(|n| n.name().value() == name))
                .ok_or_else(|| ConfigError::ThemeNotFound(name.to_string()))?
        } else {
            doc.nodes()
                .iter()
                .find(|n| n.name().value() == name)
                .ok_or_else(|| ConfigError::ThemeNotFound(name.to_string()))?
        };

    let children: Vec<_> = theme_node
        .children()
        .as_ref()
        .map(|d| d.nodes().to_vec())
        .unwrap_or_default();

    // Prefer explicit component blocks when present. Saved themes include both
    // palette keys and per-component data, and the component data should win.
    let has_component_nodes = children.iter().any(|n| {
        matches!(
            n.name().value(),
            "text_unselected"
                | "text_selected"
                | "ribbon_unselected"
                | "ribbon_selected"
                | "table_title"
                | "table_cell_unselected"
                | "table_cell_selected"
                | "list_unselected"
                | "list_selected"
                | "frame_unselected"
                | "frame_selected"
                | "frame_highlight"
                | "exit_code_success"
                | "exit_code_error"
        )
    });

    // Pure palette themes are still supported for built-ins and old files.
    let palette_keys = [
        "fg", "bg", "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    ];
    let is_palette_only = !has_component_nodes
        && children
            .iter()
            .any(|n| palette_keys.contains(&n.name().value()));

    if is_palette_only {
        return Ok(theme_from_palette(&children, name));
    }

    let mut theme = Theme::default();
    theme.name = name.to_string();

    for child in &children {
        let component_type = match child.name().value() {
            "text_unselected" => ThemeComponentType::TextUnselected,
            "text_selected" => ThemeComponentType::TextSelected,
            "ribbon_unselected" => ThemeComponentType::RibbonUnselected,
            "ribbon_selected" => ThemeComponentType::RibbonSelected,
            "table_title" => ThemeComponentType::TableTitle,
            "table_cell_unselected" => ThemeComponentType::TableCellUnselected,
            "table_cell_selected" => ThemeComponentType::TableCellSelected,
            "list_unselected" => ThemeComponentType::ListUnselected,
            "list_selected" => ThemeComponentType::ListSelected,
            "frame_unselected" => ThemeComponentType::FrameUnselected,
            "frame_selected" => ThemeComponentType::FrameSelected,
            "frame_highlight" => ThemeComponentType::FrameHighlight,
            "exit_code_success" => ThemeComponentType::ExitCodeSuccess,
            "exit_code_error" => ThemeComponentType::ExitCodeError,
            _ => continue,
        };

        let component = parse_component(child);
        *theme.get_mut(component_type) = component;
    }

    Ok(theme)
}

fn parse_palette_color(nodes: &[kdl::KdlNode], key: &str) -> Option<RgbColor> {
    let node = nodes.iter().find(|n| n.name().value() == key)?;
    let entries: Vec<_> = node.entries().iter().collect();
    if entries.len() < 3 {
        return None;
    }
    let r = entries[0].value().as_i64()?.clamp(0, 255) as u8;
    let g = entries[1].value().as_i64()?.clamp(0, 255) as u8;
    let b = entries[2].value().as_i64()?.clamp(0, 255) as u8;
    Some(RgbColor::new(r, g, b))
}

fn theme_from_palette(nodes: &[kdl::KdlNode], name: &str) -> Theme {
    let black   = parse_palette_color(nodes, "black")  .unwrap_or(RgbColor::new(30, 30, 30));
    let fg      = parse_palette_color(nodes, "fg")     .unwrap_or(RgbColor::new(200, 200, 200));
    let bg      = parse_palette_color(nodes, "bg")     .unwrap_or(RgbColor::new(40, 40, 40));
    let white   = parse_palette_color(nodes, "white")  .unwrap_or(RgbColor::new(240, 240, 240));
    let red     = parse_palette_color(nodes, "red")    .unwrap_or(RgbColor::new(220, 80, 80));
    let green   = parse_palette_color(nodes, "green")  .unwrap_or(RgbColor::new(80, 200, 80));
    let yellow  = parse_palette_color(nodes, "yellow") .unwrap_or(RgbColor::new(220, 190, 100));
    let blue    = parse_palette_color(nodes, "blue")   .unwrap_or(RgbColor::new(80, 130, 210));
    let _magenta = parse_palette_color(nodes, "magenta").unwrap_or(RgbColor::new(180, 100, 200));
    let orange  = parse_palette_color(nodes, "orange") .unwrap_or(yellow);

    // A slightly lighter bg for "selected" backgrounds
    let bg_sel = RgbColor::new(
        bg.r.saturating_add(25),
        bg.g.saturating_add(25),
        bg.b.saturating_add(25),
    );

    let mk = |base: RgbColor, background: RgbColor| ThemeComponent {
        base,
        background,
        emphasis_0: white,
        emphasis_1: fg,
        emphasis_2: RgbColor::new(fg.r / 2 + 60, fg.g / 2 + 60, fg.b / 2 + 60),
        emphasis_3: black,
    };

    let mut components = HashMap::new();
    components.insert(ThemeComponentType::TextUnselected,       mk(fg,      bg));
    components.insert(ThemeComponentType::TextSelected,         mk(white,   bg_sel));
    components.insert(ThemeComponentType::RibbonUnselected,     mk(fg,      bg));
    components.insert(ThemeComponentType::RibbonSelected,       mk(bg,      blue));
    components.insert(ThemeComponentType::TableTitle,           mk(blue,    bg));
    components.insert(ThemeComponentType::TableCellUnselected,  mk(fg,      bg));
    components.insert(ThemeComponentType::TableCellSelected,    mk(white,   bg_sel));
    components.insert(ThemeComponentType::ListUnselected,       mk(fg,      bg));
    components.insert(ThemeComponentType::ListSelected,         mk(white,   blue));
    components.insert(ThemeComponentType::FrameUnselected,      mk(black,   bg));
    components.insert(ThemeComponentType::FrameSelected,        mk(blue,    bg));
    components.insert(ThemeComponentType::FrameHighlight,       mk(orange,  bg));
    components.insert(ThemeComponentType::ExitCodeSuccess,      mk(green,   bg));
    components.insert(ThemeComponentType::ExitCodeError,        mk(red,     bg));

    Theme {
        name: name.to_string(),
        components,
    }
}

fn parse_component(node: &kdl::KdlNode) -> ThemeComponent {
    let mut component = ThemeComponent::default();

    let children = match node.children() {
        Some(doc) => doc.nodes().to_vec(),
        None => return component,
    };

    for child in &children {
        let entries: Vec<_> = child.entries().iter().collect();
        if entries.len() < 3 {
            continue;
        }
        let r = entries[0].value().as_i64().unwrap_or(0).clamp(0, 255) as u8;
        let g = entries[1].value().as_i64().unwrap_or(0).clamp(0, 255) as u8;
        let b = entries[2].value().as_i64().unwrap_or(0).clamp(0, 255) as u8;
        let color = RgbColor::new(r, g, b);

        match child.name().value() {
            "base" => component.base = color,
            "background" => component.background = color,
            "emphasis_0" => component.emphasis_0 = color,
            "emphasis_1" => component.emphasis_1 = color,
            "emphasis_2" => component.emphasis_2 = color,
            "emphasis_3" => component.emphasis_3 = color,
            _ => {}
        }
    }

    component
}

fn theme_to_kdl(theme: &Theme) -> String {
    let mut output = String::new();
    output.push_str("themes {\n");
    output.push_str(&format!("    {} {{\n", theme.name));

    // Include standard Zellij palette keys so older Zellij versions (e.g. on
    // Ubuntu/Pop!_OS) can parse the file without "missing fg" errors.
    let fg      = theme.get(ThemeComponentType::TextUnselected).base;
    let bg      = theme.get(ThemeComponentType::TextUnselected).background;
    let white   = theme.get(ThemeComponentType::TextSelected).base;
    let black   = theme.get(ThemeComponentType::TextUnselected).emphasis_3;
    let blue    = theme.get(ThemeComponentType::RibbonSelected).background;
    let green   = theme.get(ThemeComponentType::ExitCodeSuccess).base;
    let red     = theme.get(ThemeComponentType::ExitCodeError).base;
    let yellow  = theme.get(ThemeComponentType::FrameHighlight).base;
    let magenta = theme.get(ThemeComponentType::RibbonUnselected).emphasis_0;
    let cyan    = theme.get(ThemeComponentType::RibbonUnselected).emphasis_2;
    let orange  = theme.get(ThemeComponentType::FrameHighlight).base;

    for (key, c) in &[
        ("fg", fg), ("bg", bg), ("black", black), ("red", red),
        ("green", green), ("yellow", yellow), ("blue", blue),
        ("magenta", magenta), ("cyan", cyan), ("white", white),
        ("orange", orange),
    ] {
        output.push_str(&format!("        {} {} {} {}\n", key, c.r, c.g, c.b));
    }

    for component_type in ThemeComponentType::all() {
        let component = theme.get(*component_type);
        output.push_str(&format!("        {} {{\n", component_type.component_key()));
        output.push_str(&format!(
            "            base {} {} {}\n",
            component.base.r, component.base.g, component.base.b
        ));
        output.push_str(&format!(
            "            background {} {} {}\n",
            component.background.r, component.background.g, component.background.b
        ));
        output.push_str(&format!(
            "            emphasis_0 {} {} {}\n",
            component.emphasis_0.r, component.emphasis_0.g, component.emphasis_0.b
        ));
        output.push_str(&format!(
            "            emphasis_1 {} {} {}\n",
            component.emphasis_1.r, component.emphasis_1.g, component.emphasis_1.b
        ));
        output.push_str(&format!(
            "            emphasis_2 {} {} {}\n",
            component.emphasis_2.r, component.emphasis_2.g, component.emphasis_2.b
        ));
        output.push_str(&format!(
            "            emphasis_3 {} {} {}\n",
            component.emphasis_3.r, component.emphasis_3.g, component.emphasis_3.b
        ));
        output.push_str("        }\n");
    }

    output.push_str("    }\n");
    output.push_str("}\n");
    output
}

#[cfg(test)]
mod tests {
    use super::{parse_theme_kdl, theme_to_kdl};
    use crate::theme::{RgbColor, Theme, ThemeComponentType};

    #[test]
    fn saved_theme_round_trips_component_edits() {
        let mut theme = Theme::default();
        theme.name = "round-trip".to_string();
        theme.get_mut(ThemeComponentType::RibbonSelected).background = RgbColor::new(1, 2, 3);
        theme.get_mut(ThemeComponentType::TextSelected).base = RgbColor::new(9, 8, 7);

        let saved = theme_to_kdl(&theme);
        let loaded = parse_theme_kdl(&saved, &theme.name).expect("saved theme should parse");

        assert_eq!(
            loaded.get(ThemeComponentType::RibbonSelected).background,
            RgbColor::new(1, 2, 3)
        );
        assert_eq!(
            loaded.get(ThemeComponentType::TextSelected).base,
            RgbColor::new(9, 8, 7)
        );
    }
}
