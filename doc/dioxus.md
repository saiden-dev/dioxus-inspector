# Dioxus Guide

## Component Structure

```rust
use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        div { class: "container",
            Header {}
            Content {}
            Footer {}
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header { class: "header",
            h1 { "XC" }
        }
    }
}
```

## State Management

```rust
fn counter() -> Element {
    // Local state (like Vue ref)
    let mut count = use_signal(|| 0);

    rsx! {
        button {
            onclick: move |_| count += 1,
            "Count: {count}"
        }
    }
}
```

## Props

```rust
#[component]
fn ModelCard(
    name: String,
    thumbnail: String,
    #[props(default = false)] selected: bool,
) -> Element {
    rsx! {
        div { class: if selected { "card selected" } else { "card" },
            img { src: "{thumbnail}" }
            span { "{name}" }
        }
    }
}

// Usage
rsx! {
    ModelCard { name: "SDXL".to_string(), thumbnail: url, selected: true }
}
```

## Lists

```rust
fn model_list() -> Element {
    let models = use_signal(|| vec!["SDXL", "SD 1.5", "Flux"]);

    rsx! {
        ul {
            for model in models.read().iter() {
                li { key: "{model}", "{model}" }
            }
        }
    }
}
```

## Async Data Fetching

```rust
fn gallery() -> Element {
    let images = use_resource(|| async {
        fetch_images().await
    });

    match &*images.read_unchecked() {
        Some(Ok(data)) => rsx! {
            for img in data {
                img { src: "{img.url}" }
            }
        },
        Some(Err(e)) => rsx! { "Error: {e}" },
        None => rsx! { "Loading..." },
    }
}
```

## Global State (like Pinia)

```rust
use dioxus::prelude::*;

#[derive(Clone)]
struct AppState {
    selected_model: Signal<Option<String>>,
    generating: Signal<bool>,
}

fn app() -> Element {
    use_context_provider(|| AppState {
        selected_model: Signal::new(None),
        generating: Signal::new(false),
    });

    rsx! { Gallery {} }
}

fn Gallery() -> Element {
    let state = use_context::<AppState>();
    let model = state.selected_model.read();

    rsx! {
        div { "Selected: {model:?}" }
    }
}
```

## Hot Reload

Dioxus has hot reload with state preservation:

```bash
dx serve --hot-reload
```

Changes to `rsx!` blocks update instantly without losing state.

---

# Component Libraries

## Stack

