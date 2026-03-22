use crate::theme::{RgbColor, Theme, ThemeComponent, ThemeComponentType};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph},
    Frame,
};

pub struct App {
    pub theme: Theme,
    pub selected_element: PreviewElement,
    pub selected_attribute: PreviewAttribute,
    pub editing_attribute: PreviewAttribute,
    pub config_manager: crate::config::ConfigManager,
    pub message: Option<String>,
    pub input_mode: InputMode,
    pub color_editor: ColorEditor,
    pub original_component: Option<ThemeComponent>,
    pub theme_name_input: String,
    pub loadable_themes: Vec<String>,
    pub selected_theme_index: usize,
    pub dirty: bool,
    pub pending_apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Preview,
    ColorPicker,
    ThemeNameInput,
    ThemeLoad,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewAttribute {
    Foreground,
    Background,
}

impl PreviewAttribute {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Foreground => "FG",
            Self::Background => "BG",
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Self::Foreground => Self::Background,
            Self::Background => Self::Foreground,
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
            selected_attribute: PreviewAttribute::Foreground,
            editing_attribute: PreviewAttribute::Foreground,
            config_manager: crate::config::ConfigManager::new(),
            message: None,
            input_mode: InputMode::Preview,
            color_editor: ColorEditor::from_rgb(200, 200, 200),
            original_component: None,
            theme_name_input: String::from("default"),
            loadable_themes: Vec::new(),
            selected_theme_index: 0,
            dirty: false,
            pending_apply: false,
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
        if self.pending_apply {
            self.pending_apply = false;
            self.apply_theme_to_zellij();
        } else {
            self.save_theme();
        }
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
        let component = get_component_by_type(&self.theme, self.selected_element.component_type());
        match attr {
            PreviewAttribute::Foreground => component.base,
            PreviewAttribute::Background => component.background,
        }
    }

    pub fn get_color(&self) -> RgbColor {
        self.get_color_by_attr(self.editing_attribute)
    }

    pub fn set_color_by_attr(&mut self, attr: PreviewAttribute, color: RgbColor) {
        let comp_type = self.selected_element.component_type();
        let component = get_component_mut_by_type(&mut self.theme, comp_type);
        match attr {
            PreviewAttribute::Foreground => component.base = color,
            PreviewAttribute::Background => component.background = color,
        }
        self.dirty = true;
    }

    pub fn apply_current_color(&mut self) {
        let color = self.color_editor.to_rgb();
        let attr = self.editing_attribute;
        self.set_color_by_attr(attr, color);
    }

    pub fn open_color_picker(&mut self) {
        self.editing_attribute = self.selected_attribute;
        let comp_type = self.selected_element.component_type();
        self.original_component = Some(get_component_by_type(&self.theme, comp_type).clone());
        let color = self.get_color();
        self.color_editor = ColorEditor::from_rgb(color.r, color.g, color.b);
        self.input_mode = InputMode::ColorPicker;
    }

    pub fn close_color_picker(&mut self, save: bool) {
        if !save {
            if let Some(original) = self.original_component.take() {
                let comp_type = self.selected_element.component_type();
                let component = get_component_mut_by_type(&mut self.theme, comp_type);
                *component = original;
            }
        }
        self.original_component = None;
        self.input_mode = InputMode::Preview;
    }

    pub fn switch_editing_attribute(&mut self) {
        self.editing_attribute.toggle();
        let attr = self.editing_attribute;
        let color = self.get_color_by_attr(attr);
        self.color_editor = ColorEditor::from_rgb(color.r, color.g, color.b);
    }

    pub fn render(&self, frame: &mut Frame) {
        match self.input_mode {
            InputMode::Preview => self.render_preview(frame),
            InputMode::ColorPicker => self.render_color_picker_mode(frame),
            InputMode::ThemeNameInput => self.render_theme_name_input_mode(frame),
            InputMode::ThemeLoad => self.render_theme_load_mode(frame),
            InputMode::Help => self.render_help_mode(frame),
        }
    }

    // ── Preview layout (full Zellij screen) ──────────────────────────────

    fn render_preview(&self, frame: &mut Frame) {
        let area = frame.area();
        let [tab_bar, main, status_bar] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_zellij_tab_bar(frame, tab_bar);
        self.render_zellij_panes(frame, main);
        self.render_zellij_status_bar(frame, status_bar);
    }

