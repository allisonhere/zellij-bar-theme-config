use crate::theme::ThemeComponentType;
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

pub fn render_slider_lines(
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
                ("s",    "SAVE AS",    "SAVE"),
                ("l",    "LOAD",       "LD"),
                ("a",    "SAVE+APPLY", "APPLY"),
                ("?",    "HELP",       "?"),
                ("q",    "QUIT",       "QT"),
            ],
            InputMode::ColorPicker => {
                const WITH_TAB: &[(&str, &str, &str)] = &[
                    ("↑↓",     "CHANNEL", "CH"),
                    ("←→",     "±5",      "±5"),
                    ("S+←→",   "±1",      "±1"),
                    ("PgUp/Dn","±25",     "±25"),
                    ("#",      "HEX",     "HX"),
                    ("tab",    "FG/BG",   "F/B"),
                    ("Enter",  "KEEP",    "OK"),
                    ("Esc",    "CANCEL",  "ESC"),
                ];
                const WITHOUT_TAB: &[(&str, &str, &str)] = &[
                    ("↑↓",     "CHANNEL", "CH"),
                    ("←→",     "±5",      "±5"),
                    ("S+←→",   "±1",      "±1"),
                    ("PgUp/Dn","±25",     "±25"),
                    ("#",      "HEX",     "HX"),
                    ("Enter",  "KEEP",    "OK"),
                    ("Esc",    "CANCEL",  "ESC"),
                ];
                if self.selected_element.is_frame() { WITHOUT_TAB } else { WITH_TAB }
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
            InputMode::Help => &[
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
        let area = frame.area();
        let overlay_w = 52u16.min(area.width.saturating_sub(4));
        let overlay_h = 17u16.min(area.height.saturating_sub(4));
        let overlay_x = area.width.saturating_sub(overlay_w + 2);
        let overlay_y = (area.height.saturating_sub(overlay_h)) / 2;
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

        frame.render_widget(Clear, overlay_area);

        const OB_BG:     Color = Color::Rgb(10, 10, 14);
        const OB_BORDER: Color = Color::Rgb(55, 50, 75);
        const KEY_BG:    Color = Color::Rgb(80, 80, 160);
        const KEY_FG:    Color = Color::Rgb(18, 18, 18);
        const LBL_FG:    Color = Color::Rgb(150, 140, 200);
        const LBL_BG:    Color = Color::Rgb(22, 18, 42);

        let r = self.color_editor.r;
        let g = self.color_editor.g;
        let b = self.color_editor.b;
        let cur_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        let cur_color = Color::Rgb(r, g, b);

        // Derive "before" color from saved original_component
        let orig_color = self.original_component.as_ref().map(|orig| {
            let c = match self.selected_attribute {
                PreviewAttribute::Base => orig.base,
                PreviewAttribute::Background => orig.background,
            };
            (
                Color::Rgb(c.r, c.g, c.b),
                format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
            )
        });

        let element_name = self.selected_element.label();
        let attr_name = self.selected_attribute.label();

        let mut lines: Vec<Line> = Vec::new();

        // ── Header row: title left, attribute pill right (mirrors loader layout) ──
        let title_str = format!(" {} — {}", element_name, attr_name);
        let title_span = Span::styled(title_str.clone(), Style::new().fg(Color::Rgb(200, 200, 240)).add_modifier(Modifier::BOLD));
        let attr_pill: Vec<Span> = vec![
            Span::styled(format!(" {} ", attr_name), Style::new().fg(KEY_FG).bg(KEY_BG).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", element_name), Style::new().fg(LBL_FG).bg(LBL_BG)),
            Span::raw(" "),
        ];
        let title_w = title_str.len();
        let pill_w: usize = attr_pill.iter().map(|s| s.width()).sum();
        let inner_w = overlay_w.saturating_sub(2) as usize;
        let gap = inner_w.saturating_sub(title_w + pill_w);
        let mut header_spans = vec![title_span];
        header_spans.push(Span::styled(" ".repeat(gap), Style::new().bg(OB_BG)));
        header_spans.extend(attr_pill);
        lines.push(Line::from(header_spans));

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
                Span::styled(display, Style::new().fg(Color::Rgb(167, 139, 250)).add_modifier(Modifier::BOLD)),
                Span::styled("  type hex, Enter to apply, Esc to cancel", Style::new().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "  press # to enter hex code",
                Style::new().fg(Color::DarkGray),
            )]));
        }
        lines.push(Line::from(""));

        // Two-row flat pill hints — same style as the theme loader D/S pills
        let mk_pill = |key: &str, action: &str| -> Vec<Span<'static>> {
            vec![
                Span::styled(format!(" {} ", key),    Style::new().fg(KEY_FG).bg(KEY_BG).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", action), Style::new().fg(LBL_FG).bg(LBL_BG)),
                Span::raw("  "),
            ]
        };
        let row1: Vec<Span> = [("↑↓","ch"), ("←→","±5"), ("S+←→","±1"), ("#","hex")]
            .iter().flat_map(|(k,a)| mk_pill(k, a)).collect();
        let row2: Vec<Span> = if self.selected_element.is_frame() {
            [("↵","keep"), ("Esc","cancel")]
                .iter().flat_map(|(k,a)| mk_pill(k, a)).collect()
        } else {
            [("tab","fg/bg"), ("↵","keep"), ("Esc","cancel")]
                .iter().flat_map(|(k,a)| mk_pill(k, a)).collect()
        };
        lines.push(Line::from([vec![Span::raw(" ")], row1].concat()));
        lines.push(Line::from([vec![Span::raw(" ")], row2].concat()));

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(OB_BORDER))
            .style(Style::new().bg(OB_BG));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(OB_BG)),
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
            .border_style(Style::new().fg(Color::Rgb(55, 50, 75)))
            .style(Style::new().bg(Color::Rgb(10, 10, 14)));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(Color::Rgb(10, 10, 14))),
            area,
        );
    }

    fn render_help_overlay(&self, frame: &mut Frame) {
        let entries: &[(&str, &str)] = &[
            // Normal mode
            ("↑/j  ↓/k  ← →",  "Navigate preview elements"),
            ("Tab",             "Toggle FG / BG (pane borders: FG only)"),
            ("c",               "Open color picker for selected color"),
            ("y",               "Yank (copy) current color"),
            ("p",               "Paste yanked color"),
            ("u",               "Undo last color change"),
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
            ("#",               "[Color picker] Enter hex code"),
            ("Tab",             "[Color picker] Toggle FG / BG (non-pane)"),
            ("Enter",           "[Color picker] Confirm color"),
            ("Esc",             "[Color picker] Cancel"),
            ("",                ""),
            // Theme loader
            ("↑ ↓",            "[Load] Navigate themes"),
            ("type",            "[Load] Search/filter themes by name"),
            ("Backspace",       "[Load] Delete search character"),
            ("d",               "[Load] Filter: default / built-in themes"),
            ("s",               "[Load] Filter: saved themes"),
            ("Enter",           "[Load] Load selected theme"),
            ("a",               "[Load] Apply selected theme to Zellij"),
            ("r",               "[Load] Rename selected saved theme"),
            ("x",               "[Load] Delete selected saved theme"),
            ("Esc",             "[Load] Clear search or cancel"),
        ];

        let height = (entries.len() as u16 + 4).min(frame.area().height.saturating_sub(4));
        let area = centered_rect(frame.area(), 68, height);
        frame.render_widget(Clear, area);

        let key_style   = Style::new().fg(Color::Rgb(167, 139, 250)).add_modifier(Modifier::BOLD);
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
            .title_style(Style::new().fg(Color::Rgb(167, 139, 250)).add_modifier(Modifier::BOLD))
            .border_style(Style::new().fg(Color::Rgb(55, 50, 75)))
            .style(Style::new().bg(Color::Rgb(10, 10, 14)));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .style(Style::new().bg(Color::Rgb(10, 10, 14))),
            area,
        );
    }

    fn render_theme_load_overlay(&self, frame: &mut Frame) {
        let overlay_h = 23u16.min(frame.area().height.saturating_sub(2));
        let overlay_w = 56u16.min(frame.area().width.saturating_sub(4));
        let area = centered_rect(frame.area(), overlay_w, overlay_h);
        frame.render_widget(Clear, area);

        const PANEL_BG: Color = Color::Rgb(10, 10, 14);

        // Outer panel (no title in border — title lives in first inner row)
        let outer_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Rgb(55, 50, 75)))
            .style(Style::new().bg(PANEL_BG));
        let inner = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        // ── Header row: "Default Themes" left, D/S pills right ──────────────
        let panel_title = match self.theme_filter {
            ThemeFilter::Builtin => " Default Themes",
            ThemeFilter::Saved   => " Saved Themes",
            ThemeFilter::All     => " All Themes",
        };
        let title_spans: Vec<Span> = vec![
            Span::styled(
                panel_title,
                Style::new().fg(Color::Rgb(200, 200, 240)).add_modifier(Modifier::BOLD),
            ),
        ];
        use crate::ui::state::ThemeFilter;
        let d_active = matches!(self.theme_filter, ThemeFilter::Builtin);
        let s_active = matches!(self.theme_filter, ThemeFilter::Saved);
        let (d_key_bg, d_key_fg, d_lbl_fg, d_lbl_bg) = if d_active {
            (Color::Rgb(140,140,255), Color::Rgb(10,10,10), Color::Rgb(180,180,255), Color::Rgb(35,35,65))
        } else {
            (Color::Rgb(80,80,160), Color::Rgb(18,18,18), Color::Rgb(120,120,180), Color::Rgb(22,22,38))
        };
        let (s_key_bg, s_key_fg, s_lbl_fg, s_lbl_bg) = if s_active {
            (Color::Rgb(140,140,255), Color::Rgb(10,10,10), Color::Rgb(180,180,255), Color::Rgb(35,35,65))
        } else {
            (Color::Rgb(60,60,120), Color::Rgb(18,18,18), Color::Rgb(100,100,150), Color::Rgb(22,22,38))
        };
        let pill_spans: Vec<Span> = vec![
            Span::styled(" D ", Style::new().fg(d_key_fg).bg(d_key_bg).add_modifier(Modifier::BOLD)),
            Span::styled(" default ", Style::new().fg(d_lbl_fg).bg(d_lbl_bg)),
            Span::raw("  "),
            Span::styled(" S ", Style::new().fg(s_key_fg).bg(s_key_bg).add_modifier(Modifier::BOLD)),
            Span::styled(" saved ", Style::new().fg(s_lbl_fg).bg(s_lbl_bg)),
            Span::raw(" "),
        ];
        let title_w: usize = title_spans.iter().map(|s| s.width()).sum();
        let pills_w: usize = pill_spans.iter().map(|s| s.width()).sum();
        let gap = (inner.width as usize).saturating_sub(title_w + pills_w);
        let mut header_spans = title_spans;
        header_spans.push(Span::styled(" ".repeat(gap), Style::new().bg(PANEL_BG)));
        header_spans.extend(pill_spans);

        let header_rect = Rect { x: inner.x, y: inner.y, width: inner.width, height: 1 };
        frame.render_widget(
            Paragraph::new(Line::from(header_spans)).style(Style::new().bg(PANEL_BG)),
            header_rect,
        );

        // ── Search row ───────────────────────────────────────────────────────
        let display_query = self.theme_search_query.trim_start_matches('/');
        let search_text = if display_query.is_empty() {
            String::from("  search: (type to filter)")
        } else if self.search_focused {
            format!("  search: {}_", display_query)
        } else {
            format!("  search: {}  (Enter/↑↓ to navigate)", display_query)
        };
        let search_fg = if self.theme_search_query.is_empty() {
            Color::Rgb(80, 80, 100)
        } else {
            Color::Rgb(200, 200, 240)
        };
        let search_rect = Rect { x: inner.x, y: inner.y + 1, width: inner.width, height: 1 };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(search_text, Style::new().fg(search_fg).bg(PANEL_BG)))),
            search_rect,
        );

        // ── Cards ────────────────────────────────────────────────────────────
        // Each card slot = 4 lines (card border + 2 inner + card border)
        // plus 1 blank gap row = 5 total per slot
        const CARD_SLOT: u16 = 5;
        let cards_y = inner.y + 3; // 1 header + 1 search + 1 blank gap
        let cards_h = inner.height.saturating_sub(3);
        let cards_per_view = (cards_h / CARD_SLOT) as usize;

        let active_name = self.original_theme.as_ref().map(|t| t.name.clone());

        let scroll = if self.loadable_themes.len() <= cards_per_view {
            0
        } else if self.selected_theme_index < cards_per_view / 2 {
            0
        } else {
            (self.selected_theme_index.saturating_sub(cards_per_view / 2))
                .min(self.loadable_themes.len().saturating_sub(cards_per_view))
        };

        for i in scroll..self.loadable_themes.len() {
            let card_top = cards_y + ((i - scroll) as u16 * CARD_SLOT);
            if card_top + 4 > inner.y + inner.height {
                break;
            }
            let card_rect = Rect {
                x: inner.x,
                y: card_top,
                width: inner.width,
                height: 4,
            };
            self.render_theme_card(frame, card_rect, i, i == self.selected_theme_index, &active_name);
        }
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

        let (card_bg, border_col) = if selected {
            (Color::Rgb(22, 20, 30), Color::Rgb(100, 90, 130))
        } else {
            (Color::Rgb(14, 13, 20), Color::Rgb(45, 42, 60))
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(border_col))
            .style(Style::new().bg(card_bg));
        let card_inner = block.inner(area);
        frame.render_widget(block, area);

        // ── Line 1: name [●active] ──── swatches ──
        let name_fg = if selected { Color::White } else { Color::Rgb(190, 190, 215) };
        let mut line1: Vec<Span> = vec![
            Span::raw(" "),
            Span::styled(entry.name(), Style::new().fg(name_fg).bg(card_bg).add_modifier(Modifier::BOLD)),
        ];
        if is_active {
            line1.push(Span::styled(" ●", Style::new().fg(Color::Rgb(80, 210, 80)).bg(card_bg)));
        }
        // Build swatch spans and measure them
        let swatch_spans: Vec<Span> = swatches.iter().map(|sw| {
            Span::styled("⬤ ", Style::new().fg(Color::Rgb(sw.r, sw.g, sw.b)).bg(card_bg))
        }).collect();
        let trailing = Span::styled(" ", Style::new().bg(card_bg));
        let right_w: usize = swatch_spans.iter().map(|s| s.width()).sum::<usize>() + trailing.width();
        let name_w: usize = line1.iter().map(|s| s.width()).sum();
        let pad = (card_inner.width as usize).saturating_sub(name_w + right_w);
        line1.push(Span::styled(" ".repeat(pad), Style::new().bg(card_bg)));
        line1.extend(swatch_spans);
        line1.push(trailing);

        let line1_rect = Rect { x: card_inner.x, y: card_inner.y, width: card_inner.width, height: 1 };
        frame.render_widget(Paragraph::new(Line::from(line1)), line1_rect);

        // ── Line 2: subtitle ──── [A] activate ──
        let subtitle = if entry.is_builtin() { "Built-in theme" } else { "Saved theme" };
        let btn_spans: Vec<Span> = vec![
            Span::styled("[", Style::new().fg(Color::Rgb(80, 80, 105)).bg(card_bg)),
            Span::styled("A", Style::new().fg(Color::Rgb(192, 132, 252)).bg(card_bg).add_modifier(Modifier::BOLD)),
            Span::styled("] activate ", Style::new().fg(Color::Rgb(150, 150, 180)).bg(card_bg)),
        ];
        let btn_w: usize = btn_spans.iter().map(|s| s.width()).sum();
        let sub_max = (card_inner.width as usize).saturating_sub(btn_w + 1);
        let sub_text = format!(" {:<width$}", subtitle, width = sub_max.saturating_sub(1));
        let mut line2: Vec<Span> = vec![
            Span::styled(sub_text, Style::new().fg(Color::Rgb(100, 100, 140)).bg(card_bg)),
        ];
        line2.extend(btn_spans);

        let line2_rect = Rect { x: card_inner.x, y: card_inner.y + 1, width: card_inner.width, height: 1 };
        frame.render_widget(Paragraph::new(Line::from(line2)), line2_rect);
    }
}
