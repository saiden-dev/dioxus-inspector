//! Inspector Playground - Dioxus Primitives Showcase

use dioxus::desktop::{tao::dpi::LogicalSize, window, Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_inspector::{start_bridge, EvalResponse};
use dioxus_primitives::{
    accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
    checkbox::{Checkbox, CheckboxIndicator},
    progress::{Progress, ProgressIndicator},
    separator::Separator,
    slider::{Slider, SliderRange, SliderThumb, SliderTrack},
    switch::{Switch, SwitchThumb},
    tabs::{TabContent, TabList, TabTrigger, Tabs},
    toggle::Toggle,
    toggle_group::{ToggleGroup, ToggleItem},
};
use dioxus_terminal::{Terminal, Theme};

const BRIDGE_PORT: u16 = 9999;

fn main() {
    let window = WindowBuilder::new()
        .with_title("Dioxus Primitives Showcase")
        .with_inner_size(LogicalSize::new(1000, 1100));

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
        let mut eval_rx = start_bridge(BRIDGE_PORT, "primitives");
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
        document::Style { {PRIMITIVE_STYLES} }

        div {
            id: "app",
            class: "min-h-screen bg-slate-900 text-white p-8",

            // Header
            header { id: "header", class: "mb-8 text-center",
                h1 { class: "text-3xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent",
                    "Dioxus Primitives"
                }
                p { class: "text-slate-400 mt-2", "Headless UI components • Inspector :9999" }
            }

            // Grid layout
            div { class: "grid grid-cols-2 gap-6 max-w-4xl mx-auto",

                // Switch
                Section { id: "switch", title: "Switch",
                    SwitchDemo {}
                }

                // Checkbox
                Section { id: "checkbox", title: "Checkbox",
                    CheckboxDemo {}
                }

                // Progress
                Section { id: "progress", title: "Progress",
                    ProgressDemo {}
                }

                // Slider
                Section { id: "slider", title: "Slider",
                    SliderDemo {}
                }

                // Toggle
                Section { id: "toggle", title: "Toggle",
                    ToggleDemo {}
                }

                // Toggle Group
                Section { id: "toggle-group", title: "Toggle Group",
                    ToggleGroupDemo {}
                }

                // Tabs (full width)
                div { class: "col-span-2",
                    Section { id: "tabs", title: "Tabs",
                        TabsDemo {}
                    }
                }

                // Accordion (full width)
                div { class: "col-span-2",
                    Section { id: "accordion", title: "Accordion",
                        AccordionDemo {}
                    }
                }
            }

            // Separator
            div { class: "max-w-4xl mx-auto my-6",
                Separator { class: "separator" }
            }

            // Terminal section (full width)
            section { id: "terminal", class: "bg-slate-800 rounded-xl p-5 max-w-4xl mx-auto",
                h2 { class: "text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wide", "Terminal" }
                div { class: "rounded-lg overflow-hidden",
                    Terminal {
                        command: "/bin/bash",
                        args: vec!["--login".to_string()],
                        rows: 12,
                        cols: 100,
                        theme: Theme::zinc(),
                        class: "rounded-lg",
                    }
                }
            }
        }
    }
}

// ============ DEMO COMPONENTS ============

#[component]
fn SwitchDemo() -> Element {
    let mut checked1 = use_signal(|| false);
    let mut checked2 = use_signal(|| true);

    rsx! {
        div { class: "space-y-4",
            div { class: "flex items-center justify-between",
                span { class: "text-sm", "Notifications" }
                Switch {
                    checked: checked1(),
                    on_checked_change: move |v| checked1.set(v),
                    class: "switch",
                    SwitchThumb { class: "switch-thumb" }
                }
            }
            div { class: "flex items-center justify-between",
                span { class: "text-sm", "Dark mode" }
                Switch {
                    checked: checked2(),
                    on_checked_change: move |v| checked2.set(v),
                    class: "switch",
                    SwitchThumb { class: "switch-thumb" }
                }
            }
        }
    }
}

