use crate::theme::RgbColor;
use palette::{FromColor, Hsl, Hsv, RgbHue, Srgb};
use ratatui::layout::{Constraint, Layout, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerMode {
    RgbSliders,
    HslField,
}

impl ColorPickerMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::RgbSliders => Self::HslField,
            Self::HslField => Self::RgbSliders,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerFocus {
    ModeToggle,
    RgbSlider(usize),
    HslField,
    LightnessSlider,
    HexField,
    RgbField(usize),
    HslFieldValue(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDragTarget {
    HslField,
    LightnessSlider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditableField {
    Hex,
    Rgb(usize),
    Hsl(usize),
}

#[derive(Debug, Clone)]
pub struct TextEditState {
    pub target: EditableField,
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct HslValue {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct HsvValue {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PickerRects {
    pub overlay: Rect,
    pub mode_switch: Rect,
    pub main_view: Rect,
    pub aux_slider: Rect,
    pub hex_field: Rect,
    pub rgb_fields: [Rect; 3],
    pub hsl_fields: [Rect; 3],
}

impl Default for PickerRects {
    fn default() -> Self {
        let zero = Rect::new(0, 0, 0, 0);
        Self {
            overlay: zero,
            mode_switch: zero,
            main_view: zero,
            aux_slider: zero,
            hex_field: zero,
            rgb_fields: [zero; 3],
            hsl_fields: [zero; 3],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorEditor {
    pub mode: ColorPickerMode,
    pub focus: ColorPickerFocus,
    pub rgb: [u8; 3],
    pub hsl: HslValue,
    pub hsv: HsvValue,
    pub drag_target: Option<ColorDragTarget>,
    pub text_edit: Option<TextEditState>,
}

impl ColorEditor {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let mut editor = Self {
            mode: ColorPickerMode::RgbSliders,
            focus: ColorPickerFocus::RgbSlider(0),
            rgb: [r, g, b],
            hsl: HslValue {
                hue: 0.0,
                saturation: 0.0,
                lightness: 0.0,
            },
            hsv: HsvValue {
                hue: 0.0,
                saturation: 0.0,
                value: 0.0,
            },
            drag_target: None,
            text_edit: None,
        };
        editor.sync_from_rgb_preserve_hue(None);
        editor
    }

    pub fn to_rgb(&self) -> RgbColor {
        RgbColor::new(self.rgb[0], self.rgb[1], self.rgb[2])
    }

    pub fn hsv(&self) -> HsvValue {
        self.hsv
    }

    pub fn hex(&self) -> String {
        self.to_rgb().to_hex()
    }

    pub fn start_hex_input(&mut self) {
        self.focus = ColorPickerFocus::HexField;
        self.text_edit = Some(TextEditState {
            target: EditableField::Hex,
            value: self.hex().trim_start_matches('#').to_string(),
        });
    }

    pub fn start_editing_focused(&mut self) {
        let target = match self.focus {
            ColorPickerFocus::HexField => EditableField::Hex,
            ColorPickerFocus::RgbField(i) => EditableField::Rgb(i),
            ColorPickerFocus::HslFieldValue(i) => EditableField::Hsl(i),
            _ => return,
        };
        let value = match target {
            EditableField::Hex => self.hex().trim_start_matches('#').to_string(),
            EditableField::Rgb(i) => self.rgb[i].to_string(),
            EditableField::Hsl(0) => self.hsl.hue.round().to_string(),
            EditableField::Hsl(1) => self.hsl.saturation.round().to_string(),
            EditableField::Hsl(2) => self.hsl.lightness.round().to_string(),
            EditableField::Hsl(_) => String::new(),
        };
        self.text_edit = Some(TextEditState { target, value });
    }

    pub fn is_editing_text(&self) -> bool {
        self.text_edit.is_some()
    }

    pub fn push_input_char(&mut self, c: char) -> bool {
        let Some(edit) = self.text_edit.as_mut() else {
            return false;
        };
        match edit.target {
            EditableField::Hex => {
                if edit.value.len() < 6 && c.is_ascii_hexdigit() {
                    edit.value.push(c.to_ascii_lowercase());
                    return true;
                }
            }
            EditableField::Rgb(_) | EditableField::Hsl(_) => {
                if c.is_ascii_digit() {
                    edit.value.push(c);
                    return true;
                }
                if c == '.' && matches!(edit.target, EditableField::Hsl(_)) && !edit.value.contains('.') {
                    edit.value.push(c);
                    return true;
                }
            }
        }
        false
    }

    pub fn pop_input_char(&mut self) -> bool {
        let Some(edit) = self.text_edit.as_mut() else {
            return false;
        };
        edit.value.pop();
        true
    }

    pub fn cancel_text_edit(&mut self) {
        self.text_edit = None;
    }

    pub fn commit_text_edit(&mut self) -> bool {
        let Some(edit) = self.text_edit.take() else {
            return false;
        };
        match edit.target {
            EditableField::Hex => {
                if let Some(rgb) = RgbColor::from_hex(&edit.value) {
                    self.set_rgb([rgb.r, rgb.g, rgb.b]);
                    return true;
                }
            }
            EditableField::Rgb(i) => {
                if let Ok(value) = edit.value.parse::<u16>() {
                    if value <= 255 {
                        let mut rgb = self.rgb;
                        rgb[i] = value as u8;
                        self.set_rgb(rgb);
                        return true;
                    }
                }
            }
            EditableField::Hsl(i) => {
                if let Ok(value) = edit.value.parse::<f32>() {
                    match i {
                        0 => self.set_hsl(value, self.hsl.saturation, self.hsl.lightness),
                        1 => self.set_hsl(self.hsl.hue, value, self.hsl.lightness),
                        2 => self.set_hsl(self.hsl.hue, self.hsl.saturation, value),
                        _ => {}
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.toggle();
        self.focus = match self.mode {
            ColorPickerMode::RgbSliders => ColorPickerFocus::RgbSlider(0),
            ColorPickerMode::HslField => ColorPickerFocus::HslField,
        };
        self.drag_target = None;
        self.text_edit = None;
    }

    pub fn focus_next(&mut self, reverse: bool) {
        self.text_edit = None;
        self.drag_target = None;
        if self.mode == ColorPickerMode::HslField {
            self.focus = match self.focus {
                ColorPickerFocus::LightnessSlider => ColorPickerFocus::HslField,
                _ => ColorPickerFocus::LightnessSlider,
            };
            return;
        }
        let order = self.focus_order();
        let idx = order.iter().position(|focus| *focus == self.focus).unwrap_or(0);
        let next = if reverse {
            if idx == 0 { order.len() - 1 } else { idx - 1 }
        } else {
            (idx + 1) % order.len()
        };
        self.focus = order[next];
    }

    pub fn adjust_rgb_slider_selection(&mut self, delta: i32) {
        if let ColorPickerFocus::RgbSlider(idx) = self.focus {
            let mut rgb = self.rgb;
            rgb[idx] = (i32::from(rgb[idx]) + delta).clamp(0, 255) as u8;
            self.set_rgb(rgb);
        }
    }

    pub fn move_rgb_slider_focus(&mut self, reverse: bool) {
        let current = match self.focus {
            ColorPickerFocus::RgbSlider(idx) => idx,
            _ => 0,
        };
        let next = if reverse {
            if current == 0 { 2 } else { current - 1 }
        } else {
            (current + 1) % 3
        };
        self.focus = ColorPickerFocus::RgbSlider(next);
    }

    pub fn adjust_focused_numeric(&mut self, delta: f32) -> bool {
        match self.focus {
            ColorPickerFocus::RgbSlider(i) | ColorPickerFocus::RgbField(i) => {
                let mut rgb = self.rgb;
                rgb[i] = (f32::from(rgb[i]) + delta).clamp(0.0, 255.0).round() as u8;
                self.set_rgb(rgb);
                true
            }
            ColorPickerFocus::HslFieldValue(0) => {
                self.set_hsl(self.hsl.hue + delta, self.hsl.saturation, self.hsl.lightness);
                true
            }
            ColorPickerFocus::HslFieldValue(1) => {
                self.set_hsl(self.hsl.hue, self.hsl.saturation + delta, self.hsl.lightness);
                true
            }
            ColorPickerFocus::HslFieldValue(2) => {
                self.set_hsl(self.hsl.hue, self.hsl.saturation, self.hsl.lightness + delta);
                true
            }
            ColorPickerFocus::LightnessSlider => {
                self.set_hsv(self.hsv.hue, self.hsv.saturation, self.hsv.value + delta);
                true
            }
            _ => false,
        }
    }

    pub fn update_from_hsl_field(&mut self, x_frac: f32, y_frac: f32) {
        let hue = x_frac.clamp(0.0, 1.0) * 360.0;
        let saturation = (1.0 - y_frac.clamp(0.0, 1.0)) * 100.0;
        self.set_hsv(hue, saturation, self.hsv.value);
        self.focus = ColorPickerFocus::HslField;
    }

    pub fn update_lightness_from_frac(&mut self, y_frac: f32) {
        let value = (1.0 - y_frac.clamp(0.0, 1.0)) * 100.0;
        self.set_hsv(self.hsv.hue, self.hsv.saturation, value);
        self.focus = ColorPickerFocus::LightnessSlider;
    }

    pub fn nudge_hsl_field(&mut self, delta_hue: f32, delta_saturation: f32) {
        self.set_hsv(
            self.hsv.hue + delta_hue,
            self.hsv.saturation + delta_saturation,
            self.hsv.value,
        );
    }

    pub fn set_drag_target(&mut self, target: Option<ColorDragTarget>) {
        self.drag_target = target;
    }

    pub fn field_value(&self, field: EditableField) -> String {
        if let Some(edit) = &self.text_edit {
            let matches = match (edit.target, field) {
                (EditableField::Hex, EditableField::Hex) => true,
                (EditableField::Rgb(a), EditableField::Rgb(b)) => a == b,
                (EditableField::Hsl(a), EditableField::Hsl(b)) => a == b,
                _ => false,
            };
            if matches {
                return edit.value.clone();
            }
        }
        match field {
            EditableField::Hex => self.hex(),
            EditableField::Rgb(i) => self.rgb[i].to_string(),
            EditableField::Hsl(0) => format!("{:.0}", self.hsl.hue),
            EditableField::Hsl(1) => format!("{:.0}", self.hsl.saturation),
            EditableField::Hsl(2) => format!("{:.0}", self.hsl.lightness),
            EditableField::Hsl(_) => String::new(),
        }
    }

    pub fn focus_label(&self) -> &'static str {
        match self.focus {
            ColorPickerFocus::ModeToggle => "mode switch",
            ColorPickerFocus::RgbSlider(0) => "red slider",
            ColorPickerFocus::RgbSlider(1) => "green slider",
            ColorPickerFocus::RgbSlider(2) => "blue slider",
            ColorPickerFocus::RgbSlider(_) => "rgb slider",
            ColorPickerFocus::HslField => "color field",
            ColorPickerFocus::LightnessSlider => "value slider",
            ColorPickerFocus::HexField => "hex field",
            ColorPickerFocus::RgbField(0) => "red field",
            ColorPickerFocus::RgbField(1) => "green field",
            ColorPickerFocus::RgbField(2) => "blue field",
            ColorPickerFocus::RgbField(_) => "rgb field",
            ColorPickerFocus::HslFieldValue(0) => "hue field",
            ColorPickerFocus::HslFieldValue(1) => "sat field",
            ColorPickerFocus::HslFieldValue(2) => "light field",
            ColorPickerFocus::HslFieldValue(_) => "hsl field",
        }
    }

    pub fn focus_for_point(&self, rects: &PickerRects, x: u16, y: u16) -> Option<ColorPickerFocus> {
        let point = (x, y);
        if contains(rects.mode_switch, point) {
            return Some(ColorPickerFocus::ModeToggle);
        }
        if contains(rects.main_view, point) {
            return Some(match self.mode {
                ColorPickerMode::RgbSliders => {
                    let idx = ((y.saturating_sub(rects.main_view.y)) / 2).min(2) as usize;
                    ColorPickerFocus::RgbSlider(idx)
                }
                ColorPickerMode::HslField => ColorPickerFocus::HslField,
            });
        }
        if contains(rects.aux_slider, point) {
            return Some(match self.mode {
                ColorPickerMode::RgbSliders => ColorPickerFocus::RgbSlider(2),
                ColorPickerMode::HslField => ColorPickerFocus::LightnessSlider,
            });
        }
        if contains(rects.hex_field, point) {
            return Some(ColorPickerFocus::HexField);
        }
        for (idx, rect) in rects.rgb_fields.iter().enumerate() {
            if contains(*rect, point) {
                return Some(ColorPickerFocus::RgbField(idx));
            }
        }
        for (idx, rect) in rects.hsl_fields.iter().enumerate() {
            if contains(*rect, point) {
                return Some(ColorPickerFocus::HslFieldValue(idx));
            }
        }
        None
    }

    pub fn set_focus(&mut self, focus: ColorPickerFocus) {
        self.focus = focus;
        self.text_edit = None;
        self.drag_target = None;
    }

    fn focus_order(&self) -> Vec<ColorPickerFocus> {
        match self.mode {
            ColorPickerMode::RgbSliders => vec![
                ColorPickerFocus::ModeToggle,
                ColorPickerFocus::RgbSlider(0),
                ColorPickerFocus::RgbSlider(1),
                ColorPickerFocus::RgbSlider(2),
                ColorPickerFocus::HexField,
                ColorPickerFocus::RgbField(0),
                ColorPickerFocus::RgbField(1),
                ColorPickerFocus::RgbField(2),
                ColorPickerFocus::HslFieldValue(0),
                ColorPickerFocus::HslFieldValue(1),
                ColorPickerFocus::HslFieldValue(2),
            ],
            ColorPickerMode::HslField => vec![
                ColorPickerFocus::ModeToggle,
                ColorPickerFocus::HslField,
                ColorPickerFocus::LightnessSlider,
                ColorPickerFocus::HexField,
                ColorPickerFocus::RgbField(0),
                ColorPickerFocus::RgbField(1),
                ColorPickerFocus::RgbField(2),
                ColorPickerFocus::HslFieldValue(0),
                ColorPickerFocus::HslFieldValue(1),
                ColorPickerFocus::HslFieldValue(2),
            ],
        }
    }

    fn set_rgb(&mut self, rgb: [u8; 3]) {
        self.rgb = rgb;
        let preserve_hue = Some(self.hsv.hue);
        self.sync_from_rgb_preserve_hue(preserve_hue);
    }

    fn set_hsl(&mut self, hue: f32, saturation: f32, lightness: f32) {
        self.hsl = HslValue {
            hue: normalize_hue(hue),
            saturation: saturation.clamp(0.0, 100.0),
            lightness: lightness.clamp(0.0, 100.0),
        };
        let hsl = Hsl::new(
            RgbHue::from_degrees(self.hsl.hue),
            self.hsl.saturation / 100.0,
            self.hsl.lightness / 100.0,
        );
        let srgb: Srgb<f32> = Srgb::from_color(hsl);
        let srgb_u8 = srgb.into_format::<u8>();
        self.rgb = [srgb_u8.red, srgb_u8.green, srgb_u8.blue];
    }

    fn set_hsv(&mut self, hue: f32, saturation: f32, value: f32) {
        self.hsv = HsvValue {
            hue: normalize_hue(hue),
            saturation: saturation.clamp(0.0, 100.0),
            value: value.clamp(0.0, 100.0),
        };
        let hsv = Hsv::new(
            RgbHue::from_degrees(self.hsv.hue),
            self.hsv.saturation / 100.0,
            self.hsv.value / 100.0,
        );
        let srgb: Srgb<f32> = Srgb::from_color(hsv);
        let srgb_u8 = srgb.into_format::<u8>();
        self.rgb = [srgb_u8.red, srgb_u8.green, srgb_u8.blue];
        self.sync_hsl_from_rgb();
    }

    fn sync_hsl_from_rgb(&mut self) {
        let hsl: Hsl = Hsl::from_color(srgb_f32(self.rgb));
        self.hsl = HslValue {
            hue: normalize_hue(hsl.hue.into_degrees()),
            saturation: hsl.saturation * 100.0,
            lightness: hsl.lightness * 100.0,
        };
    }

    fn sync_from_rgb_preserve_hue(&mut self, preserve_hue: Option<f32>) {
        let hsv: Hsv = Hsv::from_color(srgb_f32(self.rgb));
        let raw_saturation = hsv.saturation * 100.0;
        let raw_value = hsv.value * 100.0;
        let hue = if raw_saturation <= 0.01 || raw_value <= 0.01 {
            preserve_hue.unwrap_or_else(|| normalize_hue(hsv.hue.into_degrees()))
        } else {
            normalize_hue(hsv.hue.into_degrees())
        };
        self.hsv = HsvValue {
            hue,
            saturation: raw_saturation,
            value: raw_value,
        };
        self.sync_hsl_from_rgb();
        if self.hsl.saturation <= 0.01 {
            self.hsl.hue = hue;
        }
    }
}

pub fn picker_layout(area: Rect, mode: ColorPickerMode) -> PickerRects {
    let overlay_w = 76u16.min(area.width.saturating_sub(4));
    let overlay_h = 24u16.min(area.height.saturating_sub(4));
    let overlay = super::render::centered_rect(area, overlay_w, overlay_h);
    let inner = Rect::new(overlay.x + 1, overlay.y + 1, overlay.width.saturating_sub(2), overlay.height.saturating_sub(2));
    let [header, body, _footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(16),
        Constraint::Length(2),
    ])
    .areas(inner);
    let [main_col, side_col] = Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)]).areas(body);
    let (main_view, aux_slider) = match mode {
        ColorPickerMode::RgbSliders => {
            let [sliders, _spacer] = Layout::vertical([Constraint::Length(7), Constraint::Min(1)]).areas(main_col);
            (sliders, Rect::new(0, 0, 0, 0))
        }
        ColorPickerMode::HslField => {
            let [field, slider] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(6)]).areas(main_col);
            (field, slider)
        }
    };
    let [preview_area, fields_area] = Layout::vertical([Constraint::Length(5), Constraint::Fill(1)]).areas(side_col);
    let [mode_switch, preview_swatch] = Layout::horizontal([Constraint::Length(16), Constraint::Fill(1)]).areas(header);
    let [hex_row, rgb_row, hsl_row] = Layout::vertical([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3)]).areas(fields_area);
    let [hex_field, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(0)]).areas(hex_row);
    let rgb_fields = split_three(rgb_row);
    let hsl_fields = split_three(hsl_row);
    let _ = preview_area;
    let _ = preview_swatch;
    PickerRects {
        overlay,
        mode_switch,
        main_view,
        aux_slider,
        hex_field,
        rgb_fields,
        hsl_fields,
    }
}

pub fn split_three(area: Rect) -> [Rect; 3] {
    let [a, b, c] = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Percentage(33),
    ])
    .areas(area);
    [a, b, c]
}

pub fn hsv_field_cell(hue_deg: f32, saturation_pct: f32, value_pct: f32) -> RgbColor {
    let hsv = Hsv::new(
        RgbHue::from_degrees(normalize_hue(hue_deg)),
        saturation_pct.clamp(0.0, 100.0) / 100.0,
        value_pct.clamp(0.0, 100.0) / 100.0,
    );
    let srgb: Srgb<f32> = Srgb::from_color(hsv);
    let srgb = srgb.into_format::<u8>();
    RgbColor::new(srgb.red, srgb.green, srgb.blue)
}

pub fn contrast_text(rgb: RgbColor) -> RgbColor {
    let luminance = (0.2126 * f32::from(rgb.r) + 0.7152 * f32::from(rgb.g) + 0.0722 * f32::from(rgb.b)) / 255.0;
    if luminance > 0.55 {
        RgbColor::new(18, 18, 24)
    } else {
        RgbColor::new(245, 245, 250)
    }
}

pub fn srgb_f32(rgb: [u8; 3]) -> Srgb<f32> {
    Srgb::new(rgb[0], rgb[1], rgb[2]).into_format()
}

pub fn normalize_hue(hue: f32) -> f32 {
    hue.rem_euclid(360.0)
}

fn contains(rect: Rect, point: (u16, u16)) -> bool {
    point.0 >= rect.x
        && point.0 < rect.x + rect.width
        && point.1 >= rect.y
        && point.1 < rect.y + rect.height
}
