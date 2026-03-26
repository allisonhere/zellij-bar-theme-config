use crate::theme::{RgbColor, Theme, ThemeComponent, ThemeComponentType};
use crate::update::UpdateMsg;
use crate::ui::color_picker::ColorEditor;

#[derive(Debug, Clone)]
pub enum ThemeEntry {
    User(String),
    Builtin(&'static str),
}

impl ThemeEntry {
    pub fn name(&self) -> &str {
        match self {
            Self::User(n) => n.as_str(),
            Self::Builtin(n) => n,
        }
    }
    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::Builtin(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeFilter {
    All,
    Builtin,
    Saved,
}

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
    pub all_themes: Vec<ThemeEntry>,
    pub loadable_themes: Vec<ThemeEntry>,
    pub theme_filter: ThemeFilter,
    pub selected_theme_index: usize,
    pub dirty: bool,
    pub original_theme: Option<crate::theme::Theme>,
    pub theme_swatches: std::collections::HashMap<String, [crate::theme::RgbColor; 4]>,
    pub theme_search_query: String,
    pub search_focused: bool,
    pub help_scroll: u16,
    // Feature 3: copy/paste color
    pub clipboard_color: Option<crate::theme::RgbColor>,
    // Feature 4: undo
    pub undo_history: std::collections::HashMap<(PreviewElement, PreviewAttribute), crate::theme::RgbColor>,
    // Feature 5: rename/delete
    pub loader_action_index: usize,
    // Self-update
    pub update_status: UpdateStatus,
    pub update_rx: Option<std::sync::mpsc::Receiver<UpdateMsg>>,
    pub restart_after_exit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Preview,
    ColorPicker,
    ThemeNameInput,
    ThemeNameInputApply,
    ThemeLoad,
    ThemeLoadRename,
    ThemeLoadDeleteConfirm,
    UpdateRestartConfirm,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available(String),
    Downloading,
    Done,
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

fn theme_swatches(theme: &crate::theme::Theme) -> [crate::theme::RgbColor; 4] {
    [
        theme.get(crate::theme::ThemeComponentType::TextUnselected).background,
        theme.get(crate::theme::ThemeComponentType::RibbonSelected).background,
        theme.get(crate::theme::ThemeComponentType::FrameSelected).base,
        theme.get(crate::theme::ThemeComponentType::FrameHighlight).base,
    ]
}

fn load_entry(entry: &ThemeEntry, config_manager: &crate::config::ConfigManager) -> Result<crate::theme::Theme, crate::config::ConfigError> {
    match entry {
        ThemeEntry::User(n) => config_manager.load_theme(n),
        ThemeEntry::Builtin(n) => {
            let kdl = crate::bundled_themes::THEMES
                .iter()
                .find(|(k, _)| k == n)
                .map(|(_, v)| *v)
                .unwrap_or("");
            crate::config::parse_theme_kdl(kdl, n)
        }
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
            all_themes: Vec::new(),
            loadable_themes: Vec::new(),
            theme_filter: ThemeFilter::All,
            selected_theme_index: 0,
            dirty: false,
            original_theme: None,
            theme_swatches: std::collections::HashMap::new(),
            theme_search_query: String::new(),
            search_focused: false,
            help_scroll: 0,
            clipboard_color: None,
            undo_history: std::collections::HashMap::new(),
            loader_action_index: 0,
            update_status: UpdateStatus::Idle,
            update_rx: None,
            restart_after_exit: false,
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
        self.theme_search_query = String::new();
        self.search_focused = false;
        self.theme_filter = ThemeFilter::All;
        self.original_theme = Some(self.theme.clone());
        self.refresh_theme_list();
        self.selected_theme_index = self
            .loadable_themes
            .iter()
            .position(|e| e.name() == self.theme.name)
            .unwrap_or(0);
        self.theme_swatches = self.all_themes.iter().map(|entry| {
            let sw = match load_entry(entry, &self.config_manager) {
                Ok(t) => theme_swatches(&t),
                Err(_) => [crate::theme::RgbColor::new(50, 50, 50); 4],
            };
            (entry.name().to_string(), sw)
        }).collect();
        // Live-preview the initially selected theme
        if let Some(entry) = self.loadable_themes.get(self.selected_theme_index).cloned() {
            if let Ok(t) = load_entry(&entry, &self.config_manager) {
                self.theme = t;
            }
        }
        self.input_mode = InputMode::ThemeLoad;
        self.message = Some(String::from("Select a theme to load"));
    }

    pub fn load_selected_theme(&mut self) {
        if let Some(entry) = self.loadable_themes.get(self.selected_theme_index).cloned() {
            let name = entry.name().to_string();
            match load_entry(&entry, &self.config_manager) {
                Ok(t) => { self.theme = t; }
                Err(e) => {
                    self.message = Some(format!("✗ Error loading \"{}\": {}", name, e));
                    return;
                }
            }
            self.sync_theme_name_input();
            self.original_theme = None;
            self.dirty = false;
            self.message = Some(format!("✓ Loaded: {}", name));
            self.input_mode = InputMode::Preview;
        }
    }

    pub fn apply_selected_theme(&mut self) {
        // theme already loaded into self.theme from scrolling
        let name = self.loadable_themes
            .get(self.selected_theme_index)
            .map(|e| e.name().to_string())
            .unwrap_or_default();
        self.sync_theme_name_input();
        self.original_theme = None;
        self.dirty = false;
        self.input_mode = InputMode::Preview;
        match self.config_manager.apply_theme_to_zellij(&self.theme) {
            Ok(()) => self.message = Some(format!("✓ Applied \"{}\" — restart Zellij to see changes", name)),
            Err(e) => self.message = Some(format!("✗ Error: {}", e)),
        }
    }

    pub fn cancel_theme_load(&mut self) {
        if let Some(original) = self.original_theme.take() {
            self.theme = original;
        }
        self.sync_theme_name_input();
        self.theme_search_query = String::new();
        self.search_focused = false;
        self.input_mode = InputMode::Preview;
        self.message = None;
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
        let user_themes: Vec<ThemeEntry> = match self.config_manager.list_themes() {
            Ok(mut names) => {
                names.sort();
                names.into_iter().map(ThemeEntry::User).collect()
            }
            Err(e) => {
                self.message = Some(format!("✗ Error listing themes: {}", e));
                Vec::new()
            }
        };

        let builtin_themes: Vec<ThemeEntry> = crate::bundled_themes::THEMES
            .iter()
            .map(|(name, _)| ThemeEntry::Builtin(name))
            .collect();

        self.all_themes = user_themes.into_iter().chain(builtin_themes).collect();
        self.apply_filter_to_list();
    }

    pub fn apply_filter_to_list(&mut self) {
        let q = self.theme_search_query.trim_start_matches('/').to_ascii_lowercase();
        self.loadable_themes = self.all_themes.iter().filter(|e| {
            let matches_filter = match self.theme_filter {
                ThemeFilter::All => true,
                ThemeFilter::Builtin => e.is_builtin(),
                ThemeFilter::Saved => !e.is_builtin(),
            };
            let matches_search = q.is_empty() || fuzzy_match(&e.name().to_ascii_lowercase(), &q);
            matches_filter && matches_search
        }).cloned().collect();

        if self.selected_theme_index >= self.loadable_themes.len() {
            self.selected_theme_index = self.loadable_themes.len().saturating_sub(1);
        }
    }

    pub fn set_theme_filter(&mut self, filter: ThemeFilter) {
        // Toggle off if same filter pressed again
        self.theme_filter = if self.theme_filter == filter { ThemeFilter::All } else { filter };
        self.apply_filter_to_list();
        self.selected_theme_index = 0;
        // Swatches are keyed by name; no recompute needed after filter changes
        // Live-preview the newly selected theme
        if let Some(entry) = self.loadable_themes.get(0).cloned() {
            if let Ok(t) = load_entry(&entry, &self.config_manager) {
                self.theme = t;
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
        if let Some(entry) = self.loadable_themes.get(self.selected_theme_index).cloned() {
            if let Ok(t) = load_entry(&entry, &self.config_manager) {
                self.theme = t;
            }
        }
    }

    pub fn move_theme_selection_down(&mut self) {
        if self.loadable_themes.is_empty() {
            self.selected_theme_index = 0;
        } else {
            self.selected_theme_index =
                (self.selected_theme_index + 1) % self.loadable_themes.len();
        }
        if let Some(entry) = self.loadable_themes.get(self.selected_theme_index).cloned() {
            if let Ok(t) = load_entry(&entry, &self.config_manager) {
                self.theme = t;
            }
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
        let previous_mode = self.color_editor.mode;
        self.color_editor = ColorEditor::from_rgb(color.r, color.g, color.b);
        if self.color_editor.mode != previous_mode {
            self.color_editor.toggle_mode();
        }
        self.input_mode = InputMode::ColorPicker;
    }

    pub fn close_color_picker(&mut self, save: bool) {
        if save {
            self.record_undo();
        } else {
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
        let previous_mode = self.color_editor.mode;
        self.color_editor = ColorEditor::from_rgb(color.r, color.g, color.b);
        if self.color_editor.mode != previous_mode {
            self.color_editor.toggle_mode();
        }
    }

    // Feature 2: move selection to index and live-preview
    pub fn move_theme_selection_to(&mut self, index: usize) {
        self.selected_theme_index = index;
        if let Some(entry) = self.loadable_themes.get(self.selected_theme_index).cloned() {
            if let Ok(t) = load_entry(&entry, &self.config_manager) {
                self.theme = t;
            }
        }
    }

    // Feature 3: yank/paste color
    pub fn yank_color(&mut self) {
        let c = self.get_color_by_attr(self.selected_attribute);
        self.clipboard_color = Some(c);
        self.message = Some(format!("Yanked #{:02x}{:02x}{:02x}", c.r, c.g, c.b));
    }

    pub fn paste_color(&mut self) {
        if let Some(c) = self.clipboard_color {
            self.set_color_by_attr(self.selected_attribute, c);
            self.dirty = true;
            self.message = Some(format!("Pasted #{:02x}{:02x}{:02x}", c.r, c.g, c.b));
        } else {
            self.message = Some(String::from("Nothing to yank"));
        }
    }

    // Feature 4: undo
    pub fn record_undo(&mut self) {
        if let Some(ref orig) = self.original_component {
            let before = match self.selected_attribute {
                PreviewAttribute::Base => orig.base,
                PreviewAttribute::Background => orig.background,
            };
            self.undo_history.insert((self.selected_element, self.selected_attribute), before);
        }
    }

    pub fn undo_color(&mut self) {
        let key = (self.selected_element, self.selected_attribute);
        if let Some(prev) = self.undo_history.remove(&key) {
            self.set_color_by_attr(self.selected_attribute, prev);
            self.dirty = true;
            self.message = Some(String::from("Undone"));
        } else {
            self.message = Some(String::from("Nothing to undo"));
        }
    }

    // Feature 5: rename/delete saved themes
    pub fn begin_rename_selected_theme(&mut self) {
        if self.loadable_themes.get(self.selected_theme_index).map(|e| e.is_builtin()).unwrap_or(true) {
            self.message = Some(String::from("Cannot rename built-in themes"));
            return;
        }
        self.loader_action_index = self.selected_theme_index;
        self.theme_name_input = self.loadable_themes[self.selected_theme_index].name().to_string();
        self.input_mode = InputMode::ThemeLoadRename;
    }

    pub fn commit_rename_theme(&mut self) {
        let new_name = normalize_theme_name(&self.theme_name_input);
        if new_name.is_empty() {
            self.message = Some(String::from("✗ Invalid name"));
            return;
        }
        let old_name = self.loadable_themes.get(self.loader_action_index).map(|e| e.name().to_string()).unwrap_or_default();
        match self.config_manager.rename_theme(&old_name, &new_name) {
            Ok(()) => self.message = Some(format!("✓ Renamed to {}", new_name)),
            Err(e) => self.message = Some(format!("✗ {}", e)),
        }
        self.refresh_theme_list();
        self.input_mode = InputMode::ThemeLoad;
    }

    pub fn begin_delete_selected_theme(&mut self) {
        if self.loadable_themes.get(self.selected_theme_index).map(|e| e.is_builtin()).unwrap_or(true) {
            self.message = Some(String::from("Cannot delete built-in themes"));
            return;
        }
        self.loader_action_index = self.selected_theme_index;
        let name = self.loadable_themes[self.selected_theme_index].name().to_string();
        self.message = Some(format!("Delete \"{}\"? y = confirm, n = cancel", name));
        self.input_mode = InputMode::ThemeLoadDeleteConfirm;
    }

    pub fn confirm_delete_theme(&mut self) {
        let name = self.loadable_themes.get(self.loader_action_index).map(|e| e.name().to_string()).unwrap_or_default();
        match self.config_manager.delete_theme(&name) {
            Ok(()) => self.message = Some(format!("✓ Deleted \"{}\"", name)),
            Err(e) => self.message = Some(format!("✗ {}", e)),
        }
        self.refresh_theme_list();
        self.selected_theme_index = self.selected_theme_index.min(self.loadable_themes.len().saturating_sub(1));
        self.input_mode = InputMode::ThemeLoad;
    }

    // Self-update
    pub fn start_update_check(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.update_status = UpdateStatus::Checking;
        self.update_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(UpdateMsg::VersionChecked(crate::update::check_version()));
        });
    }

    pub fn start_self_update(&mut self) {
        match &self.update_status {
            UpdateStatus::Available(tag) => {
                let tag = tag.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                self.update_status = UpdateStatus::Downloading;
                self.update_rx = Some(rx);
                self.message = None;
                std::thread::spawn(move || {
                    let _ = tx.send(UpdateMsg::UpdateComplete(crate::update::download_and_replace(&tag)));
                });
            }
            UpdateStatus::Checking => {
                self.message = Some(String::from("Still checking for updates"));
            }
            UpdateStatus::UpToDate | UpdateStatus::Idle => {
                self.message = Some(String::from("No update available"));
            }
            UpdateStatus::Downloading => {
                self.message = Some(String::from("Update already in progress"));
            }
            UpdateStatus::Done => {
                self.message = Some(String::from("Update already installed; restart to apply"));
            }
            UpdateStatus::Failed(err) => {
                self.message = Some(format!("✗ Update unavailable: {}", err));
            }
        }
    }

    pub fn poll_update_channel(&mut self) {
        let Some(rx) = self.update_rx.as_ref() else {
            return;
        };

        match rx.try_recv() {
            Ok(msg) => {
                self.update_rx = None;
                self.message = None;
                match msg {
                    UpdateMsg::VersionChecked(Ok(Some(tag))) => {
                        self.update_status = UpdateStatus::Available(tag);
                    }
                    UpdateMsg::VersionChecked(Ok(None)) => {
                        self.update_status = UpdateStatus::UpToDate;
                    }
                    UpdateMsg::VersionChecked(Err(e)) => {
                        self.update_status = UpdateStatus::Failed(e);
                    }
                    UpdateMsg::UpdateComplete(Ok(())) => {
                        self.update_status = UpdateStatus::Done;
                        self.input_mode = InputMode::UpdateRestartConfirm;
                    }
                    UpdateMsg::UpdateComplete(Err(e)) => {
                        self.update_status = UpdateStatus::Failed(e);
                    }
                };
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.update_rx = None;
                self.message = None;
                self.update_status = UpdateStatus::Failed(String::from("update worker disconnected"));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
    }

    pub fn defer_restart(&mut self) {
        self.input_mode = InputMode::Preview;
        self.update_status = UpdateStatus::Idle;
        self.message = None;
    }

    pub fn confirm_restart(&mut self) {
        self.restart_after_exit = true;
        self.input_mode = InputMode::Preview;
    }
}

fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut hay_chars = haystack.chars();
    for n in needle.chars() {
        if !hay_chars.any(|h| h == n) {
            return false;
        }
    }
    true
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