| Library | Purpose |
|---------|---------|
| [Dioxus Components](https://dioxuslabs.com/components/) | Accessible primitives + styled components |
| [Tailwind CSS](https://tailwindcss.com/) | Utility-first styling |
| [dioxus-free-icons](https://crates.io/crates/dioxus-free-icons) | SVG icon sets |

## Dioxus Components (Official)

First-party component library with two layers:

1. **Primitives** - Unstyled, accessible (like Radix)
2. **Components** - Styled, ready-to-use (like shadcn)

```rust
use dioxus::prelude::*;
use dioxus_components::{Button, Dialog, Select, Switch};

fn settings_dialog() -> Element {
    let mut open = use_signal(|| false);
    let mut dark_mode = use_signal(|| true);

    rsx! {
        Button { onclick: move |_| open.set(true), "Settings" }

        Dialog { open: open(),
            Dialog::Title { "Settings" }
            Dialog::Description { "Configure your preferences" }

            div { class: "flex items-center gap-2",
                "Dark mode"
                Switch {
                    checked: dark_mode(),
                    on_change: move |v| dark_mode.set(v),
                }
            }

            Dialog::Close { "Done" }
        }
    }
}
```

Available primitives:
- `Dialog` - Modal dialogs with focus trap
- `Popover` - Positioned floating content
- `Select` - Dropdown selection
- `Switch` - Toggle switch
- `Checkbox` - Checkbox input
- `Tabs` - Tabbed interface
- `Tooltip` - Hover tooltips

## Tailwind CSS Setup

```toml
# Dioxus.toml
[application]
name = "xc"
asset_dir = "assets"

[web.watcher]
watch_path = ["src", "assets"]

[web.resource]
style = ["assets/tailwind.css"]
```

```bash
# Install Tailwind CLI
npm install -D tailwindcss
npx tailwindcss init
```

```javascript
// tailwind.config.js
module.exports = {
  content: ["./src/**/*.rs"],
  theme: {
    extend: {
      colors: {
        surface: {
          1: "var(--surface-1)",
          2: "var(--surface-2)",
          3: "var(--surface-3)",
        },
      },
    },
  },
}
```

```css
/* assets/input.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  --surface-1: #1a1a1a;
  --surface-2: #2a2a2a;
  --surface-3: #3a3a3a;
  --text-1: #ffffff;
  --text-2: #a0a0a0;
  --primary: #3b82f6;
  --danger: #ef4444;
}
```

```bash
# Build CSS (add to dev script)
npx tailwindcss -i assets/input.css -o assets/tailwind.css --watch
```

### No Inline Styles in Rust

**Don't do this:**
```rust
// BAD - styles as Rust const
const STYLES: &str = r#"
    .card { background: #252542; }
    .button { padding: 0.75rem; }
"#;

fn app() -> Element {
    rsx! {
        style { {STYLES} }
        div { class: "card", ... }
    }
}
```

**Do this instead:**
```rust
// GOOD - Tailwind classes inline
fn app() -> Element {
    rsx! {
        div { class: "bg-surface-2 rounded-lg p-6", ... }
    }
}
```

**Why:**
- Tailwind purges unused classes (smaller bundle)
- Styles visible at point of use
- No context switching to find CSS
- IDE autocomplete for class names

Usage in components:

```rust
fn model_card() -> Element {
    rsx! {
        div { class: "bg-surface-2 rounded-lg p-4 hover:bg-surface-3 transition-colors",
            img { class: "w-full aspect-square object-cover rounded", src: "..." }
            h3 { class: "text-text-1 font-medium mt-2", "SDXL" }
            p { class: "text-text-2 text-sm", "Base model" }
        }
    }
}
```

## dioxus-free-icons

```toml
# Cargo.toml
[dependencies]
dioxus-free-icons = { version = "0.9", features = [
    "font-awesome-solid",
    "font-awesome-regular",
    "heroicons-outline",
    "heroicons-solid",
] }
```

Usage:

```rust
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaGear, FaImage, FaPlay, FaTrash};
use dioxus_free_icons::Icon;

fn toolbar() -> Element {
    rsx! {
        div { class: "flex gap-2",
            button { class: "p-2 hover:bg-surface-2 rounded",
                Icon { icon: FaPlay, width: 16, height: 16, fill: "currentColor" }
            }
            button { class: "p-2 hover:bg-surface-2 rounded",
                Icon { icon: FaImage, width: 16, height: 16, fill: "currentColor" }
            }
            button { class: "p-2 hover:bg-surface-2 rounded",
                Icon { icon: FaGear, width: 16, height: 16, fill: "currentColor" }
            }
            button { class: "p-2 hover:bg-surface-2 rounded text-danger",
                Icon { icon: FaTrash, width: 16, height: 16, fill: "currentColor" }
            }
        }
    }
}
```

Available icon sets:
- `font-awesome-solid` / `font-awesome-regular` / `font-awesome-brands`
- `heroicons-outline` / `heroicons-solid`
- `lucide`
- `bootstrap`
- `octicons`
- `ionicons`

## Component Organization

**Use Dioxus Components directly** - don't wrap them in "primitives" folders.

```
src/
├── components/
│   ├── mod.rs
│   ├── layout.rs            # Header, Sidebar (if small, one file)
│   ├── model_card.rs        # Domain: model display
│   ├── model_grid.rs        # Domain: model listing
│   ├── prompt_input.rs      # Domain: generation input
│   └── gallery.rs           # Domain: image gallery
```

**Rules:**
- Use `Dialog`, `Select`, `Switch` from Dioxus Components directly in domain components
- Style with Tailwind classes, not wrapper components
- Only create a component file when it has domain logic (not just styling)
- One file can have multiple small components

## Example: Using Library Components

```rust
use dioxus::prelude::*;
use dioxus_components::{Dialog, Select, Switch};
use dioxus_free_icons::icons::fa_solid_icons::FaGear;
use dioxus_free_icons::Icon;

/// Settings dialog - uses library components directly, styled with Tailwind
fn SettingsDialog(open: Signal<bool>) -> Element {
    let mut host = use_signal(|| "localhost:8188".to_string());
    let mut dark_mode = use_signal(|| true);

    rsx! {
        Dialog { open: open(),
            div { class: "bg-zinc-900 rounded-lg p-6 min-w-80",
                h2 { class: "text-lg font-semibold mb-4", "Settings" }

                div { class: "space-y-4",
                    // Text input - just use native with Tailwind
                    div {
                        label { class: "block text-sm text-zinc-400 mb-1", "ComfyUI Host" }
                        input {
                            class: "w-full bg-zinc-800 rounded px-3 py-2 text-sm",
                            value: "{host}",
                            oninput: move |e| host.set(e.value()),
                        }
                    }

                    // Toggle - use library Switch
                    div { class: "flex items-center justify-between",
                        span { class: "text-sm", "Dark mode" }
                        Switch { checked: dark_mode(), on_change: move |v| dark_mode.set(v) }
                    }
                }

                div { class: "mt-6 flex justify-end",
                    button {
                        class: "bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded text-sm",
                        onclick: move |_| open.set(false),
                        "Done"
                    }
                }
            }
        }
    }
}
```

**Note:** No custom Button component - just `<button>` with Tailwind classes. No custom Dialog wrapper - use Dioxus Components directly.
