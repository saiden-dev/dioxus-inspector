//! Inspector Playground - Dioxus desktop app with inspector bridge.

mod state;

use dioxus::desktop::{
    tao::dpi::LogicalSize,
    window, Config, WindowBuilder,
};
use dioxus::prelude::*;
use dioxus_inspector::{EvalResponse, start_bridge};
use state::AppState;

const BRIDGE_PORT: u16 = 9999;

/// Dev options read from environment:
/// - DI_FULLSCREEN=1      Start in fullscreen mode
/// - DI_MONITOR=N         Use monitor N (0=primary, 1=secondary, etc.)
/// - DI_LIST_MONITORS=1   Print available monitors and exit
struct DevOptions {
    fullscreen: bool,
    monitor_index: usize,
    list_monitors: bool,
}

impl DevOptions {
    fn from_env() -> Self {
        Self {
            fullscreen: std::env::var("DI_FULLSCREEN").is_ok(),
            monitor_index: std::env::var("DI_MONITOR")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            list_monitors: std::env::var("DI_LIST_MONITORS").is_ok(),
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Inspector Playground");

    let window = WindowBuilder::new()
        .with_title("Inspector Playground")
        .with_maximizable(true)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(800, 600));

    let config = Config::new().with_window(window);

    LaunchBuilder::desktop().with_cfg(config).launch(app);
}

fn app() -> Element {
    let state = use_signal(AppState::new);

    // Dev options: DI_FULLSCREEN=1 DI_MONITOR=N DI_LIST_MONITORS=1
    // Must be done after app starts to access monitors via window()
    use_hook(|| {
        let opts = DevOptions::from_env();
        let monitors: Vec<_> = window().available_monitors().collect();

        if opts.list_monitors {
            println!("Available monitors:");
            for (i, monitor) in monitors.iter().enumerate() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let name = monitor.name().unwrap_or_else(|| "Unknown".to_string());
                // Show effective resolution (UI looks like) for HiDPI displays
                let effective_w = (size.width as f64 / scale).round() as u32;
                let effective_h = (size.height as f64 / scale).round() as u32;
                println!(
                    "  {}: {} ({}x{} @{}x)",
                    i, name, effective_w, effective_h, scale
                );
            }
            std::process::exit(0);
        }

        if opts.fullscreen {
            let target_monitor = monitors.get(opts.monitor_index).or(monitors.first());

            if let Some(monitor) = target_monitor {
                // Move window to the target monitor, then fullscreen
                let pos = monitor.position();
                window().set_outer_position(pos);
                window().set_fullscreen(true);
            }
        }
    });

    // Start the inspector bridge and handle eval commands
    use_effect(move || {
        let mut eval_rx = start_bridge(BRIDGE_PORT, "playground");

        spawn(async move {
            while let Some(cmd) = eval_rx.recv().await {
                tracing::debug!("Executing: {}", &cmd.script[..cmd.script.len().min(50)]);

                let result = document::eval(&cmd.script).await;
                let response = match result {
                    Ok(val) => {
                        let s = val.to_string();
                        EvalResponse::success(s)
                    }
                    Err(e) => EvalResponse::error(e.to_string()),
                };

                let _ = cmd.response_tx.send(response);
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./assets/tailwind.css") }

        div { class: "min-h-screen bg-gray-900 text-white p-8",
            div { class: "max-w-2xl mx-auto",
                Header {}
                CounterSection { state }
                MessageSection { state }
                InspectorInfo {}
            }
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header { class: "mb-8",
            h1 { class: "text-3xl font-bold text-blue-400",
                "Inspector Playground"
            }
            p { class: "text-gray-400 mt-2",
                "A Dioxus desktop app with inspector bridge enabled"
            }
        }
    }
}

#[component]
fn CounterSection(state: Signal<AppState>) -> Element {
    let count = state.read().counter.read().to_string();

    rsx! {
        section {
            id: "counter-section",
            class: "bg-gray-800 rounded-lg p-6 mb-6",

            h2 { class: "text-xl font-semibold mb-4", "Counter Demo" }

            div { class: "flex items-center gap-4",
                button {
                    class: "btn-primary",
                    onclick: move |_| state.write().decrement(),
                    "-"
                }
                span {
                    id: "counter-value",
                    class: "text-4xl font-mono w-20 text-center",
                    "{count}"
                }
                button {
                    class: "btn-primary",
                    onclick: move |_| state.write().increment(),
                    "+"
                }
                button {
                    class: "btn-secondary ml-4",
                    onclick: move |_| state.write().reset(),
                    "Reset"
                }
            }
        }
    }
}

#[component]
fn MessageSection(state: Signal<AppState>) -> Element {
    let message = state.read().message.read().clone();

    rsx! {
        section {
            id: "message-section",
            class: "bg-gray-800 rounded-lg p-6 mb-6",

            h2 { class: "text-xl font-semibold mb-4", "Message" }

            p {
                id: "message-display",
                class: "text-lg text-gray-300 mb-4",
                "{message}"
            }

            input {
                r#type: "text",
                class: "input-field w-full",
                placeholder: "Type a new message...",
                value: "{message}",
                oninput: move |evt| state.write().set_message(evt.value())
            }
        }
    }
}

#[component]
fn InspectorInfo() -> Element {
    rsx! {
        section { class: "bg-gray-800 rounded-lg p-6 border border-blue-500/30",
            h2 { class: "text-xl font-semibold mb-4 text-blue-400",
                "Inspector Bridge Active"
            }
            p { class: "text-gray-400 mb-2",
                "The inspector bridge is listening on:"
            }
            code { class: "text-green-400 font-mono",
                "http://127.0.0.1:{BRIDGE_PORT}"
            }
            p { class: "text-gray-500 mt-4 text-sm",
                "Use the dioxus-mcp server to connect from Claude Code."
            }
        }
    }
}
