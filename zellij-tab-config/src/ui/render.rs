use crate::theme::ThemeComponentType;
use crate::ui::color_picker::{
    contrast_text, hsv_field_cell, picker_layout, ColorPickerFocus, ColorPickerMode, EditableField,
};
use crate::ui::state::{App, InputMode, PreviewAttribute, PreviewElement};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph},
    Frame,
};

use super::state::normalize_theme_name;

pub fn lighten(color: Color, amount: u8) -> Color {
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

pub fn get_fg(comp: ThemeComponentType, theme: &crate::theme::Theme) -> Color {
    let c = theme.get(comp);
    Color::Rgb(c.base.r, c.base.g, c.base.b)
}

pub fn get_bg(comp: ThemeComponentType, theme: &crate::theme::Theme) -> Color {
    let c = theme.get(comp);
    Color::Rgb(c.background.r, c.background.g, c.background.b)
}

pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_width = width.min(area.width.saturating_sub(2)).max(1);
    let popup_height = height.min(area.height.saturating_sub(2)).max(1);
    let x = area.x + area.width.saturating_sub(popup_width) / 2;
    let y = area.y + area.height.saturating_sub(popup_height) / 2;
    Rect::new(x, y, popup_width, popup_height)
}

fn clip_text(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

fn tui_rgb(color: crate::theme::RgbColor) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

impl App {
    pub fn render(&self, frame: &mut Frame) {
        match self.input_mode {
            InputMode::Preview => self.render_preview(frame),
            InputMode::ColorPicker => self.render_color_picker_mode(frame),
            InputMode::ThemeNameInput | InputMode::ThemeNameInputApply => {
                self.render_theme_name_input_mode(frame)
            }
            InputMode::ThemeLoad => self.render_theme_load_mode(frame),
            InputMode::ThemeLoadRename => {
                self.render_theme_load_mode(frame);
                self.render_theme_name_input_overlay(frame);
            }
            InputMode::ThemeLoadDeleteConfirm => {
                self.render_theme_load_mode(frame);
            }
            InputMode::UpdateRestartConfirm => {
                self.render_preview(frame);
                self.render_update_restart_overlay(frame);
            }
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
            .title(" Pane (Selected) ")
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
            .title(" Pane (Unselected) ")
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
            // git status header lines
            Line::from(Span::styled("$ git status", Style::new().fg(text_fg).bg(pane_bg))),
            Line::from(Span::styled("On branch main", Style::new().fg(text_fg).bg(pane_bg))),
            Line::from(Span::raw("")),
            // Changes staged — table_title style
            Line::from(Span::styled(
                " Changes staged",
                Style::new()
                    .fg(tt_fg)
                    .bg(tt_bg)
                    .add_modifier(sel_mod(PreviewElement::TableTitle) | Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  modified: main.rs",
                Style::new()
                    .fg(ts_fg)
                    .bg(ts_bg)
                    .add_modifier(sel_mod(PreviewElement::TableCellSelected)),
            )),
            Line::from(Span::styled(
                "  modified: lib.rs",
                Style::new()
                    .fg(tu_fg)
                    .bg(tu_bg)
                    .add_modifier(sel_mod(PreviewElement::TableCellUnselected)),
            )),
            Line::from(Span::raw("")),
            // Untracked files — list style
            Line::from(Span::styled(
                " Untracked files",
                Style::new()
                    .fg(ls_fg)
                    .bg(ls_bg)
                    .add_modifier(sel_mod(PreviewElement::ListSelected) | Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  src/utils.rs",
                Style::new()
                    .fg(lu_fg)
                    .bg(lu_bg)
                    .add_modifier(sel_mod(PreviewElement::ListUnselected)),
            )),
            Line::from(Span::styled(
                "  README.md",
                Style::new().fg(list_unsel_fg).bg(list_unsel_bg),
            )),
            Line::from(Span::styled("", Style::new().bg(pane_bg))),
            // Exit code pills
            Line::from(vec![
                Span::styled(" ", Style::new().bg(pane_bg)),
                Span::styled(
                    " exit 0 ",
                    Style::new()
                        .fg(eo_fg)
                        .bg(eo_bg)
                        .add_modifier(sel_mod(PreviewElement::ExitSuccess)),
                ),
                Span::styled("  ", Style::new().bg(pane_bg)),
                Span::styled(
                    " exit 1 ",
                    Style::new()
                        .fg(ee_fg)
                        .bg(ee_bg)
                        .add_modifier(sel_mod(PreviewElement::ExitError)),
                ),
            ]),
        ];

        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title(" Pane (Highlight) ")
            .title_style(border_style)
            .border_style(border_style)
            .style(Style::new().bg(pane_bg));

        frame.render_widget(Paragraph::new(content).block(block), area);
    }

    fn render_zellij_status_bar(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let is_editing_status = self.selected_element == PreviewElement::StatusBar;
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
                InputMode::ThemeNameInput | InputMode::ThemeNameInputApply => {
                    (" APPLY  ", sel_fg, sel_bg)
                }
                InputMode::ThemeLoad => (" LOAD   ", sel_fg, sel_bg),
                InputMode::ThemeLoadRename => (" RENAME ", sel_fg, sel_bg),
                InputMode::ThemeLoadDeleteConfirm => (" DELETE ", Color::Rgb(255, 100, 100), Color::Rgb(80, 20, 20)),
                InputMode::UpdateRestartConfirm => (" UPDATE ", sel_fg, sel_bg),
                InputMode::Help => (" HELP   ", sel_fg, sel_bg),
            }
        };

        let pill_full = |key: &str, action: &str| -> Vec<Span<'static>> {
            vec![
                Span::styled("", Style::new().fg(sel_bg).bg(bar_bg)),
                Span::styled(
                    format!(" {} ", key),
                    Style::new().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD),
                ),
                Span::styled("", Style::new().fg(unsel_bg).bg(sel_bg)),
                Span::styled(
                    format!(" {} ", action),
                    Style::new().fg(unsel_fg).bg(unsel_bg),
                ),
                Span::styled("", Style::new().fg(unsel_bg).bg(bar_bg)),
            ]
        };

        let pill_key_only = |key: &str| -> Vec<Span<'static>> {
            vec![
                Span::styled("", Style::new().fg(sel_bg).bg(bar_bg)),
                Span::styled(
                    format!(" {} ", key),
                    Style::new().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD),
                ),
                Span::styled("", Style::new().fg(sel_bg).bg(bar_bg)),
            ]
        };

        let gap = || Span::styled(" ", Style::new().bg(bar_bg));

        // Measure a candidate set of pill spans to get their true rendered width.
        let spans_width = |spans: &[Span]| -> usize { spans.iter().map(|s| s.width()).sum() };

        // (key, long label, short label)
        let bindings: &[(&str, &str, &str)] = match self.input_mode {
            InputMode::Preview => &[
                ("↑↓←→", "NAVIGATE",   "NAV"),
                ("c",    "COLOR",      "CLR"),
                ("U",    "UPDATE",     "UPD"),
                ("s",    "SAVE AS",    "SAVE"),
                ("l",    "LOAD",       "LD"),
                ("a",    "SAVE+APPLY", "APPLY"),
                ("?",    "HELP",       "?"),
                ("q",    "QUIT",       "QT"),
            ],
            InputMode::ColorPicker => {
                const BINDINGS: &[(&str, &str, &str)] = &[
                    ("tab",    "FOCUS",   "TAB"),
                    ("m",      "MODE",    "MD"),
                    ("f",      "FG/BG",   "F/B"),
                    ("drag",   "PICK",    "PK"),
                    ("#",      "HEX",     "HX"),
                    ("Enter",  "EDIT/OK", "OK"),
                    ("Esc",    "CANCEL",  "ESC"),
                ];
                BINDINGS
            }
            InputMode::ThemeNameInput | InputMode::ThemeNameInputApply => &[
                ("type",  "NAME",   "NM"),
                ("Enter", "SAVE",   "OK"),
                ("Esc",   "CANCEL", "ESC"),
            ],
            InputMode::ThemeLoad => &[
                ("↑↓",   "SELECT",  "SEL"),
                ("d",    "DEFAULT", "DEF"),
                ("s",    "SAVED",   "SVD"),
                ("Enter","LOAD",    "LD"),
                ("a",    "APPLY",   "AP"),
                ("r",    "RENAME",  "RN"),
                ("x",    "DELETE",  "DEL"),
                ("Esc",  "CANCEL",  "ESC"),
            ],
            InputMode::ThemeLoadRename => &[
                ("type",  "NAME",   "NM"),
                ("Enter", "RENAME", "OK"),
                ("Esc",   "CANCEL", "ESC"),
            ],
            InputMode::ThemeLoadDeleteConfirm => &[
                ("y", "CONFIRM", "Y"),
                ("n", "CANCEL",  "N"),
            ],
            InputMode::UpdateRestartConfirm => &[
                ("Enter", "RESTART", "OK"),
                ("l",     "LATER",   "L"),
                ("Esc",   "LATER",   "X"),
            ],
            InputMode::Help => &[
                ("↑↓", "SCROLL", "SCR"),
                ("Pg", "JUMP",   "PG"),
                ("Esc", "CLOSE", "X"),
            ],
        };

        // ── Pick label level by measuring actual span widths ──────────────
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
        let right_w = info.chars().count()
            + self.message.as_ref().map(|m| m.chars().count() + 3).unwrap_or(0);

        // mode pill spans (built once, measured)
        let mode_pill_spans: Vec<Span> = vec![
            Span::styled("", Style::new().fg(mode_bg).bg(bar_bg)),
            Span::styled(mode_label, Style::new().fg(mode_fg).bg(mode_bg).add_modifier(Modifier::BOLD)),
            Span::styled("", Style::new().fg(mode_bg).bg(bar_bg)),
            gap(),
        ];
        let mode_pill_w = spans_width(&mode_pill_spans);

        let measure_pills = |label_idx: usize| -> usize {
            bindings.iter().map(|(k, l, s)| {
                let action = if label_idx == 0 { l } else { s };
                let mut w = spans_width(&pill_full(k, action));
                w += gap().width();
                w
            }).sum()
        };
        let measure_keys_only = || -> usize {
            bindings.iter().map(|(k, _, _)| {
                spans_width(&pill_key_only(k)) + gap().width()
            }).sum()
        };

        let available = (area.width as usize).saturating_sub(mode_pill_w + right_w);
        let level = if measure_pills(0) <= available { 0 }
                    else if measure_pills(1) <= available { 1 }
                    else { 2 };
        let _ = measure_keys_only; // silence unused warning

        let mut spans: Vec<Span> = Vec::new();

        // ── Mode pill ─────────────────────────────────────────────────────
        spans.push(Span::styled("", Style::new().fg(mode_bg).bg(bar_bg)));
        spans.push(Span::styled(
            mode_label,
            Style::new().fg(mode_fg).bg(mode_bg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("", Style::new().fg(mode_bg).bg(bar_bg)));
        spans.push(gap());

        // ── Keybinding pills ──────────────────────────────────────────────
        for (key, long, short) in bindings {
            match level {
                0 => spans.extend(pill_full(key, long)),
                1 => spans.extend(pill_full(key, short)),
                _ => spans.extend(pill_key_only(key)),
            }
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
        let fill = (area.width as usize).saturating_sub(used + right_w);
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
        } else {
            use crate::ui::state::UpdateStatus;
            let (update_text, update_color, bold) = match &self.update_status {
                UpdateStatus::Checking    => (Some(" checking for update… ".to_string()),  Color::DarkGray, false),
                UpdateStatus::Available(v) => (Some(format!(" ↑ {} available — press U ", v)), Color::Yellow,   true),
                UpdateStatus::Downloading => (Some(" downloading update… ".to_string()),   Color::Yellow,   false),
                UpdateStatus::Done        => (Some(" ✓ updated — restart to apply ".to_string()), Color::Green, true),
                UpdateStatus::Failed(e)   => (Some(format!(" update check failed: {} ", e)), Color::Red,    false),
                _                         => (None, Color::Reset, false),
            };
            if let Some(text) = update_text {
                let mut style = Style::new().fg(update_color).bg(bar_bg);
                if bold { style = style.add_modifier(Modifier::BOLD); }
                spans.push(Span::styled(text, style));
            }
        }

        spans.push(Span::styled(
            info,
            Style::new()
                .fg(Color::Rgb(167, 139, 250))
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
        const OB_BG: Color = Color::Rgb(22, 22, 26);
        const OB_BORDER: Color = Color::Rgb(90, 85, 115);
        const OB_TEXT: Color = Color::Rgb(212, 212, 230);
        const OB_MUTED: Color = Color::Rgb(120, 120, 145);
        const OB_DIM: Color = Color::Rgb(84, 84, 104);
        const ACCENT_BG: Color = Color::Rgb(97, 88, 150);
        const ACCENT_FG: Color = Color::Rgb(242, 240, 255);
        const SUBTLE_BG: Color = Color::Rgb(54, 50, 74);
        const SUBTLE_FG: Color = Color::Rgb(214, 210, 235);
        const SURFACE_BG: Color = Color::Rgb(26, 26, 32);
        const SURFACE_FOCUS_BG: Color = Color::Rgb(34, 31, 46);

        let rects = picker_layout(frame.area(), self.color_editor.mode);
        frame.render_widget(Clear, rects.overlay);

        let outer = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(OB_BORDER))
            .style(Style::new().bg(OB_BG));
        let inner = outer.inner(rects.overlay);
        frame.render_widget(outer, rects.overlay);

        let [header, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(16),
            Constraint::Length(2),
        ])
        .areas(inner);
        let [main_col, side_col] =
            Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)]).areas(body);
        let [preview_area, _fields_area] =
            Layout::vertical([Constraint::Length(5), Constraint::Fill(1)]).areas(side_col);

        let current_rgb = self.color_editor.to_rgb();
        let current_hex = self.color_editor.hex();
        let hsl = self.color_editor.hsl;
        let hsv = self.color_editor.hsv();

        let original_rgb = self.original_component.as_ref().map(|orig| match self.selected_attribute {
            PreviewAttribute::Base => orig.base,
            PreviewAttribute::Background => orig.background,
        });

        let mk_pill = |key: &str, label: &str, active: bool| -> Vec<Span<'static>> {
            let (key_bg, key_fg, lbl_bg, lbl_fg) = if active {
                (ACCENT_BG, ACCENT_FG, SUBTLE_BG, SUBTLE_FG)
            } else {
                (SUBTLE_BG, SUBTLE_FG, OB_BG, OB_MUTED)
            };
            vec![
                Span::styled("", Style::new().fg(key_bg).bg(OB_BG)),
                Span::styled(
                    format!(" {} ", key),
                    Style::new().fg(key_fg).bg(key_bg).add_modifier(Modifier::BOLD),
                ),
                Span::styled("", Style::new().fg(lbl_bg).bg(key_bg)),
                Span::styled(format!(" {} ", label), Style::new().fg(lbl_fg).bg(lbl_bg)),
                Span::styled("", Style::new().fg(lbl_bg).bg(OB_BG)),
            ]
        };

        let mut header_spans = vec![Span::styled(
            " Color Picker ",
            Style::new().fg(OB_TEXT).add_modifier(Modifier::BOLD),
        )];
        let mode_focused = self.color_editor.focus == ColorPickerFocus::ModeToggle;
        let mode_pills = [
            if mode_focused {
                vec![Span::styled("› ", Style::new().fg(ACCENT_FG).add_modifier(Modifier::BOLD))]
            } else {
                vec![]
            },
            mk_pill(
                "M",
                "rgb",
                self.color_editor.mode == ColorPickerMode::RgbSliders,
            ),
            vec![Span::raw(" ")],
            mk_pill(
                "M",
                "hsl",
                self.color_editor.mode == ColorPickerMode::HslField,
            ),
            if mode_focused {
                vec![Span::styled(" ‹", Style::new().fg(ACCENT_FG).add_modifier(Modifier::BOLD))]
            } else {
                vec![]
            },
        ]
        .concat();
        let right_pill = {
            let attr = self.selected_attribute.label();
            let name = self.selected_element.label();
            vec![
                Span::styled("", Style::new().fg(ACCENT_BG).bg(OB_BG)),
                Span::styled(
                    format!(" {} ", attr),
                    Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD),
                ),
                Span::styled("", Style::new().fg(SUBTLE_BG).bg(ACCENT_BG)),
                Span::styled(format!(" {} ", name), Style::new().fg(SUBTLE_FG).bg(SUBTLE_BG)),
                Span::styled("", Style::new().fg(SUBTLE_BG).bg(OB_BG)),
            ]
        };
        let left_w: usize = header_spans.iter().map(|s| s.width()).sum::<usize>()
            + mode_pills.iter().map(|s| s.width()).sum::<usize>()
            + 1;
        let right_w: usize = right_pill.iter().map(|s| s.width()).sum();
        let gap = header.width as usize - left_w - right_w;
        header_spans.push(Span::raw(" "));
        header_spans.extend(mode_pills);
        header_spans.push(Span::styled(" ".repeat(gap.max(1)), Style::new().bg(OB_BG)));
        header_spans.extend(right_pill);
        frame.render_widget(
            Paragraph::new(Line::from(header_spans)).style(Style::new().bg(OB_BG)),
            header,
        );

        frame.render_widget(
            Block::default().style(Style::new().bg(SURFACE_BG)),
            main_col,
        );
        frame.render_widget(
            Block::default().style(Style::new().bg(OB_BG)),
            side_col,
        );

        match self.color_editor.mode {
            ColorPickerMode::RgbSliders => {
                let channels_focus = matches!(self.color_editor.focus, ColorPickerFocus::RgbSlider(_));
                let channels_block = Block::bordered()
                    .title(" Channels ")
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(if channels_focus { ACCENT_BG } else { OB_BORDER }))
                    .style(Style::new().bg(SURFACE_BG));
                let channels_inner = channels_block.inner(rects.main_view);
                frame.render_widget(channels_block, rects.main_view);
                let slider_width = channels_inner.width.saturating_sub(8) as usize;
                let labels = ["R", "G", "B"];
                for (idx, label) in labels.into_iter().enumerate() {
                    let row_rect = Rect {
                        x: channels_inner.x,
                        y: channels_inner.y + (idx as u16 * 2),
                        width: channels_inner.width,
                        height: 1,
                    };
                    let value = self.color_editor.rgb[idx];
                    let filled = ((value as f32 / 255.0) * slider_width as f32).round() as usize;
                    let bar: String = (0..slider_width)
                        .map(|i| if i < filled { '█' } else { '░' })
                        .collect();
                    let is_focus = self.color_editor.focus == ColorPickerFocus::RgbSlider(idx);
                    let color = match idx {
                        0 => Color::Rgb(255, 96, 96),
                        1 => Color::Rgb(106, 220, 124),
                        _ => Color::Rgb(102, 186, 255),
                    };
                    let line = Line::from(vec![
                        Span::styled(
                            format!(" {} ", label),
                            Style::new()
                                .fg(if is_focus { ACCENT_FG } else { OB_TEXT })
                                .bg(if is_focus { ACCENT_BG } else { SURFACE_BG })
                                .add_modifier(if is_focus { Modifier::BOLD } else { Modifier::empty() }),
                        ),
                        Span::styled(bar, Style::new().fg(color).bg(SURFACE_BG)),
                        Span::styled(
                            format!(" {:>3}", value),
                            Style::new().fg(if is_focus { OB_TEXT } else { OB_MUTED }),
                        ),
                    ]);
                    frame.render_widget(Paragraph::new(line).style(Style::new().bg(SURFACE_BG)), row_rect);
                }
                frame.render_widget(
                    Paragraph::new(vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            " RGB sliders for exact channel edits",
                            Style::new().fg(OB_TEXT).add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            " Press M to switch to the HSL field picker.",
                            Style::new().fg(OB_MUTED),
                        )),
                        Line::from(""),
                        Line::from(Span::styled(
                            format!(" Current  {}", current_hex),
                            Style::new().fg(OB_TEXT),
                        )),
                        Line::from(Span::styled(
                            format!(" HSV {:.0} / {:.0}% / {:.0}%", hsv.hue, hsv.saturation, hsv.value),
                            Style::new().fg(OB_MUTED),
                        )),
                    ])
                    .style(Style::new().bg(SURFACE_BG)),
                    Rect {
                        x: channels_inner.x,
                        y: channels_inner.y + 6,
                        width: channels_inner.width,
                        height: channels_inner.height.saturating_sub(6),
                    },
                );
            }
            ColorPickerMode::HslField => {
                let field_focus = self.color_editor.focus == ColorPickerFocus::HslField;
                let field_block = Block::bordered()
                    .title(if field_focus { " Color Field ● " } else { " Color Field " })
                    .title_style(
                        Style::new()
                            .fg(if field_focus { ACCENT_FG } else { OB_MUTED })
                            .add_modifier(if field_focus { Modifier::BOLD } else { Modifier::empty() }),
                    )
                    .border_type(if field_focus { BorderType::Double } else { BorderType::Rounded })
                    .border_style(Style::new().fg(if field_focus { ACCENT_BG } else { OB_BORDER }))
                    .style(Style::new().bg(if field_focus { SURFACE_FOCUS_BG } else { SURFACE_BG }));
                let field_area = field_block.inner(rects.main_view);
                frame.render_widget(field_block, rects.main_view);
                for row in 0..field_area.height {
                    let mut spans = Vec::with_capacity(field_area.width as usize);
                    for col in 0..field_area.width {
                        let x_frac = col as f32 / field_area.width.saturating_sub(1).max(1) as f32;
                        let top_frac = (row as f32 * 2.0) / (field_area.height.max(1) as f32 * 2.0 - 1.0);
                        let bottom_frac = ((row as f32 * 2.0) + 1.0)
                            / (field_area.height.max(1) as f32 * 2.0 - 1.0);
                        let top = hsv_field_cell(x_frac * 360.0, (1.0 - top_frac) * 100.0, hsv.value);
                        let bottom =
                            hsv_field_cell(x_frac * 360.0, (1.0 - bottom_frac) * 100.0, hsv.value);
                        let selected_col = ((hsv.hue / 360.0)
                            * field_area.width.saturating_sub(1).max(1) as f32)
                            .round() as u16;
                        let selected_row = (((100.0 - hsv.saturation) / 100.0)
                            * field_area.height.saturating_sub(1).max(1) as f32)
                            .round() as u16;
                        if col == selected_col && row == selected_row {
                            let marker = contrast_text(current_rgb);
                            spans.push(Span::styled(
                                "◉",
                                Style::new().fg(tui_rgb(marker)).bg(tui_rgb(current_rgb)).add_modifier(Modifier::BOLD),
                            ));
                        } else {
                            spans.push(Span::styled(
                                "▀",
                                Style::new().fg(tui_rgb(top)).bg(tui_rgb(bottom)),
                            ));
                        }
                    }
                    frame.render_widget(
                        Paragraph::new(Line::from(spans))
                            .style(Style::new().bg(if field_focus { SURFACE_FOCUS_BG } else { SURFACE_BG })),
                        Rect {
                            x: field_area.x,
                            y: field_area.y + row,
                            width: field_area.width,
                            height: 1,
                        },
                    );
                }
                let value_focus = self.color_editor.focus == ColorPickerFocus::LightnessSlider;
                let value_block = Block::bordered()
                    .title(if value_focus { " V ● " } else { " V " })
                    .title_style(
                        Style::new()
                            .fg(if value_focus { ACCENT_FG } else { OB_MUTED })
                            .add_modifier(if value_focus { Modifier::BOLD } else { Modifier::empty() }),
                    )
                    .border_type(if value_focus { BorderType::Double } else { BorderType::Rounded })
                    .border_style(Style::new().fg(if value_focus { ACCENT_BG } else { OB_BORDER }))
                    .style(Style::new().bg(if value_focus { SURFACE_FOCUS_BG } else { SURFACE_BG }));
                let value_area = value_block.inner(rects.aux_slider);
                frame.render_widget(value_block, rects.aux_slider);
                let selected_row = (((100.0 - hsv.value) / 100.0)
                    * value_area.height.saturating_sub(1).max(1) as f32)
                    .round() as u16;
                for row in 0..value_area.height {
                    let top_frac = (row as f32 * 2.0) / (value_area.height.max(1) as f32 * 2.0 - 1.0);
                    let bottom_frac = ((row as f32 * 2.0) + 1.0)
                        / (value_area.height.max(1) as f32 * 2.0 - 1.0);
                    let top_value = (1.0 - top_frac.clamp(0.0, 1.0)) * 100.0;
                    let bottom_value = (1.0 - bottom_frac.clamp(0.0, 1.0)) * 100.0;
                    let top_color = hsv_field_cell(hsv.hue, hsv.saturation, top_value);
                    let bottom_color = hsv_field_cell(hsv.hue, hsv.saturation, bottom_value);
                    let selected = row == selected_row;
                    let indicator_color = bottom_color;
                    let indicator_fg = contrast_text(indicator_color);
                    let content = if selected {
                        "█".repeat(value_area.width as usize)
                    } else {
                        "▀".repeat(value_area.width as usize)
                    };
                    let style = if selected {
                        Style::new()
                            .fg(tui_rgb(indicator_fg))
                            .bg(tui_rgb(indicator_color))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(tui_rgb(top_color)).bg(tui_rgb(bottom_color))
                    };
                    frame.render_widget(
                        Paragraph::new(Line::from(vec![Span::styled(content, style)])),
                        Rect {
                            x: value_area.x,
                            y: value_area.y + row,
                            width: value_area.width,
                            height: 1,
                        },
                    );
                }
            }
        }

        let preview_lines = {
            let current_fg = tui_rgb(contrast_text(current_rgb));
            let before_line = if let Some(orig) = original_rgb {
                Line::from(vec![
                    Span::styled("      ", Style::new().bg(tui_rgb(orig))),
                    Span::styled("  →  ", Style::new().fg(OB_DIM).bg(OB_BG)),
                    Span::styled("      ", Style::new().bg(tui_rgb(current_rgb))),
                ])
            } else {
                Line::from(vec![Span::styled("      ", Style::new().bg(tui_rgb(current_rgb)))])
            };
            vec![
                Line::from(Span::styled(
                    format!(" {}", current_hex),
                    Style::new().fg(OB_TEXT).add_modifier(Modifier::BOLD),
                )),
                before_line,
                Line::from(Span::styled(
                    format!(" rgb {} {} {}", current_rgb.r, current_rgb.g, current_rgb.b),
                    Style::new().fg(OB_MUTED),
                )),
                Line::from(Span::styled(
                    format!(" hsl {:.0} {:.0}% {:.0}%", hsl.hue, hsl.saturation, hsl.lightness),
                    Style::new().fg(OB_MUTED),
                )),
                Line::from(Span::styled(
                    format!(" hsv {:.0} {:.0}% {:.0}%", hsv.hue, hsv.saturation, hsv.value),
                    Style::new().fg(current_fg),
                )),
                Line::from(Span::styled(
                    format!(" focus {}", self.color_editor.focus_label()),
                    Style::new().fg(ACCENT_FG),
                )),
            ]
        };
        frame.render_widget(
            Paragraph::new(preview_lines).style(Style::new().bg(OB_BG)),
            preview_area,
        );

        let mut field_block = |rect: Rect, title: &str, focused: bool| {
            let border = if focused { ACCENT_BG } else { OB_BORDER };
            frame.render_widget(
                Block::bordered()
                    .title(format!(" {} ", title))
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(border))
                    .style(Style::new().bg(OB_BG)),
                rect,
            );
        };
        field_block(
            rects.hex_field,
            "HEX",
            self.color_editor.focus == ColorPickerFocus::HexField,
        );
        for (idx, rect) in rects.rgb_fields.iter().enumerate() {
            field_block(
                *rect,
                ["R", "G", "B"][idx],
                self.color_editor.focus == ColorPickerFocus::RgbField(idx),
            );
        }
        for (idx, rect) in rects.hsl_fields.iter().enumerate() {
            field_block(
                *rect,
                ["H", "S", "L"][idx],
                self.color_editor.focus == ColorPickerFocus::HslFieldValue(idx),
            );
        }
        let render_field_value = |frame: &mut Frame, rect: Rect, value: String, suffix: &str, editing: bool| {
            let inner = Rect {
                x: rect.x + 1,
                y: rect.y + 1,
                width: rect.width.saturating_sub(2),
                height: rect.height.saturating_sub(2),
            };
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        value,
                        Style::new().fg(if editing { ACCENT_FG } else { OB_TEXT }).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(suffix.to_string(), Style::new().fg(OB_MUTED)),
                ]))
                .style(Style::new().bg(OB_BG)),
                inner,
            );
        };
        render_field_value(
            frame,
            rects.hex_field,
            self.color_editor.field_value(EditableField::Hex),
            "",
            matches!(
                self.color_editor.text_edit.as_ref().map(|edit| edit.target),
                Some(EditableField::Hex)
            ),
        );
        for idx in 0..3 {
            render_field_value(
                frame,
                rects.rgb_fields[idx],
                self.color_editor.field_value(EditableField::Rgb(idx)),
                "",
                matches!(
                    self.color_editor.text_edit.as_ref().map(|edit| edit.target),
                    Some(EditableField::Rgb(i)) if i == idx
                ),
            );
        }
        for idx in 0..3 {
            render_field_value(
                frame,
                rects.hsl_fields[idx],
                self.color_editor.field_value(EditableField::Hsl(idx)),
                if idx == 0 { "°" } else { "%" },
                matches!(
                    self.color_editor.text_edit.as_ref().map(|edit| edit.target),
                    Some(EditableField::Hsl(i)) if i == idx
                ),
            );
        }

        let footer_lines = vec![
            Line::from(vec![
                Span::styled(" Tab ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" focus  ", Style::new().fg(OB_MUTED)),
                Span::styled(" M ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" switch  ", Style::new().fg(OB_MUTED)),
                Span::styled(" F ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" fg/bg  ", Style::new().fg(OB_MUTED)),
                Span::styled(" Enter ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" edit/keep", Style::new().fg(OB_MUTED)),
            ]),
            Line::from(vec![
                Span::styled(" Mouse ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" drag  ", Style::new().fg(OB_MUTED)),
                Span::styled(" # ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" hex  ", Style::new().fg(OB_MUTED)),
                Span::styled(" Esc ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" cancel", Style::new().fg(OB_MUTED)),
            ]),
        ];
        frame.render_widget(
            Paragraph::new(footer_lines).style(Style::new().bg(OB_BG)),
            footer,
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
                Span::styled(" Name: ", Style::new().fg(Color::Rgb(167, 139, 250))),
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
            .title_style(Style::new().fg(Color::Rgb(167, 139, 250)).add_modifier(Modifier::BOLD))
            .border_style(Style::new().fg(Color::Rgb(90, 85, 115)))
            .style(Style::new().bg(Color::Rgb(22, 22, 26)));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(Color::Rgb(22, 22, 26))),
            area,
        );
    }

    fn render_help_overlay(&self, frame: &mut Frame) {
        const PANEL_BG: Color = Color::Rgb(22, 22, 26);
        const PANEL_BORDER: Color = Color::Rgb(90, 85, 115);
        const PANEL_MUTED: Color = Color::Rgb(120, 120, 145);
        const PANEL_TEXT: Color = Color::Rgb(212, 212, 230);
        const ACCENT_BG: Color = Color::Rgb(97, 88, 150);
        const ACCENT_FG: Color = Color::Rgb(242, 240, 255);
        const SUBTLE_BG: Color = Color::Rgb(54, 50, 74);
        const SUBTLE_FG: Color = Color::Rgb(214, 210, 235);
        const DIVIDER: Color = Color::Rgb(50, 48, 64);

        enum HelpRow<'a> {
            Section(&'a str),
            Entry(&'a str, &'a str),
            Spacer,
        }

        let rows: &[HelpRow<'_>] = &[
            HelpRow::Section("Preview"),
            HelpRow::Entry("↑/j  ↓/k  ← →", "Navigate preview elements"),
            HelpRow::Entry("Tab", "Toggle FG / BG (pane borders: FG only)"),
            HelpRow::Entry("c", "Open color picker for selected color"),
            HelpRow::Entry("y", "Yank (copy) current color"),
            HelpRow::Entry("p", "Paste yanked color"),
            HelpRow::Entry("u", "Undo last color change"),
            HelpRow::Entry("s", "Save theme as… (prompts for name)"),
            HelpRow::Entry("l", "Open theme loader"),
            HelpRow::Entry("a", "Apply current theme to Zellij config"),
            HelpRow::Entry("U", "Install latest released binary on Linux x86_64 when available"),
            HelpRow::Entry("?", "Toggle this help screen"),
            HelpRow::Entry("q / Esc", "Quit"),
            HelpRow::Spacer,
            HelpRow::Section("Color Picker"),
            HelpRow::Entry("Tab / Shift+Tab", "Move focus between controls"),
            HelpRow::Entry("m", "Switch RGB sliders / HSL field"),
            HelpRow::Entry("f", "Toggle FG / BG (non-pane)"),
            HelpRow::Entry("Mouse drag", "Drag in HSL field or lightness slider"),
            HelpRow::Entry("← → ↑ ↓", "Nudge the focused control"),
            HelpRow::Entry("Shift / Alt", "Coarse / fine nudging"),
            HelpRow::Entry("Enter", "Edit the focused value field or confirm"),
            HelpRow::Entry("#", "Jump to hex field editing"),
            HelpRow::Entry("Esc", "Cancel"),
            HelpRow::Spacer,
            HelpRow::Section("Theme Loader"),
            HelpRow::Entry("type", "Search and filter themes"),
            HelpRow::Entry("Enter / ↓", "Commit search and move into results"),
            HelpRow::Entry("↑ ↓", "Navigate themes"),
            HelpRow::Entry("Enter", "Load selected theme into editor"),
            HelpRow::Entry("a", "Apply selected theme to Zellij"),
            HelpRow::Entry("d", "Filter built-in themes"),
            HelpRow::Entry("s", "Filter saved themes"),
            HelpRow::Entry("r", "Rename selected saved theme"),
            HelpRow::Entry("x", "Delete selected saved theme"),
            HelpRow::Entry("Esc", "Clear search or cancel"),
        ];

        let overlay_h = 24u16.min(frame.area().height.saturating_sub(2));
        let overlay_w = 84u16.min(frame.area().width.saturating_sub(4));
        let area = centered_rect(frame.area(), overlay_w, overlay_h);
        frame.render_widget(Clear, area);

        let outer_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(PANEL_BORDER))
            .style(Style::new().bg(PANEL_BG));
        let inner = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        let [header_rect, body_rect, footer_rect] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .areas(inner);

        let title = vec![Span::styled(
            " Help ",
            Style::new().fg(PANEL_TEXT).add_modifier(Modifier::BOLD),
        )];
        let scroll_label = format!("{:>2} rows", rows.len());
        let mut header_spans = title;
        let title_w: usize = header_spans.iter().map(|span| span.width()).sum();
        let right_spans = vec![Span::styled(scroll_label, Style::new().fg(PANEL_MUTED))];
        let right_w: usize = right_spans.iter().map(|span| span.width()).sum();
        let gap = (header_rect.width as usize).saturating_sub(title_w + right_w);
        header_spans.push(Span::styled(" ".repeat(gap), Style::new().bg(PANEL_BG)));
        header_spans.extend(right_spans);
        frame.render_widget(
            Paragraph::new(Line::from(header_spans)).style(Style::new().bg(PANEL_BG)),
            Rect {
                x: header_rect.x,
                y: header_rect.y,
                width: header_rect.width,
                height: 1,
            },
        );

        let header_hint = vec![
            Span::styled(" ↑↓ ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
            Span::styled(" scroll  ", Style::new().fg(PANEL_MUTED)),
            Span::styled(" PgUp/PgDn ", Style::new().fg(SUBTLE_FG).bg(SUBTLE_BG).add_modifier(Modifier::BOLD)),
            Span::styled(" jump", Style::new().fg(PANEL_MUTED)),
        ];
        frame.render_widget(
            Paragraph::new(Line::from(header_hint)).style(Style::new().bg(PANEL_BG)),
            Rect {
                x: header_rect.x,
                y: header_rect.y + 1,
                width: header_rect.width,
                height: 1,
            },
        );

        let [list_rect, detail_rect] =
            Layout::horizontal([Constraint::Percentage(66), Constraint::Percentage(34)]).areas(body_rect);

        frame.render_widget(
            Paragraph::new("").block(
                Block::default()
                    .borders(ratatui::widgets::Borders::RIGHT)
                    .border_style(Style::new().fg(DIVIDER)),
            ),
            list_rect,
        );

        let list_inner = Rect {
            x: list_rect.x,
            y: list_rect.y,
            width: list_rect.width.saturating_sub(1),
            height: list_rect.height,
        };
        let detail_inner = Rect {
            x: detail_rect.x + 1,
            y: detail_rect.y,
            width: detail_rect.width.saturating_sub(2),
            height: detail_rect.height,
        };

        let visible_rows = list_inner.height as usize;
        let max_scroll = rows.len().saturating_sub(visible_rows) as u16;
        let scroll = self.help_scroll.min(max_scroll);

        let list_lines: Vec<Line> = rows
            .iter()
            .map(|row| match row {
                HelpRow::Section(title) => Line::from(vec![
                    Span::styled(" ", Style::new().bg(PANEL_BG)),
                    Span::styled(
                        format!(" {} ", title),
                        Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD),
                    ),
                ]),
                HelpRow::Entry(key, desc) => Line::from(vec![
                    Span::styled(" ", Style::new().bg(PANEL_BG)),
                    Span::styled(format!("{:<18}", key), Style::new().fg(PANEL_TEXT).add_modifier(Modifier::BOLD)),
                    Span::styled(desc.to_string(), Style::new().fg(PANEL_MUTED)),
                ]),
                HelpRow::Spacer => Line::from(""),
            })
            .collect();

        frame.render_widget(
            Paragraph::new(list_lines)
                .scroll((scroll, 0))
                .style(Style::new().bg(PANEL_BG)),
            list_inner,
        );

        let detail_lines = vec![
            Line::from(Span::styled(
                " Navigation",
                Style::new().fg(PANEL_MUTED).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                " Use cursor keys or j/k to scroll this help view.",
                Style::new().fg(PANEL_TEXT),
            )),
            Line::from(Span::styled(
                " Press uppercase U in preview mode to update when the status bar says one is available.",
                Style::new().fg(PANEL_MUTED),
            )),
            Line::from(Span::styled(
                " PgUp/PgDn moves faster. Home/End jumps to top or bottom.",
                Style::new().fg(PANEL_MUTED),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Overlay Style",
                Style::new().fg(PANEL_MUTED).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                " This panel now uses the same rounded frame, accent pills, and split layout as the picker and loader.",
                Style::new().fg(PANEL_TEXT),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Scroll",
                Style::new().fg(PANEL_MUTED).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(" Showing from row {} of {}", scroll.saturating_add(1), rows.len()),
                Style::new().fg(if max_scroll > 0 { ACCENT_FG } else { PANEL_TEXT }),
            )),
        ];
        frame.render_widget(
            Paragraph::new(detail_lines).style(Style::new().bg(PANEL_BG)),
            detail_inner,
        );

        let footer_spans = vec![
            Span::styled(" ↑↓ ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
            Span::styled(" scroll  ", Style::new().fg(PANEL_MUTED)),
            Span::styled(" Home/End ", Style::new().fg(SUBTLE_FG).bg(SUBTLE_BG).add_modifier(Modifier::BOLD)),
            Span::styled(" jump  ", Style::new().fg(PANEL_MUTED)),
            Span::styled(" Esc ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
            Span::styled(" close", Style::new().fg(PANEL_MUTED)),
        ];
        frame.render_widget(
            Paragraph::new(Line::from(footer_spans)).style(Style::new().bg(PANEL_BG)),
            footer_rect,
        );
    }

    fn render_update_restart_overlay(&self, frame: &mut Frame) {
        const PANEL_BG: Color = Color::Rgb(22, 22, 26);
        const PANEL_BORDER: Color = Color::Rgb(90, 85, 115);
        const PANEL_MUTED: Color = Color::Rgb(120, 120, 145);
        const PANEL_TEXT: Color = Color::Rgb(212, 212, 230);
        const ACCENT_BG: Color = Color::Rgb(97, 88, 150);
        const ACCENT_FG: Color = Color::Rgb(242, 240, 255);
        const SUBTLE_BG: Color = Color::Rgb(54, 50, 74);
        const SUBTLE_FG: Color = Color::Rgb(214, 210, 235);

        let area = centered_rect(frame.area(), 58, 10);
        frame.render_widget(Clear, area);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Update Installed ")
            .title_style(Style::new().fg(ACCENT_FG).add_modifier(Modifier::BOLD))
            .border_style(Style::new().fg(PANEL_BORDER))
            .style(Style::new().bg(PANEL_BG));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines = vec![
            Line::from(Span::styled(
                " The new build is ready.",
                Style::new().fg(PANEL_TEXT).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Restart now to launch the updated binary, or choose later and keep working.",
                Style::new().fg(PANEL_MUTED),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " Enter ",
                    Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" restart now  ", Style::new().fg(PANEL_MUTED)),
                Span::styled(
                    " L ",
                    Style::new().fg(SUBTLE_FG).bg(SUBTLE_BG).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" later", Style::new().fg(PANEL_MUTED)),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(lines).style(Style::new().bg(PANEL_BG)),
            inner,
        );
    }

    fn render_theme_load_overlay(&self, frame: &mut Frame) {
        let overlay_h = 22u16.min(frame.area().height.saturating_sub(2));
        let overlay_w = 80u16.min(frame.area().width.saturating_sub(4));
        let area = centered_rect(frame.area(), overlay_w, overlay_h);
        frame.render_widget(Clear, area);

        const PANEL_BG: Color = Color::Rgb(22, 22, 26);
        const PANEL_BORDER: Color = Color::Rgb(90, 85, 115);
        const PANEL_MUTED: Color = Color::Rgb(120, 120, 145);
        const PANEL_TEXT: Color = Color::Rgb(212, 212, 230);
        const PANEL_DIM: Color = Color::Rgb(84, 84, 104);
        const ACCENT_BG: Color = Color::Rgb(97, 88, 150);
        const ACCENT_FG: Color = Color::Rgb(242, 240, 255);
        const SUBTLE_BG: Color = Color::Rgb(54, 50, 74);
        const SUBTLE_FG: Color = Color::Rgb(214, 210, 235);

        let outer_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(PANEL_BORDER))
            .style(Style::new().bg(PANEL_BG));
        let inner = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        use crate::ui::state::ThemeFilter;

        let title_spans = vec![
            Span::styled(
                " Themes ",
                Style::new().fg(PANEL_TEXT).add_modifier(Modifier::BOLD),
            ),
        ];
        let count_label = format!("{:>2} matches", self.loadable_themes.len());
        let count_spans = vec![Span::styled(count_label, Style::new().fg(PANEL_MUTED))];
        let title_w: usize = title_spans.iter().map(|s| s.width()).sum();
        let count_w: usize = count_spans.iter().map(|s| s.width()).sum();
        let gap = (inner.width as usize).saturating_sub(title_w + count_w);
        let mut header_spans = title_spans;
        header_spans.push(Span::styled(" ".repeat(gap), Style::new().bg(PANEL_BG)));
        header_spans.extend(count_spans);

        let header_rect = Rect { x: inner.x, y: inner.y, width: inner.width, height: 1 };
        frame.render_widget(
            Paragraph::new(Line::from(header_spans)).style(Style::new().bg(PANEL_BG)),
            header_rect,
        );

        let display_query = self.theme_search_query.trim_start_matches('/');
        let search_value = if display_query.is_empty() {
            String::from("(type to filter)")
        } else if self.search_focused {
            format!("{}_", display_query)
        } else {
            display_query.to_string()
        };
        let search_fg = if display_query.is_empty() {
            PANEL_DIM
        } else {
            PANEL_TEXT
        };
        let search_spans = vec![
            Span::styled(" Search ", Style::new().fg(PANEL_MUTED)),
            Span::styled(format!("/ {}", search_value), Style::new().fg(search_fg)),
        ];
        let search_rect = Rect { x: inner.x, y: inner.y + 1, width: inner.width, height: 1 };
        frame.render_widget(
            Paragraph::new(Line::from(search_spans)).style(Style::new().bg(PANEL_BG)),
            search_rect,
        );

        let active_name = self.original_theme.as_ref().map(|t| t.name.clone());
        let selected_entry = self.loadable_themes.get(self.selected_theme_index);

        let filter_pill = |key: &str, label: &str, active: bool| -> Vec<Span<'static>> {
            let (key_bg, key_fg, lbl_bg, lbl_fg) = if active {
                (ACCENT_BG, ACCENT_FG, SUBTLE_BG, SUBTLE_FG)
            } else {
                (SUBTLE_BG, SUBTLE_FG, PANEL_BG, PANEL_MUTED)
            };
            vec![
                Span::styled("", Style::new().fg(key_bg).bg(PANEL_BG)),
                Span::styled(
                    format!(" {} ", key),
                    Style::new().fg(key_fg).bg(key_bg).add_modifier(Modifier::BOLD),
                ),
                Span::styled("", Style::new().fg(lbl_bg).bg(key_bg)),
                Span::styled(
                    format!(" {} ", label),
                    Style::new().fg(lbl_fg).bg(lbl_bg),
                ),
                Span::styled("", Style::new().fg(lbl_bg).bg(PANEL_BG)),
                Span::raw(" "),
            ]
        };

        let mut filter_spans = filter_pill("A", "all", matches!(self.theme_filter, ThemeFilter::All));
        filter_spans.extend(filter_pill("D", "built-in", matches!(self.theme_filter, ThemeFilter::Builtin)));
        filter_spans.extend(filter_pill("S", "saved", matches!(self.theme_filter, ThemeFilter::Saved)));
        let filter_w: usize = filter_spans.iter().map(|s| s.width()).sum();
        let live_label = vec![Span::styled(" live preview ", Style::new().fg(PANEL_MUTED))];
        let live_w: usize = live_label.iter().map(|s| s.width()).sum();
        let gap = (inner.width as usize).saturating_sub(filter_w + live_w);
        filter_spans.push(Span::styled(" ".repeat(gap), Style::new().bg(PANEL_BG)));
        filter_spans.extend(live_label);

        let filter_rect = Rect { x: inner.x, y: inner.y + 2, width: inner.width, height: 1 };
        frame.render_widget(
            Paragraph::new(Line::from(filter_spans)).style(Style::new().bg(PANEL_BG)),
            filter_rect,
        );

        let body_rect = Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: inner.height.saturating_sub(4),
        };
        let [list_rect, detail_rect] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(body_rect);

        frame.render_widget(
            Paragraph::new("").block(
                Block::default()
                    .borders(ratatui::widgets::Borders::RIGHT)
                    .border_style(Style::new().fg(Color::Rgb(50, 48, 64))),
            ),
            list_rect,
        );

        let list_inner = Rect {
            x: list_rect.x,
            y: list_rect.y,
            width: list_rect.width.saturating_sub(1),
            height: list_rect.height,
        };

        let cards_per_view = list_inner.height.saturating_sub(1) as usize;

        let scroll = if self.loadable_themes.len() <= cards_per_view {
            0
        } else if self.selected_theme_index < cards_per_view / 2 {
            0
        } else {
            (self.selected_theme_index.saturating_sub(cards_per_view / 2))
                .min(self.loadable_themes.len().saturating_sub(cards_per_view))
        };

        if self.loadable_themes.is_empty() {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(" No themes match ", Style::new().fg(PANEL_TEXT).add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!(" \"/{}\"", display_query),
                        Style::new().fg(PANEL_MUTED),
                    )),
                ])
                .style(Style::new().bg(PANEL_BG)),
                list_inner,
            );
        } else {
            for i in scroll..self.loadable_themes.len() {
                let row_y = list_inner.y + (i - scroll) as u16;
                if row_y >= list_inner.y + list_inner.height {
                    break;
                }
                let row_rect = Rect {
                    x: list_inner.x,
                    y: row_y,
                    width: list_inner.width,
                    height: 1,
                };
                self.render_theme_card(frame, row_rect, i, i == self.selected_theme_index, &active_name);
            }
        }

        let detail_inner = Rect {
            x: detail_rect.x + 1,
            y: detail_rect.y,
            width: detail_rect.width.saturating_sub(2),
            height: detail_rect.height,
        };

        let mut detail_lines: Vec<Line> = Vec::new();
        if let Some(entry) = selected_entry {
            let swatches = self
                .theme_swatches
                .get(entry.name())
                .copied()
                .unwrap_or([crate::theme::RgbColor::new(50, 50, 50); 4]);
            let is_active = active_name.as_deref() == Some(entry.name());
            let type_label = if entry.is_builtin() { "built-in theme" } else { "saved theme" };
            let status = if is_active {
                " Active in editor"
            } else {
                " Previewing selection"
            };
            detail_lines.push(Line::from(Span::styled(
                format!(" {}", entry.name()),
                Style::new().fg(PANEL_TEXT).add_modifier(Modifier::BOLD),
            )));
            detail_lines.push(Line::from(Span::styled(
                format!(" {}", type_label),
                Style::new().fg(PANEL_MUTED),
            )));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                " Palette",
                Style::new().fg(PANEL_MUTED).add_modifier(Modifier::BOLD),
            )));
            detail_lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("● ", Style::new().fg(Color::Rgb(swatches[0].r, swatches[0].g, swatches[0].b))),
                Span::styled("● ", Style::new().fg(Color::Rgb(swatches[1].r, swatches[1].g, swatches[1].b))),
                Span::styled("● ", Style::new().fg(Color::Rgb(swatches[2].r, swatches[2].g, swatches[2].b))),
                Span::styled("●", Style::new().fg(Color::Rgb(swatches[3].r, swatches[3].g, swatches[3].b))),
            ]));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                " Status",
                Style::new().fg(PANEL_MUTED).add_modifier(Modifier::BOLD),
            )));
            detail_lines.push(Line::from(Span::styled(
                status,
                Style::new().fg(if is_active { Color::Rgb(110, 220, 110) } else { PANEL_TEXT }),
            )));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                " Actions",
                Style::new().fg(PANEL_MUTED).add_modifier(Modifier::BOLD),
            )));
            detail_lines.push(Line::from(Span::styled(
                if entry.is_builtin() {
                    " Enter load   A apply   R rename   X del"
                } else {
                    " Enter load   A apply   R rename   X delete"
                },
                Style::new().fg(PANEL_TEXT),
            )));
            detail_lines.push(Line::from(Span::styled(
                if entry.is_builtin() {
                    " Rename and delete are unavailable for built-in themes."
                } else {
                    " Saved themes can be renamed or deleted from here."
                },
                Style::new().fg(PANEL_MUTED),
            )));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                " Hint",
                Style::new().fg(PANEL_MUTED).add_modifier(Modifier::BOLD),
            )));
            detail_lines.push(Line::from(Span::styled(
                " Move through the list to preview each theme instantly.",
                Style::new().fg(PANEL_TEXT),
            )));
        } else {
            detail_lines.push(Line::from(Span::styled(
                " No theme selected",
                Style::new().fg(PANEL_TEXT).add_modifier(Modifier::BOLD),
            )));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                " Type to filter or use Esc to clear the search.",
                Style::new().fg(PANEL_MUTED),
            )));
        }

        frame.render_widget(
            Paragraph::new(detail_lines).style(Style::new().bg(PANEL_BG)),
            detail_inner,
        );

        let footer_rect = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" ↑↓ ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" navigate  ", Style::new().fg(PANEL_MUTED)),
                Span::styled(" type ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" filter  ", Style::new().fg(PANEL_MUTED)),
                Span::styled(" Enter ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" load  ", Style::new().fg(PANEL_MUTED)),
                Span::styled(" A ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" apply  ", Style::new().fg(PANEL_MUTED)),
                Span::styled(" Esc ", Style::new().fg(ACCENT_FG).bg(ACCENT_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" close", Style::new().fg(PANEL_MUTED)),
            ]))
            .style(Style::new().bg(PANEL_BG)),
            footer_rect,
        );
    }

    fn render_theme_card(
        &self,
        frame: &mut Frame,
        area: Rect,
        index: usize,
        selected: bool,
        active_name: &Option<String>,
    ) {
        let entry = &self.loadable_themes[index];
        let swatches = self.theme_swatches.get(entry.name()).copied()
            .unwrap_or([crate::theme::RgbColor::new(50, 50, 50); 4]);
        let is_active = active_name.as_deref() == Some(entry.name());
        let row_bg = if selected {
            Color::Rgb(30, 30, 38)
        } else {
            Color::Rgb(22, 22, 26)
        };
        let row_fg = if selected {
            Color::White
        } else {
            Color::Rgb(192, 192, 214)
        };
        let type_fg = if selected {
            Color::Rgb(176, 176, 202)
        } else {
            Color::Rgb(108, 108, 132)
        };

        let mut spans = vec![
            Span::styled(
                if selected { "> " } else { "  " },
                Style::new()
                    .fg(if selected { Color::Rgb(200, 190, 240) } else { Color::Rgb(72, 72, 92) })
                    .bg(row_bg)
                    .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() }),
            ),
        ];

        let type_label = if entry.is_builtin() { "bundled" } else { "saved" };
        let swatch_w = swatches.len() * 2;
        let active_w = if is_active { 2 } else { 0 };
        let reserved = 2 + swatch_w + active_w + type_label.len();
        let name_width = (area.width as usize).saturating_sub(reserved + 2).max(8);
        let clipped_name = clip_text(entry.name(), name_width);
        spans.push(Span::styled(
            format!("{:<width$}", clipped_name, width = name_width),
            Style::new().fg(row_fg).bg(row_bg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(" ", Style::new().bg(row_bg)));
        for sw in swatches {
            spans.push(Span::styled(
                "● ",
                Style::new().fg(Color::Rgb(sw.r, sw.g, sw.b)).bg(row_bg),
            ));
        }
        if is_active {
            spans.push(Span::styled(
                "● ",
                Style::new().fg(Color::Rgb(110, 220, 110)).bg(row_bg),
            ));
        }
        spans.push(Span::styled(
            type_label,
            Style::new().fg(type_fg).bg(row_bg),
        ));

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::new().bg(row_bg)),
            area,
        );
    }
}
