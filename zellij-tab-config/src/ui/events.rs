use crate::ui::state::{App, InputMode};
use crossterm::event::{KeyCode, KeyModifiers};

pub fn process_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match app.input_mode {
        InputMode::Preview => {
            // Any navigation clears the save message
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Up => {
                    app.selected_element.move_up();
                    if app.selected_element.is_frame() { app.selected_attribute = crate::ui::state::PreviewAttribute::Base; }
                    app.message = None;
                }
                KeyCode::Down => {
                    app.selected_element.move_down();
                    if app.selected_element.is_frame() { app.selected_attribute = crate::ui::state::PreviewAttribute::Base; }
                    app.message = None;
                }
                KeyCode::Left => {
                    app.selected_element.move_left();
                    if app.selected_element.is_frame() { app.selected_attribute = crate::ui::state::PreviewAttribute::Base; }
                    app.message = None;
                }
                KeyCode::Right => {
                    app.selected_element.move_right();
                    if app.selected_element.is_frame() { app.selected_attribute = crate::ui::state::PreviewAttribute::Base; }
                    app.message = None;
                }
                KeyCode::Tab => {
                    app.message = None;
                    if !app.selected_element.is_frame() {
                        app.selected_attribute.cycle();
                    }
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
                        app.sync_theme_name_input();
                        app.input_mode = InputMode::ThemeNameInputApply;
                        app.message = Some(String::from("Name this theme before applying"));
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
                KeyCode::Char('y') => {
                    app.message = None;
                    app.yank_color();
                }
                KeyCode::Char('p') => {
                    app.message = None;
                    app.paste_color();
                }
                KeyCode::Char('u') => {
                    app.message = None;
                    app.undo_color();
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
                if !app.selected_element.is_frame() {
                    app.switch_editing_attribute();
                }
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
        InputMode::ThemeNameInputApply => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Preview;
                app.message = Some(String::from("Save cancelled"));
            }
            KeyCode::Enter => {
                app.save_and_apply_theme_as_input_name();
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
                if !app.theme_search_query.is_empty() {
                    app.theme_search_query = String::new();
                    app.refresh_theme_list();
                    app.move_theme_selection_to(0);
                } else {
                    app.cancel_theme_load();
                }
            }
            KeyCode::Enter => {
                app.load_selected_theme();
            }
            KeyCode::Char('a') => {
                app.apply_selected_theme();
            }
            KeyCode::Char('d') => {
                app.set_theme_filter(crate::ui::state::ThemeFilter::Builtin);
            }
            KeyCode::Char('s') => {
                app.set_theme_filter(crate::ui::state::ThemeFilter::Saved);
            }
            KeyCode::Char('r') => {
                app.begin_rename_selected_theme();
            }
            KeyCode::Char('x') => {
                app.begin_delete_selected_theme();
            }
            KeyCode::Up => {
                app.move_theme_selection_up();
            }
            KeyCode::Down => {
                app.move_theme_selection_down();
            }
            KeyCode::Backspace => {
                app.theme_search_query.pop();
                app.refresh_theme_list();
                app.move_theme_selection_to(0);
            }
            KeyCode::Char(c) => {
                app.theme_search_query.push(c);
                app.refresh_theme_list();
                app.move_theme_selection_to(0);
            }
            _ => {}
        },
        InputMode::Help => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.input_mode = InputMode::Preview;
            }
            _ => {}
        },
        InputMode::ThemeLoadRename => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::ThemeLoad;
                app.message = None;
            }
            KeyCode::Enter => {
                app.commit_rename_theme();
            }
            KeyCode::Backspace => {
                app.pop_theme_name_char();
            }
            KeyCode::Char(c) => {
                app.push_theme_name_char(c);
            }
            _ => {}
        },
        InputMode::ThemeLoadDeleteConfirm => match key.code {
            KeyCode::Char('y') => {
                app.confirm_delete_theme();
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.input_mode = InputMode::ThemeLoad;
                app.message = None;
            }
            _ => {}
        },
    }
    false
}

pub fn run(mut app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    terminal.show_cursor()?;

    use crossterm::event::{self as ct_event};
    use crossterm::execute;

    execute!(terminal.backend_mut(), crossterm::event::EnableMouseCapture)?;

    loop {
        terminal.draw(|frame| app.render(frame))?;
        if let ct_event::Event::Key(key) = ct_event::read()? {
            if process_key(&mut app, key) {
                break;
            }
        }
    }

    execute!(terminal.backend_mut(), crossterm::event::DisableMouseCapture)?;
    ratatui::restore();
    Ok(())
}
