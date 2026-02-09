//! Inspector Playground - Component Showcase

use dioxus::desktop::{tao::dpi::LogicalSize, window, Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_inspector::{start_bridge, EvalResponse};

const BRIDGE_PORT: u16 = 9999;

fn main() {
    let window = WindowBuilder::new()
        .with_title("Component Showcase")
        .with_inner_size(LogicalSize::new(900, 700));

    let config = Config::new().with_window(window);
    LaunchBuilder::desktop().with_cfg(config).launch(app);
}

fn apply_dev_options() {
    let fullscreen = std::env::var("DI_FULLSCREEN").is_ok();
    let monitor_index: usize = std::env::var("DI_MONITOR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if fullscreen {
        let monitors: Vec<_> = window().available_monitors().collect();
        if let Some(monitor) = monitors.get(monitor_index).or(monitors.first()) {
            window().set_outer_position(monitor.position());
            window().set_fullscreen(true);
        }
    }
}

fn app() -> Element {
    use_hook(apply_dev_options);

    use_effect(|| {
        let mut eval_rx = start_bridge(BRIDGE_PORT, "showcase");
        spawn(async move {
            while let Some(cmd) = eval_rx.recv().await {
                let response = match document::eval(&cmd.script).await {
                    Ok(val) => EvalResponse::success(val.to_string()),
                    Err(e) => EvalResponse::error(e.to_string()),
                };
                let _ = cmd.response_tx.send(response);
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }

        div {
            id: "app",
            class: "min-h-screen bg-slate-900 text-white p-8",

            // Header
            header { id: "header", class: "mb-8 text-center",
                h1 { class: "text-3xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent",
                    "Component Showcase"
                }
                p { class: "text-slate-400 mt-2", "10 UI primitives • Inspector :9999" }
            }

            // Grid layout
            div { class: "grid grid-cols-2 gap-6 max-w-4xl mx-auto",

                // 1. Buttons
                Section { id: "buttons", title: "Buttons",
                    div { class: "flex flex-wrap gap-3",
                        Button { variant: "primary", "Primary" }
                        Button { variant: "secondary", "Secondary" }
                        Button { variant: "outline", "Outline" }
                        Button { variant: "ghost", "Ghost" }
                        Button { variant: "danger", "Danger" }
                    }
                }

                // 2. Inputs
                Section { id: "inputs", title: "Inputs",
                    div { class: "space-y-3",
                        Input { placeholder: "Default input..." }
                        Input { placeholder: "With icon...", icon: "🔍" }
                        Input { placeholder: "Disabled", disabled: true }
                    }
                }

                // 3. Cards
                Section { id: "cards", title: "Cards",
                    div { class: "flex gap-3",
                        Card { title: "Basic", "Simple card content" }
                        Card { title: "Featured", featured: true, "Highlighted card" }
                    }
                }

                // 4. Badges
                Section { id: "badges", title: "Badges",
                    div { class: "flex flex-wrap gap-2",
                        Badge { variant: "default", "Default" }
                        Badge { variant: "success", "Success" }
                        Badge { variant: "warning", "Warning" }
                        Badge { variant: "error", "Error" }
                        Badge { variant: "info", "Info" }
                    }
                }

                // 5. Avatars
                Section { id: "avatars", title: "Avatars",
                    div { class: "flex items-center gap-3",
                        Avatar { size: "sm", "🦀" }
                        Avatar { size: "md", "🚀" }
                        Avatar { size: "lg", "⚡" }
                        AvatarGroup {
                            Avatar { size: "md", "A" }
                            Avatar { size: "md", "B" }
                            Avatar { size: "md", "C" }
                        }
                    }
                }

                // 6. Toggles
                Section { id: "toggles", title: "Toggles",
                    div { class: "space-y-3",
                        Toggle { label: "Notifications", checked: true }
                        Toggle { label: "Dark mode", checked: true }
                        Toggle { label: "Auto-save", checked: false }
                    }
                }

                // 7. Progress
                Section { id: "progress", title: "Progress",
                    div { class: "space-y-3",
                        Progress { value: 25, label: "Loading..." }
                        Progress { value: 60, variant: "success" }
                        Progress { value: 90, variant: "warning" }
                    }
                }

                // 8. Alerts
                Section { id: "alerts", title: "Alerts",
                    div { class: "space-y-2",
                        Alert { variant: "info", "New version available!" }
                        Alert { variant: "success", "Changes saved." }
                        Alert { variant: "warning", "Check your settings." }
                    }
                }

                // 9. Stats
                Section { id: "stats", title: "Stats",
                    div { class: "flex gap-3",
                        Stat { value: "2.4k", label: "Users", trend: "+12%" }
                        Stat { value: "847", label: "Orders", trend: "+5%" }
                        Stat { value: "99%", label: "Uptime" }
                    }
                }

                // 10. Tags
                Section { id: "tags", title: "Tags",
                    div { class: "flex flex-wrap gap-2",
                        Tag { "Rust" }
                        Tag { "Dioxus" }
                        Tag { "Desktop" }
                        Tag { closable: true, "Removable" }
                        Tag { variant: "outline", "Outline" }
                    }
                }
            }
        }
    }
}

// ============ COMPONENTS ============

#[component]
fn Section(id: &'static str, title: &'static str, children: Element) -> Element {
    rsx! {
        section { id: id, class: "bg-slate-800 rounded-xl p-5",
            h2 { class: "text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wide", "{title}" }
            {children}
        }
    }
}

#[component]
fn Button(variant: Option<&'static str>, children: Element) -> Element {
    let class = match variant.unwrap_or("primary") {
        "primary" => "px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition",
        "secondary" => "px-4 py-2 bg-slate-600 hover:bg-slate-500 rounded-lg font-medium transition",
        "outline" => "px-4 py-2 border border-slate-500 hover:bg-slate-700 rounded-lg font-medium transition",
        "ghost" => "px-4 py-2 hover:bg-slate-700 rounded-lg font-medium transition",
        "danger" => "px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg font-medium transition",
        _ => "px-4 py-2 bg-blue-600 rounded-lg font-medium",
    };
    rsx! { button { class: class, {children} } }
}

#[component]
fn Input(placeholder: &'static str, icon: Option<&'static str>, disabled: Option<bool>) -> Element {
    let disabled = disabled.unwrap_or(false);
    let base = "w-full bg-slate-700 border border-slate-600 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500 transition";
    let class = if disabled { format!("{} opacity-50 cursor-not-allowed", base) } else { base.to_string() };

    rsx! {
        div { class: "relative",
            if let Some(ico) = icon {
                span { class: "absolute left-3 top-2.5 text-slate-400", "{ico}" }
                input { class: "{class} pl-10", placeholder: placeholder, disabled: disabled }
            } else {
                input { class: class, placeholder: placeholder, disabled: disabled }
            }
        }
    }
}

#[component]
fn Card(title: &'static str, featured: Option<bool>, children: Element) -> Element {
    let border = if featured.unwrap_or(false) { "border-blue-500" } else { "border-slate-700" };
    rsx! {
        div { class: "bg-slate-700 border {border} rounded-lg p-4 flex-1",
            h3 { class: "font-semibold mb-2", "{title}" }
            p { class: "text-slate-400 text-sm", {children} }
        }
    }
}

#[component]
fn Badge(variant: Option<&'static str>, children: Element) -> Element {
    let class = match variant.unwrap_or("default") {
        "success" => "px-2.5 py-0.5 bg-green-600/20 text-green-400 rounded-full text-xs font-medium",
        "warning" => "px-2.5 py-0.5 bg-yellow-600/20 text-yellow-400 rounded-full text-xs font-medium",
        "error" => "px-2.5 py-0.5 bg-red-600/20 text-red-400 rounded-full text-xs font-medium",
        "info" => "px-2.5 py-0.5 bg-blue-600/20 text-blue-400 rounded-full text-xs font-medium",
        _ => "px-2.5 py-0.5 bg-slate-600 text-slate-300 rounded-full text-xs font-medium",
    };
    rsx! { span { class: class, {children} } }
}

#[component]
fn Avatar(size: Option<&'static str>, children: Element) -> Element {
    let class = match size.unwrap_or("md") {
        "sm" => "w-8 h-8 rounded-full bg-slate-600 flex items-center justify-center text-sm",
        "lg" => "w-14 h-14 rounded-full bg-slate-600 flex items-center justify-center text-2xl",
        _ => "w-10 h-10 rounded-full bg-slate-600 flex items-center justify-center text-lg",
    };
    rsx! { div { class: class, {children} } }
}

#[component]
fn AvatarGroup(children: Element) -> Element {
    rsx! { div { class: "flex -space-x-3", {children} } }
}

#[component]
fn Toggle(label: &'static str, checked: bool) -> Element {
    let bg = if checked { "bg-blue-600" } else { "bg-slate-600" };
    let pos = if checked { "translate-x-5" } else { "translate-x-0" };
    rsx! {
        label { class: "flex items-center justify-between cursor-pointer",
            span { class: "text-sm", "{label}" }
            div { class: "w-11 h-6 {bg} rounded-full relative transition",
                div { class: "w-5 h-5 bg-white rounded-full absolute top-0.5 left-0.5 transition transform {pos}" }
            }
        }
    }
}

#[component]
fn Progress(value: u8, label: Option<&'static str>, variant: Option<&'static str>) -> Element {
    let color = match variant.unwrap_or("default") {
        "success" => "bg-green-500",
        "warning" => "bg-yellow-500",
        _ => "bg-blue-500",
    };
    rsx! {
        div {
            if let Some(lbl) = label {
                p { class: "text-xs text-slate-400 mb-1", "{lbl}" }
            }
            div { class: "h-2 bg-slate-700 rounded-full overflow-hidden",
                div { class: "h-full {color} transition-all", style: "width: {value}%" }
            }
        }
    }
}

#[component]
fn Alert(variant: &'static str, children: Element) -> Element {
    let (bg, border, icon) = match variant {
        "success" => ("bg-green-900/30", "border-green-700", "✓"),
        "warning" => ("bg-yellow-900/30", "border-yellow-700", "⚠"),
        _ => ("bg-blue-900/30", "border-blue-700", "ℹ"),
    };
    rsx! {
        div { class: "{bg} border {border} rounded-lg px-4 py-2 flex items-center gap-2 text-sm",
            span { "{icon}" }
            {children}
        }
    }
}

#[component]
fn Stat(value: &'static str, label: &'static str, trend: Option<&'static str>) -> Element {
    rsx! {
        div { class: "bg-slate-700 rounded-lg p-3 text-center flex-1",
            p { class: "text-xl font-bold", "{value}" }
            p { class: "text-xs text-slate-400", "{label}" }
            if let Some(t) = trend {
                p { class: "text-xs text-green-400 mt-1", "{t}" }
            }
        }
    }
}

#[component]
fn Tag(variant: Option<&'static str>, closable: Option<bool>, children: Element) -> Element {
    let base = if variant == Some("outline") {
        "inline-flex items-center gap-1 px-3 py-1 border border-slate-500 rounded-full text-sm"
    } else {
        "inline-flex items-center gap-1 px-3 py-1 bg-slate-700 rounded-full text-sm"
    };
    rsx! {
        span { class: base,
            {children}
            if closable.unwrap_or(false) {
                span { class: "text-slate-400 hover:text-white cursor-pointer", "×" }
            }
        }
    }
}
