# Dioxus Desktop Reference

Quick reference for Dioxus desktop development. Extracted from official docs.

## Hot Reload

### What Hot Reloads (instant, no rebuild)

- RSX structure changes (add/remove/modify elements)
- String attributes (`class`, `id`, `style`)
- Formatted strings with variables
- Simple literals as props (numbers, booleans, strings)
- Markup inside conditionals and loops
- CSS files (including Tailwind with watcher)
- Static images and assets

### What Requires Full Rebuild

- New variables or expressions
- Logic outside RSX (function bodies, hooks)
- Component signatures (props changes)
- Import statements, module structure
- Complex Rust expressions in attributes
- **Adding new fields to structs (like Signal fields in AppState)**

### Experimental: `--hotpatch`

```bash
dx serve --hotpatch
```

Enables Rust code hot-reloading. Press `r` to force full rebuild.

**Limitations:**
- Destructors for new globals never execute
- Workspace dependencies don't trigger changes
- Static initializer changes not detected

## DOM Structure

Dioxus desktop wraps your app in a `#main` div:

```html
<body>
  <div id="__dx-toast"></div>
  <div id="main">
    <!-- Your rsx! content renders here -->
    <!--placeholder-->
    <div id="app-root">...</div>
  </div>
  <script>...</script>
</body>
```

### Configuring Root Element

```rust
Config::new()
    .with_root_name("my-app")  // Mount to #my-app instead of #main
```

### Custom Index HTML

```rust
Config::new()
    .with_custom_index(r#"
        <!DOCTYPE html>
        <html>
        <head>...</head>
        <body>
            <div id="app"></div>
        </body>
        </html>
    "#)
    .with_root_name("app")
```

**Important:** Document MUST include a `<body>` element. Dioxus injects loader code there.

## CSS Positioning Gotcha

`position: fixed` elements are positioned relative to the **viewport**, but:

1. Dioxus renders into `#main` wrapper
2. If parent has `transform`, `filter`, `will-change`, or `contain: paint`, fixed positioning breaks
3. Check with: `getComputedStyle(element).transform`

**Safe pattern for modals:**
```rust
// Render modal at app root level, not inside components
rsx! {
    div { id: "app-root", ... }

    // Modal as sibling, not child
    if show_modal {
        Modal { ... }
    }
}
```

## JavaScript Evaluation

```rust
// Run JS in webview
let mut eval = document::eval("return document.title");
let result = eval.await;  // Returns serde_json::Value

// Two-way communication
let eval = use_eval(cx);
eval.send("message".into());  // Rust -> JS

// In JS:
// dioxus.send(data);  // JS -> Rust
```

## Platform Dependencies

| Platform | Required |
|----------|----------|
| macOS | None |
| Windows | WebView2 (auto-installs) |
| Linux | `libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev` |

## Testing

### Component Output Testing (SSR)

```rust
use dioxus_ssr::render_element;
use pretty_assertions::assert_str_eq;

fn assert_rsx_eq(first: Element, second: Element) {
    let first = render_element(first);
    let second = render_element(second);
    assert_str_eq!(first, second);
}

#[test]
fn test_button_renders() {
    let result = render_element(rsx! {
        Button { label: "Click me" }
    });
    assert!(result.contains("Click me"));
    assert!(result.contains("button"));
}
```

### Hook Testing

No built-in hook testing library. Create mock components that use the hook and manually drive the VirtualDom through state transitions.

### E2E Testing

Use Playwright with `dx serve` as dev server. See: [Dioxus Playwright examples](https://github.com/DioxusLabs/dioxus)

## References

- [Dioxus Desktop Guide](https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/)
- [Hot Reload Docs](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload/)
- [Testing Guide](https://dioxuslabs.com/learn/0.7/guides/testing/web/)
- [Config API](https://docs.rs/dioxus-desktop/latest/dioxus_desktop/struct.Config.html)
