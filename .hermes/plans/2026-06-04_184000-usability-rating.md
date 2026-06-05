# Plan: Usability rating — new two-tier nav vs old flat list

## What changed

| Aspect | Old (flat list) | New (group tabs + fields) |
|--------|-----------------|---------------------------|
| Element count visible | 14 items, one scrollable list | 4 groups, 3-5 fields each |
| Navigation | ↑↓ to scroll all 14 | ←→ for group, ↑↓ for field |
| Discovery | Everything visible at once | Must switch tabs to see content |
| Direct jump | None (scroll) | 1-4 keys jump to group |
| Search | None | / fuzzy search across all fields |
| Selection feedback | Instant | Flash on element change |

## Strengths of the new system

1. **Reduced cognitive load.** 3-5 fields per group vs 14 items. You're never looking at more than what's relevant to the current "category" of theming.

2. **Faster targeting.** 1-4 keys jump directly to a group. In the flat list you had to scroll through items you don't care about to reach the one you want.

3. **Fuzzy search is a power feature.** `/` then type "tab" or "pane" or "exit" — instant jump. For a list of 14 items this is borderline overkill, but for someone who knows what they want it's the fastest path.

4. **Group tabs create a mental model.** TabBar, Panes, Content, Status maps cleanly to Zellij's visual layout. New users can look at the preview and go "I want to change the pane border... that's under Panes."

5. **Selection flash.** The brief border pulse when switching elements draws attention to what changed. Small but meaningful — you don't have to squint at the preview to confirm your navigation worked.

## Weaknesses of the new system

1. **Lost at-a-glance overview.** In the flat list you could see all 14 items and quickly scan for what you wanted. Now you need to tab through groups to discover what's available. A new user might not know "Exit Success" lives under Status.

2. **Extra keypress for common operations.** If you're bouncing between a tab color and a table color, old system: ↑ scroll up, ↓ scroll down. New system: ←→ to switch groups THEN ↑↓ to target. It's one extra dimension of navigation.

3. **Group labels are implicit.** "Content" as a group name isn't self-explanatory — it contains Tables and Lists. Not obvious that List items are under Content.

4. **Fuzzy search discoverability.** `/` is a convention from vim/less/fzf, but someone unfamiliar with TUIs won't discover it naturally. The interface doesn't surface the search hint unless you read the status bar keybindings.

5. **Frame components can't toggle FG/BG.** We addressed this with disabled swatches and hidden keybinding hints, but it's still a conceptual wart — panes look like they should have FG+BG like everything else.

## Usability score

| Criterion | Old | New |
|-----------|-----|-----|
| Learnability | 8/10 | 7/10 |
| Efficiency (expert) | 5/10 | 8/10 |
| Discoverability | 7/10 | 6/10 |
| Error prevention | 8/10 | 8/10 |
| Satisfaction | 5/10 | 7/10 |

**Old: 6.6/10. New: 7.2/10.**

The new system is a net improvement — slightly harder to learn initially but significantly faster once you know where things are. The fuzzy search and group jumps make expert workflows much faster. The main loss is at-a-glance overview.

## If you wanted to close the gap

Three ideas that don't require reverting to flat:

1. **Show group field count on tabs** — e.g., `Panes (4)`, `Content (5)`. Helps discovery without expanding everything.

2. **Always-visible mini indicator** — a single row of dots or initials showing all groups' field counts. Gives the at-a-glance overview back.

3. **Better group name** — "Tables & Lists" instead of "Content". More discoverable.