    fn render_zellij_tab_bar(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let sel_fg = get_fg(ThemeComponentType::RibbonSelected, t);
        let sel_bg = get_bg(ThemeComponentType::RibbonSelected, t);
        let unsel_fg = get_fg(ThemeComponentType::RibbonUnselected, t);
        let unsel_bg = get_bg(ThemeComponentType::RibbonUnselected, t);
        let bar_bg = get_bg(ThemeComponentType::TextUnselected, t);
        let bar_fg = get_fg(ThemeComponentType::TextUnselected, t);

        let is_tab_sel = self.selected_element == PreviewElement::TabSelected;
        let is_tab_u1 = self.selected_element == PreviewElement::TabUnselected1;
        let is_tab_u2 = self.selected_element == PreviewElement::TabUnselected2;

        let tab_sel_fg = sel_fg;
        let tab_u1_fg = unsel_fg;
        let tab_u2_fg = unsel_fg;
        // Selection: lighten the tab's background instead of overriding fg with yellow
        let tab_sel_bg = if is_tab_sel {
            lighten(sel_bg, 30)
        } else {
            sel_bg
        };
        let tab_u1_bg = if is_tab_u1 {
            lighten(unsel_bg, 30)
        } else {
            unsel_bg
        };
        let tab_u2_bg = if is_tab_u2 {
            lighten(unsel_bg, 30)
        } else {
            unsel_bg
        };

        let layout_label = " layout: default ";

        // Session name (plain, no pill shape — matches real Zellij)
        let mut spans: Vec<Span> = vec![Span::styled(
            " zellij ",
            Style::new()
                .fg(bar_fg)
                .bg(bar_bg)
                .add_modifier(Modifier::BOLD),
        )];

        // Selected tab pill — bg lightened if this tab is the focused element
        spans.push(Span::styled("", Style::new().fg(tab_sel_bg).bg(bar_bg)));
        spans.push(Span::styled(
            " 1:terminal ",
            Style::new()
                .fg(tab_sel_fg)
                .bg(tab_sel_bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("", Style::new().fg(tab_sel_bg).bg(unsel_bg)));

        // Unselected tab 2 pill — bg lightened if focused
        spans.push(Span::styled("", Style::new().fg(tab_u1_bg).bg(bar_bg)));
        spans.push(Span::styled(
            " 2:bash ",
            Style::new()
                .fg(tab_u1_fg)
                .bg(tab_u1_bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("", Style::new().fg(tab_u1_bg).bg(bar_bg)));
        spans.push(Span::styled(" ", Style::new().bg(bar_bg)));

        // Unselected tab 3 pill — bg lightened if focused
        spans.push(Span::styled("", Style::new().fg(tab_u2_bg).bg(bar_bg)));
        spans.push(Span::styled(
            " 3:nvim ",
            Style::new()
                .fg(tab_u2_fg)
                .bg(tab_u2_bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("", Style::new().fg(tab_u2_bg).bg(bar_bg)));

        let used: usize = spans.iter().map(|s| s.width()).sum::<usize>() + layout_label.len();
        let fill = (area.width as usize).saturating_sub(used);
        spans.push(Span::styled(" ".repeat(fill), Style::new().bg(bar_bg)));
        spans.push(Span::styled(
            layout_label,
            Style::new().fg(bar_fg).bg(bar_bg),
        ));

        frame.render_widget(
            Paragraph::new(vec![Line::from(spans)]).style(Style::new().bg(bar_bg)),
            area,
        );
    }

    fn render_zellij_panes(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let bg = get_bg(ThemeComponentType::TextUnselected, t);
        let text_fg = get_fg(ThemeComponentType::TextUnselected, t);

        frame.render_widget(Paragraph::new("").style(Style::new().bg(bg)), area);

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(area);

        let [top_left, bottom_left] =
            Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(left);

        self.render_pane_selected(frame, top_left, bg, text_fg);
        self.render_pane_unselected(frame, bottom_left, bg, text_fg);
        self.render_pane_highlight(frame, right, bg, text_fg);
    }

    fn render_pane_selected(&self, frame: &mut Frame, area: Rect, bg: Color, text_fg: Color) {
        let t = &self.theme;
        let is_editing_border = self.selected_element == PreviewElement::PaneSelected;
        let is_editing_text = self.selected_element == PreviewElement::TextSelected;

        // Lighten the whole pane bg when this pane frame is focused
        let pane_bg = if is_editing_border {
            lighten(bg, 25)
        } else {
            bg
        };

        let border_color = get_fg(ThemeComponentType::FrameSelected, t);
        let border_style = Style::new()
            .fg(border_color)
            .add_modifier(if is_editing_border {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });

        let text_sel_fg = get_fg(ThemeComponentType::TextSelected, t);
        let text_sel_bg = get_bg(ThemeComponentType::TextSelected, t);
        // Lighten text-selected line when TextSelected element is focused
        let tsel_bg = if is_editing_text {
            lighten(text_sel_bg, 45)
        } else {
            text_sel_bg
        };

        let content = vec![
            Line::from(Span::styled("$ ls", Style::new().fg(text_fg).bg(pane_bg))),
            Line::from(Span::styled(
                "  documents/  downloads/",
                Style::new().fg(text_fg).bg(pane_bg),
            )),
            Line::from(Span::styled(
                if is_editing_text {
                    " ▌projects/ ▐ ◀ Text Selected"
                } else {
                    " ▌projects/▐ "
                },
                Style::new()
                    .fg(text_sel_fg)
                    .bg(tsel_bg)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  videos/     music/",
                Style::new().fg(text_fg).bg(pane_bg),
            )),
            Line::from(Span::styled("$ █", Style::new().fg(text_fg).bg(pane_bg))),
        ];

        let block = Block::bordered()
            .border_type(BorderType::Double)
            .title(" bash ")
            .title_style(border_style)
            .border_style(border_style)
            .style(Style::new().bg(pane_bg));

        frame.render_widget(Paragraph::new(content).block(block), area);
    }

    fn render_pane_unselected(&self, frame: &mut Frame, area: Rect, bg: Color, text_fg: Color) {
        let is_editing = self.selected_element == PreviewElement::PaneUnselected;
        let pane_bg = if is_editing { lighten(bg, 25) } else { bg };
        let border_color = get_fg(ThemeComponentType::FrameUnselected, &self.theme);
        let border_style = Style::new().fg(border_color).add_modifier(if is_editing {
            Modifier::BOLD
        } else {
            Modifier::empty()
        });

        let content = vec![
            Line::from(Span::styled(
                "$ git status",
                Style::new().fg(text_fg).bg(pane_bg),
            )),
            Line::from(Span::styled(
                "On branch main",
                Style::new().fg(text_fg).bg(pane_bg),
            )),
            Line::from(Span::styled(
                "nothing to commit",
                Style::new().fg(text_fg).bg(pane_bg),
            )),
        ];

        let block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(" terminal ")
            .title_style(border_style)
            .border_style(border_style)
            .style(Style::new().bg(pane_bg));

        frame.render_widget(Paragraph::new(content).block(block), area);
    }

    fn render_pane_highlight(&self, frame: &mut Frame, area: Rect, bg: Color, text_fg: Color) {
        let t = &self.theme;
        let is_editing_frame = self.selected_element == PreviewElement::PaneHighlight;

        // Lighten whole pane bg when the frame itself is focused
        let pane_bg = if is_editing_frame {
            lighten(bg, 25)
        } else {
            bg
        };
        let border_color = get_fg(ThemeComponentType::FrameHighlight, t);
        let border_style = Style::new()
            .fg(border_color)
            .add_modifier(if is_editing_frame {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });

        // Component colors
        let tbl_title_fg = get_fg(ThemeComponentType::TableTitle, t);
        let tbl_title_bg = get_bg(ThemeComponentType::TableTitle, t);
        let tbl_sel_fg = get_fg(ThemeComponentType::TableCellSelected, t);
        let tbl_sel_bg = get_bg(ThemeComponentType::TableCellSelected, t);
        let tbl_unsel_fg = get_fg(ThemeComponentType::TableCellUnselected, t);
        let tbl_unsel_bg = get_bg(ThemeComponentType::TableCellUnselected, t);
        let list_sel_fg = get_fg(ThemeComponentType::ListSelected, t);
        let list_sel_bg = get_bg(ThemeComponentType::ListSelected, t);
        let list_unsel_fg = get_fg(ThemeComponentType::ListUnselected, t);
        let list_unsel_bg = get_bg(ThemeComponentType::ListUnselected, t);
        let exit_ok_fg = get_fg(ThemeComponentType::ExitCodeSuccess, t);
        let exit_ok_bg = get_bg(ThemeComponentType::ExitCodeSuccess, t);
        let exit_err_fg = get_fg(ThemeComponentType::ExitCodeError, t);
        let exit_err_bg = get_bg(ThemeComponentType::ExitCodeError, t);

        let sel = |elem: PreviewElement, fg: Color, bg_c: Color| -> (Color, Color) {
            if self.selected_element == elem {
                (fg, lighten(bg_c, 45))
            } else {
                (fg, bg_c)
            }
        };
        let sel_mod = |elem: PreviewElement| -> Modifier {
            if self.selected_element == elem {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }
        };

        let (tt_fg, tt_bg) = sel(PreviewElement::TableTitle, tbl_title_fg, tbl_title_bg);
        let (ts_fg, ts_bg) = sel(PreviewElement::TableCellSelected, tbl_sel_fg, tbl_sel_bg);
        let (tu_fg, tu_bg) = sel(
            PreviewElement::TableCellUnselected,
            tbl_unsel_fg,
            tbl_unsel_bg,
        );
        let (ls_fg, ls_bg) = sel(PreviewElement::ListSelected, list_sel_fg, list_sel_bg);
        let (lu_fg, lu_bg) = sel(PreviewElement::ListUnselected, list_unsel_fg, list_unsel_bg);
        let (eo_fg, eo_bg) = sel(PreviewElement::ExitSuccess, exit_ok_fg, exit_ok_bg);
        let (ee_fg, ee_bg) = sel(PreviewElement::ExitError, exit_err_fg, exit_err_bg);

        let content = vec![
            // Table section
            Line::from(Span::styled(
                " Name         Size ",
                Style::new()
                    .fg(tt_fg)
                    .bg(tt_bg)
                    .add_modifier(sel_mod(PreviewElement::TableTitle) | Modifier::BOLD),
            )),
            Line::from(Span::styled(
                " main.rs      4.2K ",
                Style::new()
                    .fg(ts_fg)
                    .bg(ts_bg)
                    .add_modifier(sel_mod(PreviewElement::TableCellSelected)),
            )),
            Line::from(Span::styled(
                " Cargo.toml   1.1K ",
                Style::new()
                    .fg(tu_fg)
                    .bg(tu_bg)
                    .add_modifier(sel_mod(PreviewElement::TableCellUnselected)),
            )),
            Line::from(Span::raw("")),
            // List section
            Line::from(Span::styled(
                " > main.rs          ",
                Style::new()
                    .fg(ls_fg)
                    .bg(ls_bg)
                    .add_modifier(sel_mod(PreviewElement::ListSelected)),
            )),
            Line::from(Span::styled(
                "   Cargo.toml       ",
                Style::new()
                    .fg(lu_fg)
                    .bg(lu_bg)
                    .add_modifier(sel_mod(PreviewElement::ListUnselected)),
            )),
            Line::from(Span::styled(
                "   README.md        ",
                Style::new().fg(list_unsel_fg).bg(list_unsel_bg),
            )),
            Line::from(Span::styled("", Style::new().bg(pane_bg))),
            // Exit codes
            Line::from(vec![
                Span::styled(" ", Style::new().bg(pane_bg)),
                Span::styled(
                    " exit:0 ",
                    Style::new()
                        .fg(eo_fg)
                        .bg(eo_bg)
                        .add_modifier(sel_mod(PreviewElement::ExitSuccess)),
                ),
                Span::raw("  "),
                Span::styled(
                    " exit:1 ",
                    Style::new()
                        .fg(ee_fg)
                        .bg(ee_bg)
                        .add_modifier(sel_mod(PreviewElement::ExitError)),
                ),
            ]),
        ];

        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title(" nvim ")
            .title_style(border_style)
            .border_style(border_style)
            .style(Style::new().bg(pane_bg));

        frame.render_widget(Paragraph::new(content).block(block), area);
    }

    fn render_zellij_status_bar(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let is_editing_status = self.selected_element == PreviewElement::StatusBar;
        let bar_fg = get_fg(ThemeComponentType::TextUnselected, t);
        let bar_bg = get_bg(ThemeComponentType::TextUnselected, t);
        let sel_fg = get_fg(ThemeComponentType::RibbonSelected, t);
        let sel_bg = get_bg(ThemeComponentType::RibbonSelected, t);
        let unsel_fg = get_fg(ThemeComponentType::RibbonUnselected, t);
        let unsel_bg = get_bg(ThemeComponentType::RibbonUnselected, t);

        // Mode label changes based on app state and selection
        let (mode_label, mode_fg, mode_bg) = if is_editing_status {
            (" STATUS ", sel_fg, lighten(sel_bg, 40))
        } else {
            match self.input_mode {
                InputMode::ColorPicker => (" COLOR  ", sel_fg, sel_bg),
                InputMode::Preview => (" NORMAL ", sel_fg, sel_bg),
                InputMode::ThemeNameInput => (" SAVE   ", sel_fg, sel_bg),
                InputMode::ThemeLoad => (" LOAD   ", sel_fg, sel_bg),
                InputMode::Help => (" HELP   ", sel_fg, sel_bg),
            }
        };

        // ── helper: one ▶ content ▶ pill on unsel colors ─────────────────
        // Returns (open_arrow, key_span, action_span, close_arrow)
        let pill = |key: &str, action: &str| -> [Span<'static>; 4] {
            [
                Span::styled("", Style::new().fg(unsel_bg).bg(bar_bg)),
                Span::styled(
                    format!(" {} ", key),
                    Style::new()
                        .fg(sel_fg)
                        .bg(unsel_bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", action),
                    Style::new().fg(unsel_fg).bg(unsel_bg),
                ),
                Span::styled("", Style::new().fg(unsel_bg).bg(bar_bg)),
            ]
        };

        let gap = || Span::styled(" ", Style::new().bg(bar_bg));

        let mut spans: Vec<Span> = Vec::new();

        // ── Mode pill ─────────────────────────────────────────────────────
        spans.push(Span::styled("", Style::new().fg(mode_bg).bg(bar_bg)));
        spans.push(Span::styled(
            mode_label,
            Style::new()
                .fg(mode_fg)
                .bg(mode_bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("", Style::new().fg(mode_bg).bg(bar_bg)));
        spans.push(gap());

        // ── Keybinding pills ──────────────────────────────────────────────
        let bindings: &[(&str, &str)] = match self.input_mode {
            InputMode::Preview => &[
                ("↑↓←→", "NAVIGATE"),
                ("c", "COLOR"),
                ("tab", "FG/BG"),
                ("s", "SAVE AS"),
                ("l", "LOAD"),
                ("a", "SAVE+APPLY"),
                ("?", "HELP"),
                ("q", "QUIT"),
            ],
            InputMode::ColorPicker => &[
                ("↑↓", "CHANNEL"),
                ("←→", "±5"),
                ("S+←→", "±1"),
                ("PgUp/Dn", "±25"),
                ("#", "HEX"),
                ("tab", "FG/BG"),
                ("Enter", "KEEP"),
                ("Esc", "CANCEL"),
            ],
            InputMode::ThemeNameInput => &[
                ("type", "NAME"),
                ("Enter", "SAVE"),
                ("Esc", "CANCEL"),
            ],
            InputMode::ThemeLoad => &[
                ("↑↓", "SELECT"),
                ("Enter", "LOAD"),
                ("Esc", "CANCEL"),
            ],
            InputMode::Help => &[
                ("Esc", "CLOSE"),
            ],
        };
        for (key, action) in bindings {
            spans.extend(pill(key, action));
            spans.push(gap());
        }

        // ── Right side: element info + optional save message ──────────────
        let current_color = self.get_color_by_attr(self.selected_attribute);
        let info = format!(
            " {}{} │ {} {} #{:02x}{:02x}{:02x} ",
            self.theme.name,
            if self.dirty { "*" } else { "" },
            self.selected_element.label(),
            self.selected_attribute.label(),
            current_color.r,
            current_color.g,
            current_color.b,
        );

        let used: usize = spans.iter().map(|s| s.width()).sum();
        let right_width = info.len() + self.message.as_ref().map(|m| m.len() + 3).unwrap_or(0);
        let fill = (area.width as usize).saturating_sub(used + right_width);
        spans.push(Span::styled(" ".repeat(fill), Style::new().bg(bar_bg)));

        if let Some(ref msg) = self.message {
            let msg_color = if msg.starts_with('✗') { Color::Red } else { Color::Green };
            spans.push(Span::styled(
                format!(" {} ", msg),
                Style::new()
                    .fg(msg_color)
                    .bg(bar_bg)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        spans.push(Span::styled(
            info,
            Style::new()
                .fg(Color::Yellow)
                .bg(bar_bg)
                .add_modifier(Modifier::BOLD),
        ));

        frame.render_widget(
            Paragraph::new(vec![Line::from(spans)]).style(Style::new().bg(bar_bg)),
            area,
        );
    }

    // ── Color picker overlay ─────────────────────────────────────────────

    fn render_color_picker_mode(&self, frame: &mut Frame) {
        self.render_preview(frame);
        self.render_color_picker_overlay(frame);
    }

    fn render_theme_name_input_mode(&self, frame: &mut Frame) {
        self.render_preview(frame);
        self.render_theme_name_input_overlay(frame);
    }

    fn render_theme_load_mode(&self, frame: &mut Frame) {
        self.render_preview(frame);
        self.render_theme_load_overlay(frame);
    }

    fn render_help_mode(&self, frame: &mut Frame) {
        self.render_preview(frame);
        self.render_help_overlay(frame);
    }

    fn render_color_picker_overlay(&self, frame: &mut Frame) {
        let area = frame.area();
        let overlay_w = 52u16.min(area.width.saturating_sub(4));
        let overlay_h = 20u16.min(area.height.saturating_sub(4));
        let overlay_x = area.width.saturating_sub(overlay_w + 2);
        let overlay_y = (area.height.saturating_sub(overlay_h)) / 2;
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

        frame.render_widget(Clear, overlay_area);

        let r = self.color_editor.r;
        let g = self.color_editor.g;
        let b = self.color_editor.b;
        let cur_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        let cur_color = Color::Rgb(r, g, b);

        // Derive "before" color from saved original_component
        let orig_color = self.original_component.as_ref().map(|orig| {
            let c = match self.editing_attribute {
                PreviewAttribute::Foreground => orig.base,
                PreviewAttribute::Background => orig.background,
            };
            (
                Color::Rgb(c.r, c.g, c.b),
                format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
            )
        });

        let element_name = self.selected_element.label();
        let attr_name = self.editing_attribute.label();

        let mut lines: Vec<Line> = Vec::new();

        // Before/after swatch
        if let Some((before_color, before_hex)) = orig_color {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("        ", Style::new().bg(before_color)),
                Span::styled("  →  ", Style::new().fg(Color::DarkGray)),
                Span::styled("        ", Style::new().bg(cur_color)),
                Span::raw("  "),
                Span::styled(
                    format!("{} → {}", before_hex, cur_hex),
                    Style::new().fg(Color::White),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("        ", Style::new().bg(cur_color)),
                Span::raw("  "),
                Span::styled(
                    format!("{} — {}", attr_name, cur_hex),
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        lines.push(Line::from(""));

        // RGB sliders
        for (label, value, ch) in [("R", r, 0usize), ("G", g, 1), ("B", b, 2)] {
            let is_sel = self.color_editor.editing_channel == ch;
            let color = match ch {
                0 => Color::Red,
                1 => Color::Green,
                _ => Color::Blue,
            };
            lines.extend(render_slider_lines(label, value, is_sel, color));
            lines.push(Line::from(""));
        }

        // RGB decimal
        lines.push(Line::from(vec![Span::styled(
            format!("  rgb({} {} {})", r, g, b),
            Style::new().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(""));

        // Hex input area
        if let Some(ref hex_str) = self.color_editor.hex_input {
            let display = format!("  #{:<6}█", hex_str);
            lines.push(Line::from(vec![
                Span::styled(display, Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("  type hex, Enter to apply, Esc to cancel", Style::new().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "  press # to enter hex code",
                Style::new().fg(Color::DarkGray),
            )]));
        }
        lines.push(Line::from(""));

        // Keybinding hints
        lines.push(Line::from(vec![Span::styled(
            " ↑↓:channel  ←→:±5  Shift+←→:±1  PgUp/Dn:±25",
            Style::new().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(vec![Span::styled(
            " tab:fg/bg  Enter:keep  Esc:cancel",
            Style::new().fg(Color::DarkGray),
        )]));

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(" {} — {} ", element_name, attr_name))
            .title_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .border_style(Style::new().fg(Color::Yellow))
            .style(Style::new().bg(Color::Rgb(18, 18, 18)));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(Color::Rgb(18, 18, 18))),
            overlay_area,
        );
    }

    fn render_theme_name_input_overlay(&self, frame: &mut Frame) {
        let area = centered_rect(frame.area(), 64, 8);
        frame.render_widget(Clear, area);

        let preview_name = normalize_theme_name(&self.theme_name_input);
        let normalized = if preview_name.is_empty() {
            String::from("<invalid>")
        } else {
            format!("{}.kdl", preview_name)
        };

        let lines = vec![
            Line::from(" Save theme as "),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Name: ", Style::new().fg(Color::Yellow)),
                Span::styled(
                    format!("{}_", self.theme_name_input),
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(" File: ", Style::new().fg(Color::DarkGray)),
                Span::styled(normalized, Style::new().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                " Letters, numbers, spaces, - and _ are allowed",
                Style::new().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                " Enter: save  Esc: cancel  Backspace: delete",
                Style::new().fg(Color::DarkGray),
            )),
        ];

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Theme Name ")
            .title_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .border_style(Style::new().fg(Color::Yellow))
            .style(Style::new().bg(Color::Rgb(18, 18, 18)));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(Color::Rgb(18, 18, 18))),
            area,
        );
    }

    fn render_help_overlay(&self, frame: &mut Frame) {
        let entries: &[(&str, &str)] = &[
            // Normal mode
            ("↑/j  ↓/k  ← →",  "Navigate preview elements"),
            ("Tab",             "Toggle foreground / background"),
            ("c",               "Open color picker for selected color"),
            ("s",               "Save theme as… (prompts for name)"),
            ("l",               "Open theme loader"),
            ("a",               "Apply current theme to Zellij config"),
            ("?",               "Toggle this help screen"),
            ("q / Esc",         "Quit"),
            ("",                ""),
            // Color picker
            ("↑ ↓",            "[Color picker] Select R / G / B channel"),
            ("← → (×5)",       "[Color picker] Adjust value"),
            ("Shift + ← →",    "[Color picker] Adjust value by 1"),
            ("PgUp / PgDn",    "[Color picker] Adjust value by 25"),
            ("Enter",           "[Color picker] Confirm color"),
            ("Esc",             "[Color picker] Cancel"),
            ("",                ""),
            // Theme loader
            ("↑ ↓",            "[Load] Navigate themes"),
            ("Enter",           "[Load] Load selected theme"),
            ("Esc",             "[Load] Cancel"),
        ];

        let height = (entries.len() as u16 + 4).min(frame.area().height.saturating_sub(4));
        let area = centered_rect(frame.area(), 68, height);
        frame.render_widget(Clear, area);

        let key_style   = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let desc_style  = Style::new().fg(Color::White);
        let empty_style = Style::new();

        let lines: Vec<Line> = entries
            .iter()
            .map(|(key, desc)| {
                if key.is_empty() {
                    Line::from(Span::styled("", empty_style))
                } else {
                    Line::from(vec![
                        Span::styled(format!("  {:<20}", key), key_style),
                        Span::styled(format!(" {}", desc), desc_style),
                    ])
                }
            })
            .collect();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Help ")
            .title_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .border_style(Style::new().fg(Color::Yellow))
            .style(Style::new().bg(Color::Rgb(18, 18, 18)));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(Color::Rgb(18, 18, 18))),
            area,
        );
    }

    fn render_theme_load_overlay(&self, frame: &mut Frame) {
        let height = (self.loadable_themes.len() as u16 + 6).clamp(8, 18);
        let area = centered_rect(frame.area(), 56, height);
        frame.render_widget(Clear, area);

        let mut lines = vec![Line::from(" Load saved theme "), Line::from("")];
        for (index, name) in self.loadable_themes.iter().enumerate() {
            let selected = index == self.selected_theme_index;
            let prefix = if selected { " > " } else { "   " };
            let style = if selected {
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(format!("{prefix}{name}"), style)));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Up/Down: select  Enter: load  Esc: cancel",
            Style::new().fg(Color::DarkGray),
        )));

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Theme Loader ")
            .title_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .border_style(Style::new().fg(Color::Yellow))
            .style(Style::new().bg(Color::Rgb(18, 18, 18)));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(Color::Rgb(18, 18, 18))),
            area,
        );
    }
}

// ── Slider rendering ────────────────────────────────────────────────────────

fn render_slider_lines(
    label: &str,
    value: u8,
    is_selected: bool,
    color: Color,
) -> Vec<Line<'static>> {
    let slider_width = 30usize;
    let filled = ((value as f32 / 255.0) * slider_width as f32) as usize;
    let bar: String = (0..slider_width)
        .map(|i| if i < filled { '█' } else { '░' })
        .collect();
    let value_str = format!("{:>3}", value);
    let label = label.to_string();

    if is_selected {
        vec![Line::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("{} ", label),
                Style::new().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(bar, Style::new().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(
                value_str,
                Style::new().fg(color).add_modifier(Modifier::BOLD),
            ),
        ])]
    } else {
        vec![Line::from(vec![
            Span::raw(" "),
            Span::raw(format!("{} ", label)),
            Span::styled(bar, Style::new().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::raw(value_str),
        ])]
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_width = width.min(area.width.saturating_sub(2)).max(1);
    let popup_height = height.min(area.height.saturating_sub(2)).max(1);
    let x = area.x + area.width.saturating_sub(popup_width) / 2;
    let y = area.y + area.height.saturating_sub(popup_height) / 2;
    Rect::new(x, y, popup_width, popup_height)
}

fn normalize_theme_name(input: &str) -> String {
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

// ── Theme component accessors ───────────────────────────────────────────────

/// Lighten an Rgb color by adding `amount` to each channel (saturating).
/// Used for the selection indicator — the selected element's bg gets nudged
/// lighter so you can still see the real color while knowing it's focused.
fn lighten(color: Color, amount: u8) -> Color {
    if let Color::Rgb(r, g, b) = color {
        Color::Rgb(
            r.saturating_add(amount),
            g.saturating_add(amount),
            b.saturating_add(amount),
        )
    } else {
        color
    }
}

fn get_fg(comp: ThemeComponentType, theme: &Theme) -> Color {
    let c = get_component_by_type(theme, comp);
    Color::Rgb(c.base.r, c.base.g, c.base.b)
}

fn get_bg(comp: ThemeComponentType, theme: &Theme) -> Color {
    let c = get_component_by_type(theme, comp);
    Color::Rgb(c.background.r, c.background.g, c.background.b)
}

fn get_component_by_type<'a>(
    theme: &'a Theme,
    component_type: ThemeComponentType,
) -> &'a ThemeComponent {
    match component_type {
        ThemeComponentType::TextUnselected => &theme.text_unselected,
        ThemeComponentType::TextSelected => &theme.text_selected,
        ThemeComponentType::RibbonUnselected => &theme.ribbon_unselected,
        ThemeComponentType::RibbonSelected => &theme.ribbon_selected,
        ThemeComponentType::TableTitle => &theme.table_title,
        ThemeComponentType::TableCellUnselected => &theme.table_cell_unselected,
        ThemeComponentType::TableCellSelected => &theme.table_cell_selected,
        ThemeComponentType::ListUnselected => &theme.list_unselected,
        ThemeComponentType::ListSelected => &theme.list_selected,
        ThemeComponentType::FrameUnselected => &theme.frame_unselected,
        ThemeComponentType::FrameSelected => &theme.frame_selected,
        ThemeComponentType::FrameHighlight => &theme.frame_highlight,
        ThemeComponentType::ExitCodeSuccess => &theme.exit_code_success,
        ThemeComponentType::ExitCodeError => &theme.exit_code_error,
    }
}

fn get_component_mut_by_type(
    theme: &mut Theme,
    component_type: ThemeComponentType,
) -> &mut ThemeComponent {
    match component_type {
        ThemeComponentType::TextUnselected => &mut theme.text_unselected,
        ThemeComponentType::TextSelected => &mut theme.text_selected,
        ThemeComponentType::RibbonUnselected => &mut theme.ribbon_unselected,
        ThemeComponentType::RibbonSelected => &mut theme.ribbon_selected,
        ThemeComponentType::TableTitle => &mut theme.table_title,
        ThemeComponentType::TableCellUnselected => &mut theme.table_cell_unselected,
        ThemeComponentType::TableCellSelected => &mut theme.table_cell_selected,
        ThemeComponentType::ListUnselected => &mut theme.list_unselected,
        ThemeComponentType::ListSelected => &mut theme.list_selected,
        ThemeComponentType::FrameUnselected => &mut theme.frame_unselected,
        ThemeComponentType::FrameSelected => &mut theme.frame_selected,
        ThemeComponentType::FrameHighlight => &mut theme.frame_highlight,
        ThemeComponentType::ExitCodeSuccess => &mut theme.exit_code_success,
        ThemeComponentType::ExitCodeError => &mut theme.exit_code_error,
    }
}

// ── Event loop ─────────────────────────────────────────────────────────────

pub fn run(mut app: App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    terminal.show_cursor()?;

    use crossterm::event::{self as ct_event};
    use crossterm::execute;

    execute!(terminal.backend_mut(), crossterm::event::EnableMouseCapture)?;

    loop {
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        if let ct_event::Event::Key(key) = ct_event::read()? {
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);

            match app.input_mode {
                InputMode::Preview => {
                    // Any navigation clears the save message
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up => {
                            app.selected_element.move_up();
                            app.message = None;
                        }
                        KeyCode::Down => {
                            app.selected_element.move_down();
                            app.message = None;
                        }
                        KeyCode::Left => {
                            app.selected_element.move_left();
                            app.message = None;
                        }
                        KeyCode::Right => {
                            app.selected_element.move_right();
                            app.message = None;
                        }
                        KeyCode::Tab => {
                            app.message = None;
                            app.selected_attribute.toggle();
                        }
                        KeyCode::Char('c') => {
                            app.message = None;
                            app.open_color_picker();
                        }
                        KeyCode::Char('s') => {
                            app.message = None;
                            app.open_theme_name_input();
                        }
                        KeyCode::Char('l') => {
                            app.message = None;
                            app.open_theme_load_dialog();
                        }
                        KeyCode::Char('a') => {
                            app.message = None;
                            if app.theme.name == "default" {
                                app.pending_apply = true;
                                app.open_theme_name_input();
                                app.message = Some(String::from(
                                    "Name this theme before applying",
                                ));
                            } else {
                                app.apply_theme_to_zellij();
                            }
                        }
                        KeyCode::Char('?') => {
                            app.message = None;
                            app.input_mode = InputMode::Help;
                        }
                        KeyCode::Enter => {
                            app.message = None;
                            app.open_color_picker();
                        }
                        KeyCode::Char('j') => {
                            app.selected_element.move_down();
                            app.message = None;
                        }
                        KeyCode::Char('k') => {
                            app.selected_element.move_up();
                            app.message = None;
                        }
                        _ => {}
                    }
                }
                InputMode::ColorPicker => match key.code {
                    KeyCode::Esc => {
                        if app.color_editor.hex_input.is_some() {
                            app.color_editor.cancel_hex();
                        } else {
                            app.close_color_picker(false);
                        }
                    }
                    KeyCode::Enter => {
                        if app.color_editor.hex_input.is_some() {
                            app.color_editor.commit_hex();
                            app.apply_current_color();
                        } else {
                            app.close_color_picker(true);
                        }
                    }
                    KeyCode::Tab => {
                        app.switch_editing_attribute();
                    }
                    KeyCode::Char('#') => {
                        app.color_editor.start_hex_input();
                    }
                    KeyCode::Char(c) if app.color_editor.hex_input.is_some() => {
                        app.color_editor.push_hex_char(c);
                        // Live preview: if 6 chars, update
                        if app.color_editor.hex_input.as_ref().map(|s| s.len()) == Some(6) {
                            let _ = app.color_editor.commit_hex();
                            app.apply_current_color();
                        }
                    }
                    KeyCode::Backspace if app.color_editor.hex_input.is_some() => {
                        app.color_editor.pop_hex_char();
                    }
                    // Up → toward R (channel 0, top slider)
                    KeyCode::Up => {
                        app.color_editor.select_prev_channel();
                    }
                    // Down → toward B (channel 2, bottom slider)
                    KeyCode::Down => {
                        app.color_editor.select_next_channel();
                    }
                    KeyCode::Left => {
                        let delta = if shift { -1 } else { -5 };
                        app.color_editor.adjust(delta);
                        app.apply_current_color();
                    }
                    KeyCode::Right => {
                        let delta = if shift { 1 } else { 5 };
                        app.color_editor.adjust(delta);
                        app.apply_current_color();
                    }
                    KeyCode::PageUp => {
                        app.color_editor.adjust(25);
                        app.apply_current_color();
                    }
                    KeyCode::PageDown => {
                        app.color_editor.adjust(-25);
                        app.apply_current_color();
                    }
                    _ => {}
                },
                InputMode::ThemeNameInput => match key.code {
                    KeyCode::Esc => {
                        app.pending_apply = false;
                        app.input_mode = InputMode::Preview;
                        app.message = Some(String::from("Save cancelled"));
                    }
                    KeyCode::Enter => {
                        app.save_theme_as_input_name();
                    }
                    KeyCode::Backspace => {
                        app.pop_theme_name_char();
                    }
                    KeyCode::Char(c) => {
                        app.push_theme_name_char(c);
                    }
                    _ => {}
                },
                InputMode::ThemeLoad => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Preview;
                        app.message = Some(String::from("Load cancelled"));
                    }
                    KeyCode::Enter => {
                        app.load_selected_theme();
                    }
                    KeyCode::Up => {
                        app.move_theme_selection_up();
                    }
                    KeyCode::Down => {
                        app.move_theme_selection_down();
                    }
                    _ => {}
                },
                InputMode::Help => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                        app.input_mode = InputMode::Preview;
                    }
                    _ => {}
                },
            }
        }
    }

    execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture
    )?;
    ratatui::restore();

    Ok(())
}
