use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn saturating_add(self, delta: i8) -> u8 {
        let val = i32::from(self) + i32::from(delta);
        val.clamp(0, 255) as u8
    }

    pub fn saturating_add_unsigned(self, delta: u8) -> u8 {
        let val = i32::from(self) + i32::from(delta);
        val.clamp(0, 255) as u8
    }
}

impl From<RgbColor> for i32 {
    fn from(color: RgbColor) -> Self {
        i32::from(color.r) * 256 * 256 + i32::from(color.g) * 256 + i32::from(color.b)
    }
}

impl From<RgbColor> for u8 {
    fn from(color: RgbColor) -> Self {
        color.r
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ThemeComponent {
    pub base: RgbColor,
    pub background: RgbColor,
    pub emphasis_0: RgbColor,
    pub emphasis_1: RgbColor,
    pub emphasis_2: RgbColor,
    pub emphasis_3: RgbColor,
}

impl ThemeComponent {
    pub fn new(base: RgbColor, background: RgbColor) -> Self {
        Self {
            base,
            background,
            emphasis_0: RgbColor::new(255, 255, 255),
            emphasis_1: RgbColor::new(200, 200, 200),
            emphasis_2: RgbColor::new(150, 150, 150),
            emphasis_3: RgbColor::new(100, 100, 100),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub text_unselected: ThemeComponent,
    pub text_selected: ThemeComponent,
    pub ribbon_unselected: ThemeComponent,
    pub ribbon_selected: ThemeComponent,
    pub table_title: ThemeComponent,
    pub table_cell_unselected: ThemeComponent,
    pub table_cell_selected: ThemeComponent,
    pub list_unselected: ThemeComponent,
    pub list_selected: ThemeComponent,
    pub frame_unselected: ThemeComponent,
    pub frame_selected: ThemeComponent,
    pub frame_highlight: ThemeComponent,
    pub exit_code_success: ThemeComponent,
    pub exit_code_error: ThemeComponent,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            text_unselected: ThemeComponent::new(
                RgbColor::new(200, 200, 200),
                RgbColor::new(30, 30, 30),
            ),
            text_selected: ThemeComponent::new(
                RgbColor::new(255, 255, 255),
                RgbColor::new(60, 60, 60),
            ),
            ribbon_unselected: ThemeComponent::new(
                RgbColor::new(180, 180, 180),
                RgbColor::new(40, 40, 40),
            ),
            ribbon_selected: ThemeComponent::new(
                RgbColor::new(255, 255, 255),
                RgbColor::new(80, 80, 80),
            ),
            table_title: ThemeComponent::new(
                RgbColor::new(200, 200, 200),
                RgbColor::new(50, 50, 50),
            ),
            table_cell_unselected: ThemeComponent::new(
                RgbColor::new(180, 180, 180),
                RgbColor::new(35, 35, 35),
            ),
            table_cell_selected: ThemeComponent::new(
                RgbColor::new(255, 255, 255),
                RgbColor::new(60, 60, 60),
            ),
            list_unselected: ThemeComponent::new(
                RgbColor::new(180, 180, 180),
                RgbColor::new(30, 30, 30),
            ),
            list_selected: ThemeComponent::new(
                RgbColor::new(255, 255, 255),
                RgbColor::new(70, 70, 70),
            ),
            frame_unselected: ThemeComponent::new(
                RgbColor::new(100, 100, 100),
                RgbColor::new(30, 30, 30),
            ),
            frame_selected: ThemeComponent::new(
                RgbColor::new(255, 255, 255),
                RgbColor::new(50, 50, 50),
            ),
            frame_highlight: ThemeComponent::new(
                RgbColor::new(255, 200, 100),
                RgbColor::new(60, 50, 30),
            ),
            exit_code_success: ThemeComponent::new(
                RgbColor::new(100, 255, 100),
                RgbColor::new(30, 30, 30),
            ),
            exit_code_error: ThemeComponent::new(
                RgbColor::new(255, 100, 100),
                RgbColor::new(30, 30, 30),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeComponentType {
    TextUnselected,
    TextSelected,
    RibbonUnselected,
    RibbonSelected,
    TableTitle,
    TableCellUnselected,
    TableCellSelected,
    ListUnselected,
    ListSelected,
    FrameUnselected,
    FrameSelected,
    FrameHighlight,
    ExitCodeSuccess,
    ExitCodeError,
}

impl ThemeComponentType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TextUnselected => "Text (Unselected)",
            Self::TextSelected => "Text (Selected)",
            Self::RibbonUnselected => "Ribbon/Tab (Unselected)",
            Self::RibbonSelected => "Ribbon/Tab (Selected)",
            Self::TableTitle => "Table Title",
            Self::TableCellUnselected => "Table Cell (Unselected)",
            Self::TableCellSelected => "Table Cell (Selected)",
            Self::ListUnselected => "List Item (Unselected)",
            Self::ListSelected => "List Item (Selected)",
            Self::FrameUnselected => "Frame (Unselected)",
            Self::FrameSelected => "Frame (Selected)",
            Self::FrameHighlight => "Frame Highlight",
            Self::ExitCodeSuccess => "Exit Code (Success)",
            Self::ExitCodeError => "Exit Code (Error)",
        }
    }

    pub fn component_key(&self) -> &'static str {
        match self {
            Self::TextUnselected => "text_unselected",
            Self::TextSelected => "text_selected",
            Self::RibbonUnselected => "ribbon_unselected",
            Self::RibbonSelected => "ribbon_selected",
            Self::TableTitle => "table_title",
            Self::TableCellUnselected => "table_cell_unselected",
            Self::TableCellSelected => "table_cell_selected",
            Self::ListUnselected => "list_unselected",
            Self::ListSelected => "list_selected",
            Self::FrameUnselected => "frame_unselected",
            Self::FrameSelected => "frame_selected",
            Self::FrameHighlight => "frame_highlight",
            Self::ExitCodeSuccess => "exit_code_success",
            Self::ExitCodeError => "exit_code_error",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::TextUnselected,
            Self::TextSelected,
            Self::RibbonUnselected,
            Self::RibbonSelected,
            Self::TableTitle,
            Self::TableCellUnselected,
            Self::TableCellSelected,
            Self::ListUnselected,
            Self::ListSelected,
            Self::FrameUnselected,
            Self::FrameSelected,
            Self::FrameHighlight,
            Self::ExitCodeSuccess,
            Self::ExitCodeError,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorAttribute {
    Base,
    Background,
    Emphasis0,
    Emphasis1,
    Emphasis2,
    Emphasis3,
}

impl ColorAttribute {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::Background => "Background",
            Self::Emphasis0 => "Emphasis 0",
            Self::Emphasis1 => "Emphasis 1",
            Self::Emphasis2 => "Emphasis 2",
            Self::Emphasis3 => "Emphasis 3",
        }
    }
}
