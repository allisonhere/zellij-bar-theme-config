# Collapse 2 "Tab (Unselected)" sidebar entries to 1

## Goal

In the left sidebar, show a single "Tab (Unselected)" entry. When selected (and during color editing), both unselected tabs in the Zellij preview ("2:bash" and "3:nvim") get the underline modifier. The preview tab bar itself keeps 3 tabs.

## Current state

- `PreviewElement::TabUnselected1` and `TabUnselected2` both exist and both have the same label "Tab (Unselected)", same component type `RibbonUnselected`
- `render_zellij_tab_bar` checks `is_tab_u1` (for "2:bash") and `is_tab_u2` (for "3:nvim") independently

## Chosen approach: Option C (single entry, impact both preview tabs)

Keep the preview tab bar with 3 tabs, but:
- Remove `TabUnselected2` from `PreviewElement`
- Rename `TabUnselected1` to `TabUnselected` (cleaner)
- When `TabUnselected` is active, apply `selection_modifier` to BOTH unselected tabs in the preview

## Step-by-step

### Step 1: Consolidate PreviewElement

**File: `src/ui/state.rs`**

- Rename `TabUnselected1` to `TabUnselected` in enum (line 166)
- Remove `TabUnselected2` variant

### Step 2: Update TabBar group fields

**File: `src/ui/state.rs`** (line 154)

```rust
// Before:
Self::TabBar => &[TabSelected, TabUnselected1, TabUnselected2],
// After:
Self::TabBar => &[TabSelected, TabUnselected],
```

### Step 3: Update PreviewElement::all()

**File: `src/ui/state.rs`** (lines 186-205)

Remove `TabUnselected2` from the array. Rename `TabUnselected1` to `TabUnselected`.

### Step 4: Update PreviewElement::group()

**File: `src/ui/state.rs`** (line 210)

```rust
// Before:
TabSelected | TabUnselected1 | TabUnselected2 => PreviewGroup::TabBar,
// After:
TabSelected | TabUnselected => PreviewGroup::TabBar,
```

### Step 5: Update PreviewElement::component_type()

**File: `src/ui/state.rs`** (line 234)

```rust
// Before:
Self::TabUnselected1 | Self::TabUnselected2 => ThemeComponentType::RibbonUnselected,
// After:
Self::TabUnselected => ThemeComponentType::RibbonUnselected,
```

### Step 6: Update PreviewElement::label()

**File: `src/ui/state.rs`** (line 253)

```rust
// Before:
Self::TabUnselected1 | Self::TabUnselected2 => "Tab (Unselected)",
// After:
Self::TabUnselected => "Tab (Unselected)",
```

### Step 7: Update tab bar rendering — one selection highlights both

**File: `src/ui/render.rs`** (lines 496-534)

Change the condition: instead of separate `is_tab_u1` and `is_tab_u2`, use a single `is_tab_u` that applies to both unselected tabs.

```rust
// Before:
let is_tab_u1 = self.is_element_active(PreviewElement::TabUnselected1);
let is_tab_u2 = self.is_element_active(PreviewElement::TabUnselected2);

// After:
let is_tab_u = self.is_element_active(PreviewElement::TabUnselected);
```

And in both "2:bash" and "3:nvim" renderings, change `selection_modifier(is_tab_u1)` / `selection_modifier(is_tab_u2)` → `selection_modifier(is_tab_u)`.

### Step 8: Run cargo test to verify

```bash
cargo test
```

All tests in `render.rs` reference `TabUnselected1` — update them to use `TabUnselected`.

## Files to change

| File | Changes |
|---|---|
| `src/ui/state.rs` | Remove `TabUnselected2`, rename `TabUnselected1` → `TabUnselected`, update all methods |
| `src/ui/render.rs` | Single `is_tab_u` flag, apply to both unselected tabs in `render_zellij_tab_bar`; update test references |

## Tests / validation

- `cargo test` — all existing tests pass after renaming
- Manual: open the app, navigate to "Tab (Unselected)" in sidebar, verify both "2:bash" and "3:nvim" get underline
- Manual: edit RibbonUnselected colors via color picker, verify both unselected tabs update in preview

## Risks

- Low. Purely cosmetic/structural. No theme data changes.
- The single `TabUnselected` entry still maps to `RibbonUnselected` component type — editing it changes colors for all unselected tabs, which is correct behavior.
