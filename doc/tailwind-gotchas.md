# Tailwind CSS Gotchas for Desktop Apps

Common pitfalls when using Tailwind with Dioxus desktop.

## The #1 Problem: Missing Classes

Tailwind only generates CSS for classes it **finds in your source files**. If a class isn't in the compiled CSS, it won't work - even if you add it to your RSX.

### How Tailwind Scans

Tailwind scans files as **plain text**, looking for complete class name tokens. It does NOT:
- Parse Rust/JS/HTML as code
- Understand string interpolation
- Follow variable references

### What Gets Purged

```rust
// BROKEN: Tailwind can't see "top-0" as a token
let position = if floating { "top-0" } else { "top-4" };
div { class: "{position}" }

// BROKEN: Dynamic class construction
let size = 4;
div { class: "w-{size}" }  // "w-4" not detected
```

### What Works

```rust
// GOOD: Complete class names visible in source
div { class: if floating { "top-0" } else { "top-4" } }

// GOOD: All variants spelled out
let classes = match size {
    "sm" => "w-4 h-4",
    "md" => "w-8 h-8",
    "lg" => "w-16 h-16",
    _ => "w-8 h-8",
};
```

## Checking What's Compiled

```bash
# Search for a class in compiled CSS
grep "\.top-0" desktop/assets/tailwind.css

# List all position classes
grep -E "^\.(top|left|right|bottom|inset)" desktop/assets/tailwind.css
```

## Safelisting Classes

When you need classes that aren't in your source (e.g., generated at runtime):

### Tailwind v4 (CSS-based config)

```css
@import "tailwindcss";

/* Safelist specific classes */
@source inline("top-0 inset-0 w-screen h-screen");

/* Safelist with variants */
@source inline("{hover:,focus:,}bg-red-500");

/* Safelist ranges */
@source inline("bg-blue-{100..900..100}");
```

### Tailwind v3 (JS config)

```js
// tailwind.config.js
module.exports = {
  content: ["./src/**/*.rs"],
  safelist: [
    "top-0",
    "inset-0",
    { pattern: /bg-(red|blue|green)-\d+/ },
    { pattern: /text-(sm|lg|xl)/, variants: ["hover"] },
  ],
}
```

## Desktop-Specific Issues

### Issue: CSS Not Loading

Symptoms: Styles don't apply, inspector shows 404 for CSS file.

Fix: Use `asset!()` macro:
```rust
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

rsx! {
    document::Link { rel: "stylesheet", href: TAILWIND_CSS }
}
```

### Issue: Classes Added After Initial Build

Symptoms: New classes in RSX don't style anything.

Fix: Regenerate Tailwind CSS:
```bash
npx tailwindcss -i input.css -o desktop/assets/tailwind.css
```

Or ensure Tailwind watcher is running with `dx serve`.

### Issue: Positioning Classes Don't Work

Common missing classes in desktop apps:
- `inset-0`, `inset-x-0`, `inset-y-0`
- `top-0`, `bottom-0` (but `top-full` may exist)
- `w-screen`, `h-screen`

**Workaround:** Use available alternatives:
```rust
// Instead of: "fixed inset-0"
// Use: "fixed inset-y-0 left-0 right-0"

// Instead of: "fixed top-0 left-0 w-screen h-screen"
// Use: "fixed inset-y-0 left-0 right-0"
```

## Content Configuration

### Scanning Rust Files

```css
/* input.css - Tailwind v4 */
@import "tailwindcss";
@source "../src/**/*.rs";
@source "../desktop/src/**/*.rs";
```

```js
// tailwind.config.js - Tailwind v3
module.exports = {
  content: [
    "./src/**/*.rs",
    "./desktop/src/**/*.rs",
  ],
}
```

### Ignoring Paths

```css
@import "tailwindcss";
@source not "../src/generated";
@source not "../tests";
```

## Quick Checklist

When a Tailwind class doesn't work:

1. **Is the class in the CSS?**
   ```bash
   grep "\.my-class" desktop/assets/tailwind.css
   ```

2. **Is the class spelled correctly?**
   - Check [Tailwind docs](https://tailwindcss.com/docs)
   - Note: `inset-0` vs `inset-y-0` vs `top-0 left-0 right-0 bottom-0`

3. **Is Tailwind scanning the right files?**
   - Check `content` in config
   - Ensure `.rs` files are included

4. **Is the class written as a complete string?**
   - No interpolation: `"text-{color}-500"` ❌
   - Complete strings: `"text-red-500"` ✓

5. **Does the class need safelisting?**
   - Runtime-generated classes need explicit safelist

## References

- [Tailwind Content Configuration](https://tailwindcss.com/docs/content-configuration)
- [Tailwind Safelist](https://tailwindcss.com/docs/content-configuration#safelisting-classes)
- [Dioxus Tailwind Template](https://github.com/LyonSyonII/dioxus-tailwindcss)
