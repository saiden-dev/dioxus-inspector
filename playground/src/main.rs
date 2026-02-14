//! Inspector Playground - Hello World

use dioxus::desktop::{window, Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_inspector::{start_bridge, EvalResponse};

const BRIDGE_PORT: u16 = 9999;

fn main() {
    let window = WindowBuilder::new().with_title("Hello World");
    let config = Config::new().with_window(window);
    LaunchBuilder::desktop().with_cfg(config).launch(app);
}

fn app() -> Element {
    use_hook(|| window().set_fullscreen(true));

    use_effect(|| {
        let mut eval_rx = start_bridge(BRIDGE_PORT, "hello");
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
        document::Style { "html, body {{ margin: 0; padding: 0; overflow: hidden; }}" }
        div {
            id: "app",
            style: "display: flex; align-items: center; justify-content: center; height: 100vh; background: #000; overflow: hidden;",
            h1 {
                style: "color: white; font-size: 3rem; font-family: system-ui, sans-serif;",
                "Hello World!"
            }
        }
    }
}
