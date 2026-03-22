use crate::theme::{RgbColor, Theme, ThemeComponent, ThemeComponentType};

pub struct App {
    pub theme: Theme,
    pub selected_element: PreviewElement,
    pub selected_attribute: PreviewAttribute,
    pub config_manager: crate::config::ConfigManager,
    pub message: Option<String>,
    pub input_mode: InputMode,
    pub color_editor: ColorEditor,
    pub original_component: Option<ThemeComponent>,
    pub theme_name_input: String,
    pub loadable_themes: Vec<String>,
    pub selected_theme_index: usize,
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Preview,
    ColorPicker,
    ThemeNameInput,
    ThemeNameInputApply,
    ThemeLoad,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewAttribute {
    Base,
    Background,
}

impl PreviewAttribute {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Base => "FG",
            Self::Background => "BG",
        }
    }

    pub fn cycle(&mut self) {
        *self = match self {
            Self::Base => Self::Background,
            Self::Background => Self::Base,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewElement {
    // Tab bar
    TabSelected,
    TabUnselected1,
    TabUnselected2,
    // Status bar
    StatusBar,
    // Left panes
    PaneSelected,
    TextSelected,
    PaneUnselected,
    // Right pane (highlight) — frame + contents
    PaneHighlight,
    TableTitle,
    TableCellSelected,
    TableCellUnselected,
    ListSelected,
    ListUnselected,
    ExitSuccess,
    ExitError,
}

impl PreviewElement {
    pub fn is_frame(&self) -> bool {
        matches!(self, Self::PaneSelected | Self::PaneUnselected | Self::PaneHighlight)
    }

    pub fn component_type(&self) -> ThemeComponentType {
        match self {
            Self::TabSelected => ThemeComponentType::RibbonSelected,
            Self::TabUnselected1 | Self::TabUnselected2 => ThemeComponentType::RibbonUnselected,
            Self::StatusBar => ThemeComponentType::TextUnselected,
            Self::PaneSelected => ThemeComponentType::FrameSelected,
            Self::TextSelected => ThemeComponentType::TextSelected,
            Self::PaneHighlight => ThemeComponentType::FrameHighlight,
            Self::PaneUnselected => ThemeComponentType::FrameUnselected,
            Self::TableTitle => ThemeComponentType::TableTitle,
            Self::TableCellSelected => ThemeComponentType::TableCellSelected,
            Self::TableCellUnselected => ThemeComponentType::TableCellUnselected,
            Self::ListSelected => ThemeComponentType::ListSelected,
            Self::ListUnselected => ThemeComponentType::ListUnselected,
            Self::ExitSuccess => ThemeComponentType::ExitCodeSuccess,
            Self::ExitError => ThemeComponentType::ExitCodeError,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::TabSelected => "Tab (Selected)",
            Self::TabUnselected1 | Self::TabUnselected2 => "Tab (Unselected)",
            Self::StatusBar => "Status Bar",
            Self::PaneSelected => "Pane (Selected)",
            Self::TextSelected => "Text (Selected)",
            Self::PaneHighlight => "Pane (Highlight)",
            Self::PaneUnselected => "Pane (Unselected)",
            Self::TableTitle => "Table Title",
            Self::TableCellSelected => "Table Cell (Selected)",
            Self::TableCellUnselected => "Table Cell (Unselected)",
            Self::ListSelected => "List (Selected)",
            Self::ListUnselected => "List (Unselected)",
            Self::ExitSuccess => "Exit (Success)",
            Self::ExitError => "Exit (Error)",
        }
    }

    /// Full vertical order (top → bottom of screen)
    fn vertical_order() -> &'static [PreviewElement] {
        use PreviewElement::*;
        &[
            TabSelected,
            TabUnselected1,
            TabUnselected2,
            PaneSelected,
            TextSelected,
            PaneUnselected,
            PaneHighlight,
            TableTitle,
            TableCellSelected,
            TableCellUnselected,
            ListSelected,
            ListUnselected,
            ExitSuccess,
            ExitError,
            StatusBar,
        ]
    }

    fn vertical_index(&self) -> usize {
        Self::vertical_order()
            .iter()
            .position(|e| e == self)
            .unwrap_or(0)
    }

    pub fn move_up(&mut self) {
        let order = Self::vertical_order();
        let idx = self.vertical_index();
        let next = if idx == 0 { order.len() - 1 } else { idx - 1 };
        *self = order[next];
    }

    pub fn move_down(&mut self) {
        let order = Self::vertical_order();
        let idx = self.vertical_index();
        let next = (idx + 1) % order.len();
        *self = order[next];
    }

    pub fn move_left(&mut self) {
        use PreviewElement::*;
        *self = match self {
            // Tabs: move left among tabs
            TabSelected => TabSelected,
            TabUnselected1 => TabSelected,
            TabUnselected2 => TabUnselected1,
            // Status bar stays
            StatusBar => StatusBar,
            // Left panes stay in left column
            PaneSelected => PaneSelected,
            TextSelected => TextSelected,
            PaneUnselected => PaneUnselected,
            // Right pane contents → jump to left column
            PaneHighlight => PaneSelected,
            TableTitle | TableCellSelected | TableCellUnselected | ListSelected
            | ListUnselected | ExitSuccess => PaneSelected,
            ExitError => PaneSelected,
        };
    }

    pub fn move_right(&mut self) {
        use PreviewElement::*;
        *self = match self {
            // Tabs: move right among tabs
            TabSelected => TabUnselected1,
            TabUnselected1 => TabUnselected2,
            TabUnselected2 => TabUnselected2,
            // Status bar stays
            StatusBar => StatusBar,
            // Left panes → right pane
            PaneSelected | TextSelected | PaneUnselected => PaneHighlight,
            // Right column stays
            PaneHighlight | TableTitle | TableCellSelected | TableCellUnselected | ListSelected
            | ListUnselected => *self,
            ExitSuccess => ExitError,
            ExitError => ExitError,
        };
    }
}

pub struct ColorEditor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub editing_channel: usize,
    pub hex_input: Option<String>,
}

