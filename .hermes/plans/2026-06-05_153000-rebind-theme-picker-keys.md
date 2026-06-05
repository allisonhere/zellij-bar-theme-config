# Rebind theme picker keys: d→delete, b→built-in filter

## Goal

In the theme loader screen (`InputMode::ThemeLoad`):
- Move built-in theme filter from `d` to `b`
- Make `d` delete the selected theme (replacing current `x`)
- Remove `x` as delete key

## Current bindings

| Key | Action |
|---|---|
| `d` | Filter → built-in themes |
| `s` | Filter → saved themes |
| `x` | Delete selected theme |
| `r` | Rename selected theme |

## Proposed bindings

| Key | Action |
|---|---|
| `b` | Filter → built-in themes |
| `d` | Delete selected theme |
| `s` | Filter → saved themes |
| `r` | Rename selected theme |

`d` deletes regardless of filter — if you try to delete a built-in theme, the code already shows "Cannot delete built-in themes", so it's safe.

## Files to change

### `src/ui/events.rs` (lines 339, 348-350)

```rust
// Before:
KeyCode::Char('d') => {
    app.set_theme_filter(crate::ui::state::ThemeFilter::Builtin);
}
// After:
KeyCode::Char('b') => {
    app.set_theme_filter(crate::ui::state::ThemeFilter::Builtin);
}

// Before:
KeyCode::Char('x') => {
    app.begin_delete_selected_theme();
}
// After:
KeyCode::Char('d') => {
    app.begin_delete_selected_theme();
}
```

### `src/ui/render.rs` (lines 931-938)

Status bar hints in ThemeLoad bindings:

```rust
// Before:
("d",    "DEFAULT", "DEF"),
("x",    "DELETE",  "DEL"),
// After:
("b",    "BUILTIN", "BIN"),
("d",    "DELETE",  "DEL"),
```

Also need to update the all-filters button label — currently `d` shows "DEFAULT" in the filter pill. Let me check that...

Actually looking at the filter pills rendering, those use `A`/`D`/`S` keys — the filter_pill function uses the key passed as the first arg. Let me check where the filter pills are rendered in the theme load dialog.

### `src/ui/render.rs` — filter pills (around line 2138)

The filter pills show `A` for All, `D` for built-in, `S` for saved. Need to change `D` to `B`:

```rust
// Before:
filter_pill("D", "built-in", matches!(self.theme_filter, ThemeFilter::Builtin))
// After:
filter_pill("B", "built-in", matches!(self.theme_filter, ThemeFilter::Builtin))
```

## Tests

No test changes needed — tests don't validate keybindings strings, only component behavior.

`cargo test` should still pass 15/15.

## Risks

- Low. Purely keybinding changes. No logic changes.
- Users who muscle-memoried `x` for delete will need a session to adjust. But `d` for delete is the more natural mnemonic.
