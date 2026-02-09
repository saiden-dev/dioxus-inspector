//! Inspector Playground - Dioxus desktop app with inspector bridge.

mod state;

use dioxus::prelude::*;
use dioxus_inspector::{EvalResponse, start_bridge};
use state::AppState;

const BRIDGE_PORT: u16 = 9999;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Inspector Playground");
    dioxus::launch(app);
}

fn app() -> Element {
    let state = use_signal(AppState::new);

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