impl ColorEditor {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            editing_channel: 0,
            hex_input: None,
        }
    }

    pub fn start_hex_input(&mut self) {
        self.hex_input = Some(format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b));
    }

    pub fn push_hex_char(&mut self, c: char) {
        if let Some(ref mut s) = self.hex_input {
            if s.len() < 6 && c.is_ascii_hexdigit() {
                s.push(c.to_ascii_lowercase());
            }
        }
    }

    pub fn pop_hex_char(&mut self) -> bool {
        if let Some(ref mut s) = self.hex_input {
            s.pop();
            return true;
        }
        false
    }

    pub fn commit_hex(&mut self) -> Option<RgbColor> {
        if let Some(ref s) = self.hex_input {
            if s.len() == 6 {
                let result = crate::theme::RgbColor::from_hex(s);
                self.hex_input = None;
                if let Some(c) = result {
                    self.r = c.r;
                    self.g = c.g;
                    self.b = c.b;
                    return Some(c);
                }
            }
        }
        self.hex_input = None;
        None
    }

    pub fn cancel_hex(&mut self) {
        self.hex_input = None;
    }

    pub fn to_rgb(&self) -> RgbColor {
        RgbColor::new(self.r, self.g, self.b)
    }

    pub fn adjust(&mut self, delta: i32) {
        match self.editing_channel {
            0 => self.r = (self.r as i32 + delta).clamp(0, 255) as u8,
            1 => self.g = (self.g as i32 + delta).clamp(0, 255) as u8,
            2 => self.b = (self.b as i32 + delta).clamp(0, 255) as u8,
            _ => {}
        }
    }

    pub fn select_next_channel(&mut self) {
        self.editing_channel = (self.editing_channel + 1) % 3;
    }

    pub fn select_prev_channel(&mut self) {
        self.editing_channel = if self.editing_channel == 0 {
            2
        } else {
            self.editing_channel - 1
        };
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            selected_element: PreviewElement::TabSelected,
            selected_attribute: PreviewAttribute::Base,
            config_manager: crate::config::ConfigManager::new(),
            message: None,
            input_mode: InputMode::Preview,
            color_editor: ColorEditor::from_rgb(200, 200, 200),
            original_component: None,
            theme_name_input: String::from("default"),
            loadable_themes: Vec::new(),
            selected_theme_index: 0,
            dirty: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self::default();
        app.sync_theme_name_input();
        app
    }

    pub fn save_theme(&mut self) {
        match self.config_manager.save_theme(&self.theme) {
            Ok(()) => {
                self.message = Some(format!("✓ Saved: {}", self.theme.name));
                self.dirty = false;
            }
            Err(e) => {
                self.message = Some(format!("✗ Error: {}", e));
            }
        }
    }

    pub fn open_theme_name_input(&mut self) {
        self.sync_theme_name_input();
        self.input_mode = InputMode::ThemeNameInput;
        self.message = Some(String::from("Enter a theme name, then press Enter to save"));
    }

    pub fn save_theme_as_input_name(&mut self) {
        let normalized_name = normalize_theme_name(&self.theme_name_input);
        if normalized_name.is_empty() {
            self.message = Some(String::from("✗ Theme name must contain letters or numbers"));
            return;
        }

        self.theme.name = normalized_name;
        self.sync_theme_name_input();
        self.save_theme();
        self.refresh_theme_list();
        self.input_mode = InputMode::Preview;
    }

    pub fn save_and_apply_theme_as_input_name(&mut self) {
        let normalized_name = normalize_theme_name(&self.theme_name_input);
        if normalized_name.is_empty() {
            self.message = Some(String::from("✗ Theme name must contain letters or numbers"));
            return;
        }

        self.theme.name = normalized_name;
        self.sync_theme_name_input();
        self.apply_theme_to_zellij();
        self.refresh_theme_list();
        self.input_mode = InputMode::Preview;
    }

    pub fn open_theme_load_dialog(&mut self) {
        self.refresh_theme_list();
        if self.loadable_themes.is_empty() {
            self.message = Some(String::from(
                "✗ No saved themes found in ~/.config/zellij/themes",
            ));
            return;
        }

        self.selected_theme_index = self
            .loadable_themes
            .iter()
            .position(|name| name == &self.theme.name)
            .unwrap_or(0);
        self.input_mode = InputMode::ThemeLoad;
        self.message = Some(String::from("Select a theme to load"));
    }

    pub fn load_selected_theme(&mut self) {
        let Some(name) = self.loadable_themes.get(self.selected_theme_index).cloned() else {
            self.message = Some(String::from("✗ No theme selected"));
            self.input_mode = InputMode::Preview;
            return;
        };

        match self.config_manager.load_theme(&name) {
            Ok(theme) => {
                self.theme = theme;
                self.sync_theme_name_input();
                self.message = Some(format!("✓ Loaded: {}", name));
                self.dirty = false;
            }
            Err(e) => {
                self.message = Some(format!("✗ Error: {}", e));
            }
        }
        self.input_mode = InputMode::Preview;
    }

    pub fn apply_theme_to_zellij(&mut self) {
        match self.config_manager.apply_theme_to_zellij(&self.theme) {
            Ok(()) => {
                self.message = Some(format!(
                    "✓ Saved + applied \"{}\" — restart Zellij to see changes",
                    self.theme.name
                ));
            }
            Err(e) => {
                self.message = Some(format!("✗ Error: {}", e));
            }
        }
    }

    pub fn refresh_theme_list(&mut self) {
        match self.config_manager.list_themes() {
            Ok(mut themes) => {
                themes.sort();
                self.loadable_themes = themes;
                if self.selected_theme_index >= self.loadable_themes.len() {
                    self.selected_theme_index = self.loadable_themes.len().saturating_sub(1);
                }
            }
            Err(e) => {
                self.loadable_themes.clear();
                self.selected_theme_index = 0;
                self.message = Some(format!("✗ Error: {}", e));
            }
        }
    }

    pub fn move_theme_selection_up(&mut self) {
        if self.loadable_themes.is_empty() {
            self.selected_theme_index = 0;
        } else if self.selected_theme_index == 0 {
            self.selected_theme_index = self.loadable_themes.len() - 1;
        } else {
            self.selected_theme_index -= 1;
        }
    }

    pub fn move_theme_selection_down(&mut self) {
        if self.loadable_themes.is_empty() {
            self.selected_theme_index = 0;
        } else {
            self.selected_theme_index =
                (self.selected_theme_index + 1) % self.loadable_themes.len();
        }
    }

    pub fn push_theme_name_char(&mut self, c: char) {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' {
            self.theme_name_input.push(c);
        }
    }

    pub fn pop_theme_name_char(&mut self) {
        self.theme_name_input.pop();
    }

    pub fn sync_theme_name_input(&mut self) {
        self.theme_name_input = self.theme.name.clone();
    }

    pub fn get_color_by_attr(&self, attr: PreviewAttribute) -> RgbColor {
        let component = self.theme.get(self.selected_element.component_type());
        match attr {
            PreviewAttribute::Base => component.base,
            PreviewAttribute::Background => component.background,
        }
    }

    pub fn get_color(&self) -> RgbColor {
        self.get_color_by_attr(self.selected_attribute)
    }

    pub fn set_color_by_attr(&mut self, attr: PreviewAttribute, color: RgbColor) {
        let comp_type = self.selected_element.component_type();
        let component = self.theme.get_mut(comp_type);
        match attr {
            PreviewAttribute::Base => component.base = color,
            PreviewAttribute::Background => component.background = color,
        }
        self.dirty = true;
    }

    pub fn apply_current_color(&mut self) {
        let color = self.color_editor.to_rgb();
        let attr = self.selected_attribute;
        self.set_color_by_attr(attr, color);
    }

    pub fn open_color_picker(&mut self) {
        let comp_type = self.selected_element.component_type();
        self.original_component = Some(self.theme.get(comp_type).clone());
        let color = self.get_color();
        self.color_editor = ColorEditor::from_rgb(color.r, color.g, color.b);
        self.input_mode = InputMode::ColorPicker;
    }

    pub fn close_color_picker(&mut self, save: bool) {
        if !save {
            if let Some(original) = self.original_component.take() {
                let comp_type = self.selected_element.component_type();
                let component = self.theme.get_mut(comp_type);
                *component = original;
            }
        }
        self.original_component = None;
        self.input_mode = InputMode::Preview;
    }

    pub fn switch_editing_attribute(&mut self) {
        self.selected_attribute.cycle();
        let attr = self.selected_attribute;
        let color = self.get_color_by_attr(attr);
        self.color_editor = ColorEditor::from_rgb(color.r, color.g, color.b);
    }
}

pub fn normalize_theme_name(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                Some(c.to_ascii_lowercase())
            } else if c.is_ascii_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
