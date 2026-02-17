//! Inspector Playground - Calendar Demo

use dioxus::desktop::{window, Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_inspector::{start_bridge, EvalResponse};
use dioxus_primitives::calendar::{
    Calendar, CalendarGrid, CalendarHeader, CalendarMonthTitle, CalendarNavigation,
    CalendarNextMonthButton, CalendarPreviousMonthButton,
};
use time::Date;

const BRIDGE_PORT: u16 = 9999;

fn main() {
    let window = WindowBuilder::new().with_title("Calendar Demo");
    let config = Config::new().with_window(window);
    LaunchBuilder::desktop().with_cfg(config).launch(app);
}

fn app() -> Element {
    use_hook(|| window().set_fullscreen(true));

    use_effect(|| {
        let mut eval_rx = start_bridge(BRIDGE_PORT, "calendar");
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

    let mut selected_date = use_signal(|| None::<Date>);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        document::Style { {CALENDAR_STYLES} }

        div {
            id: "app",
            class: "flex flex-col items-center justify-center min-h-screen bg-black gap-8",

            h1 { class: "text-white text-3xl font-sans", "Pick a Date" }

            Calendar {
                class: "calendar",
                selected_date: selected_date(),
                on_date_change: move |date: Option<Date>| selected_date.set(date),
                CalendarHeader {
                    CalendarNavigation {
                        CalendarPreviousMonthButton { class: "calendar-nav-btn", "◀" }
                        CalendarMonthTitle { class: "calendar-title" }
                        CalendarNextMonthButton { class: "calendar-nav-btn", "▶" }
                    }
                }
                CalendarGrid {}
            }

            if let Some(date) = selected_date() {
                p { class: "text-white text-xl font-sans", "Selected: {date}" }
            }
        }
    }
}

const CALENDAR_STYLES: &str = r#"
.calendar {
    background: #1e293b;
    border-radius: 12px;
    padding: 1.5rem;
    color: white;
    font-family: system-ui, sans-serif;
}

.calendar-nav-btn {
    background: #334155;
    border: none;
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 1rem;
}

.calendar-nav-btn:hover {
    background: #475569;
}

.calendar-title {
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
    padding: 0 1rem;
}

.calendar-navigation {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
}

/* Table-based grid */
.calendar-grid {
    width: 100%;
    border-collapse: separate;
    border-spacing: 4px;
}

.calendar-grid-header {
    /* tr element */
}

.calendar-grid-day-header {
    text-align: center;
    color: #94a3b8;
    font-size: 0.875rem;
    font-weight: 400;
    padding: 0.5rem;
    width: 2.5rem;
}

.calendar-grid-body {
    /* tbody element */
}

.calendar-grid-week {
    /* tr element */
}

.calendar-grid-week td {
    padding: 2px;
}

.calendar-grid-cell {
    width: 2.5rem;
    height: 2.5rem;
    border: none;
    background: transparent;
    color: white;
    cursor: pointer;
    border-radius: 8px;
    font-size: 0.875rem;
}

.calendar-grid-cell:hover {
    background: #334155;
}

.calendar-grid-cell[data-selected="true"] {
    background: #3b82f6;
    color: white;
}

.calendar-grid-cell[data-today="true"]:not([data-selected="true"]) {
    background: #475569;
}

.calendar-grid-cell[data-month="last"],
.calendar-grid-cell[data-month="next"] {
    color: #64748b;
}
"#;
