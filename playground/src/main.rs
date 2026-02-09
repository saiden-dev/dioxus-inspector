//! Inspector Playground - Simple UI showcase with inspector bridge.

use dioxus::desktop::{tao::dpi::LogicalSize, Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_inspector::{start_bridge, EvalResponse};

const BRIDGE_PORT: u16 = 9999;

fn main() {
    let window = WindowBuilder::new()
        .with_title("Profile Card")
        .with_inner_size(LogicalSize::new(480, 640));

    let config = Config::new().with_window(window);

    LaunchBuilder::desktop().with_cfg(config).launch(app);
}

fn app() -> Element {
    // Start inspector bridge
    use_effect(|| {
        let mut eval_rx = start_bridge(BRIDGE_PORT, "playground");

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

        div { class: "min-h-screen bg-gradient-to-br from-slate-900 to-slate-800 flex items-center justify-center p-8",
            ProfileCard {}
        }
    }
}

#[component]
fn ProfileCard() -> Element {
    rsx! {
        article {
            id: "profile-card",
            class: "bg-slate-800 rounded-2xl shadow-2xl overflow-hidden max-w-sm w-full",

            // Header with gradient
            header {
                id: "card-header",
                class: "h-24 bg-gradient-to-r from-blue-600 to-purple-600"
            }

            // Avatar
            div { class: "flex justify-center -mt-12",
                div {
                    id: "avatar",
                    class: "w-24 h-24 rounded-full bg-slate-700 border-4 border-slate-800 flex items-center justify-center text-4xl",
                    "🦀"
                }
            }

            // Content
            div { class: "text-center px-6 py-4",
                h1 {
                    id: "name",
                    class: "text-xl font-bold text-white",
                    "Ferris Developer"
                }
                p {
                    id: "title",
                    class: "text-blue-400 text-sm",
                    "Rust Enthusiast"
                }
                p {
                    id: "bio",
                    class: "text-slate-400 text-sm mt-3 leading-relaxed",
                    "Building desktop apps with Dioxus. Debugging with the Inspector bridge."
                }
            }

            // Stats
            div {
                id: "stats",
                class: "flex justify-around py-4 border-t border-slate-700",

                StatItem { label: "Projects", value: "42" }
                StatItem { label: "Stars", value: "1.2k" }
                StatItem { label: "Commits", value: "847" }
            }

            // Action buttons
            div { class: "px-6 pb-6 flex gap-3",
                button {
                    id: "btn-follow",
                    class: "flex-1 bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition",
                    "Follow"
                }
                button {
                    id: "btn-message",
                    class: "flex-1 bg-slate-700 hover:bg-slate-600 text-white font-medium py-2 px-4 rounded-lg transition",
                    "Message"
                }
            }

            // Footer
            footer {
                id: "card-footer",
                class: "px-6 py-3 bg-slate-900/50 text-center",
                code { class: "text-xs text-green-400", "Inspector :9999" }
            }
        }
    }
}

#[component]
fn StatItem(label: &'static str, value: &'static str) -> Element {
    rsx! {
        div { class: "text-center",
            p { class: "text-white font-bold", "{value}" }
            p { class: "text-slate-500 text-xs", "{label}" }
        }
    }
}
