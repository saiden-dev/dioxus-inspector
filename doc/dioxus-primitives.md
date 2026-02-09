# Dioxus Primitives

Official unstyled components from DioxusLabs. Added as git dependency (not yet published to crates.io).

```toml
dioxus-primitives = { git = "https://github.com/DioxusLabs/components", package = "dioxus-primitives" }
```

## Components In Use

| Component | Usage | File |
|-----------|-------|------|
| `DialogRoot` + `DialogContent` | Modal dialogs | `server_selector.rs` |
| `PopoverRoot` + `PopoverContent` | Dropdown menus | `server_selector.rs` |
| `Separator` | Divider lines | `server_selector.rs` |
| `Progress` + `ProgressIndicator` | Download progress bars | `download_panel.rs` |
| `Tooltip` + `TooltipTrigger` + `TooltipContent` | Truncated text tooltips | `truncated_text.rs` |
| `Switch` + `SwitchThumb` | Search/Stored view toggle | `view_toggle.rs` |
| `Slider` + `SliderTrack` + `SliderRange` + `SliderThumb` | CFG/Steps sliders | `gen_sidebar.rs` |

## Components Not Used

| Component | Reason |
|-----------|--------|
| `DropdownMenu` | Popover is simpler for action menus |
| `Select` | Native `<select>` works for current needs |

### Note on Signal "Overhead"

The `ReadSignal` requirement is **not overhead** - it's the correct reactive pattern. The xm Vue app uses equivalent patterns (`ref` + `computed`) everywhere for:
- Real-time generation progress
- Task queue with live updates
- Model compatibility filtering (LoRAs auto-filter when checkpoint changes)
- Settings that sync across components

The one-liner `use_memo(move || Some(*signal.read()))` is equivalent to Vue's `computed()`. Consider using primitives when building features that need reactive state (which is most non-trivial UI).

## API Pattern

Most primitives require `ReadSignal<T>` for controlled props. Convert with `use_memo`:

```rust
let mut open = use_signal(|| false);
let open_signal = use_memo(move || Some(*open.read()));

DialogRoot {
    open: open_signal,
    on_open_change: move |v| { /* handle close */ },
    // ...
}
```

## CRITICAL: Read Signals Inside use_memo

**This pattern has caused bugs 3+ times.** When passing state to primitives via `use_memo`, you MUST read the signal INSIDE the memo closure, not outside:

```rust
// WRONG - captures `current_value` by value, NOT reactive!
// The component won't update when state changes.
let current_value = *state.my_signal.read();
let memo = use_memo(move || Some(current_value));

// CORRECT - reads signal inside memo, IS reactive
let memo = use_memo(move || Some(*state.my_signal.read()));
```

The wrong pattern causes the primitive's visual state to desync from the actual value - e.g., a Slider thumb won't move even though the value updates. Always read signals inside the closure.

## Styling

Components are unstyled. Add Tailwind classes via `class` prop:

```rust
DialogContent {
    class: "modal-backdrop",  // from input.css @apply
    // ...
}
```

## Reference

- Gallery: https://dioxuslabs.github.io/components/
- Repo: https://github.com/DioxusLabs/components
- Dioxus version: 0.7