#[component]
fn CheckboxDemo() -> Element {
    rsx! {
        div { class: "space-y-3",
            label { class: "flex items-center gap-3 cursor-pointer",
                Checkbox {
                    class: "checkbox",
                    CheckboxIndicator { class: "checkbox-indicator", "✓" }
                }
                span { class: "text-sm", "Accept terms of service" }
            }
            label { class: "flex items-center gap-3 cursor-pointer",
                Checkbox {
                    class: "checkbox",
                    default_checked: dioxus_primitives::checkbox::CheckboxState::Checked,
                    CheckboxIndicator { class: "checkbox-indicator", "✓" }
                }
                span { class: "text-sm", "Subscribe to newsletter" }
            }
        }
    }
}

#[component]
fn ProgressDemo() -> Element {
    let mut value = use_signal(|| 65.0_f64);

    // Animate progress
    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                value.with_mut(|v| {
                    *v += 0.5;
                    if *v > 100.0 {
                        *v = 0.0;
                    }
                });
            }
        });
    });

    let display_value = value().round() as i32;

    rsx! {
        div { class: "space-y-4",
            div {
                p { class: "text-xs text-slate-400 mb-2", "Downloading... {display_value}%" }
                Progress {
                    value: value(),
                    class: "progress",
                    ProgressIndicator { class: "progress-indicator" }
                }
            }
            div {
                p { class: "text-xs text-slate-400 mb-2", "Static (75%)" }
                Progress {
                    value: 75.0_f64,
                    class: "progress",
                    ProgressIndicator { class: "progress-indicator" }
                }
            }
        }
    }
}

#[component]
fn SliderDemo() -> Element {
    let mut value = use_signal(|| None::<dioxus_primitives::slider::SliderValue>);

    rsx! {
        div { class: "space-y-4",
            Slider {
                value: value(),
                default_value: dioxus_primitives::slider::SliderValue::Single(50.0),
                on_value_change: move |v| value.set(Some(v)),
                class: "slider",
                SliderTrack { class: "slider-track",
                    SliderRange { class: "slider-range" }
                    SliderThumb { class: "slider-thumb" }
                }
            }
            p { class: "text-xs text-slate-400 text-center",
                "Value: {value().map(|v| v.to_string()).unwrap_or(\"50\".into())}"
            }
        }
    }
}

#[component]
fn ToggleDemo() -> Element {
    let mut pressed = use_signal(|| false);

    rsx! {
        div { class: "flex items-center gap-4",
            Toggle {
                pressed: pressed(),
                on_pressed_change: move |v| pressed.set(v),
                class: "toggle-single",
                "Bold"
            }
            p { class: "text-xs text-slate-400",
                if pressed() { "On" } else { "Off" }
            }
        }
    }
}

#[component]
fn ToggleGroupDemo() -> Element {
    rsx! {
        div { class: "space-y-4",
            ToggleGroup {
                horizontal: true,
                class: "toggle-group",
                ToggleItem { index: 0usize, class: "toggle-item", "Left" }
                ToggleItem { index: 1usize, class: "toggle-item", "Center" }
                ToggleItem { index: 2usize, class: "toggle-item", "Right" }
            }
        }
    }
}

#[component]
fn TabsDemo() -> Element {
    rsx! {
        Tabs {
            default_value: "tab1",
            class: "tabs",
            TabList { class: "tab-list",
                TabTrigger { value: "tab1", index: 0usize, class: "tab-trigger", "Account" }
                TabTrigger { value: "tab2", index: 1usize, class: "tab-trigger", "Settings" }
                TabTrigger { value: "tab3", index: 2usize, class: "tab-trigger", "Billing" }
            }
            TabContent { value: "tab1", index: 0usize, class: "tab-content",
                p { "Manage your account settings and preferences." }
            }
            TabContent { value: "tab2", index: 1usize, class: "tab-content",
                p { "Configure application settings and notifications." }
            }
            TabContent { value: "tab3", index: 2usize, class: "tab-content",
                p { "View billing history and manage payment methods." }
            }
        }
    }
}

#[component]
fn AccordionDemo() -> Element {
    rsx! {
        Accordion { class: "accordion",
            AccordionItem { index: 0, class: "accordion-item",
                AccordionTrigger { class: "accordion-trigger", "What is Dioxus?" }
                AccordionContent { class: "accordion-content",
                    p { "Dioxus is a Rust framework for building cross-platform user interfaces." }
                }
            }
            AccordionItem { index: 1, class: "accordion-item",
                AccordionTrigger { class: "accordion-trigger", "What are primitives?" }
                AccordionContent { class: "accordion-content",
                    p { "Primitives are unstyled, accessible UI components that you can customize." }
                }
            }
            AccordionItem { index: 2, class: "accordion-item",
                AccordionTrigger { class: "accordion-trigger", "How do I style them?" }
                AccordionContent { class: "accordion-content",
                    p { "Use CSS with data-* attribute selectors or add your own classes." }
                }
            }
        }
    }
}

// ============ LAYOUT COMPONENTS ============

#[component]
fn Section(id: &'static str, title: &'static str, children: Element) -> Element {
    rsx! {
        section { id: id, class: "bg-slate-800 rounded-xl p-5",
            h2 { class: "text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wide", "{title}" }
            {children}
        }
    }
}

// ============ STYLES ============

const PRIMITIVE_STYLES: &str = r#"
/* Switch */
.switch {
    width: 44px;
    height: 24px;
    background: #475569;
    border-radius: 9999px;
    position: relative;
    cursor: pointer;
    transition: background 0.2s;
    border: none;
}
.switch[data-state="checked"] {
    background: #3b82f6;
}
.switch-thumb {
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 9999px;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 0.2s;
}
.switch[data-state="checked"] .switch-thumb {
    transform: translateX(20px);
}

/* Checkbox */
.checkbox {
    width: 20px;
    height: 20px;
    background: #475569;
    border: 2px solid #64748b;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
}
.checkbox[data-state="checked"] {
    background: #3b82f6;
    border-color: #3b82f6;
}
.checkbox-indicator {
    color: white;
    font-size: 12px;
}

/* Progress */
.progress {
    height: 8px;
    background: #334155;
    border-radius: 9999px;
    overflow: hidden;
}
.progress-indicator {
    height: 100%;
    background: #3b82f6;
    width: var(--progress-value);
    transition: width 0.1s;
}

/* Slider */
.slider {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    height: 20px;
}
.slider-track {
    position: relative;
    height: 6px;
    width: 100%;
    background: #334155;
    border-radius: 9999px;
}
.slider-range {
    position: absolute;
    height: 100%;
    background: #3b82f6;
    border-radius: 9999px;
}
.slider-thumb {
    position: absolute;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 9999px;
    transform: translateX(-50%);
    cursor: grab;
    border: none;
    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}
.slider-thumb:active {
    cursor: grabbing;
}
.slider-thumb:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.5);
}

/* Toggle (single) */
.toggle-single {
    padding: 8px 16px;
    background: #475569;
    border: none;
    color: white;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
}
.toggle-single[data-state="on"] {
    background: #3b82f6;
}

/* Toggle Group */
.toggle-group {
    display: flex;
    background: #334155;
    border-radius: 8px;
    padding: 4px;
}
.toggle-item {
    padding: 8px 16px;
    background: transparent;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.2s;
}
.toggle-item[data-state="on"] {
    background: #3b82f6;
    color: white;
}
.toggle-item:hover:not([data-state="on"]) {
    color: white;
}

/* Tabs */
.tabs {
    width: 100%;
}
.tab-list {
    display: flex;
    border-bottom: 1px solid #334155;
    margin-bottom: 16px;
}
.tab-trigger {
    padding: 12px 20px;
    background: transparent;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: all 0.2s;
}
.tab-trigger[data-state="active"] {
    color: white;
    border-bottom-color: #3b82f6;
}
.tab-trigger:hover:not([data-state="active"]) {
    color: #cbd5e1;
}
.tab-content {
    color: #94a3b8;
}
.tab-content[data-state="inactive"] {
    display: none;
}

/* Accordion */
.accordion {
    width: 100%;
}
.accordion-item {
    border-bottom: 1px solid #334155;
}
.accordion-trigger {
    width: 100%;
    padding: 16px 0;
    background: transparent;
    border: none;
    color: white;
    text-align: left;
    cursor: pointer;
    font-size: 15px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}
.accordion-trigger::after {
    content: "+";
    color: #64748b;
    font-size: 20px;
}
.accordion-item[data-open="true"] .accordion-trigger::after {
    content: "−";
}
.accordion-content {
    overflow: hidden;
    color: #94a3b8;
    font-size: 14px;
    padding-bottom: 16px;
}

/* Separator */
.separator {
    height: 1px;
    background: #334155;
    width: 100%;
}
"#;
