use gtk4::prelude::*;
use gtk4::{Align, Orientation, ScrolledWindow};
use libadwaita::prelude::{AdwApplicationWindowExt, ExpanderRowExt};
use libadwaita as adw;
use serde_json::Value;

const APP_ID: &str = "org.example.ErrorAlert";

pub fn open(errors: Vec<Value>) {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(move |app| {
        build_window(app, &errors);
    });
    let empty: Vec<String> = vec![];
    app.run_with_args(&empty);
}

fn build_window(app: &adw::Application, errors: &[Value]) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Errors")
        .default_width(600)
        .default_height(450)
        .resizable(true)
        .build();

    let main_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    // Header
    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new("Errors", &format!("{} error(s)", errors.len())))
        .build();
    main_box.append(&header);

    // Scrollable error list
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    let list_box = gtk4::ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(vec!["boxed-list"])
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    for (i, error) in errors.iter().enumerate() {
        list_box.append(&create_error_row(i, error));
    }

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    // Buttons
    let button_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::End)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let cancel_btn = gtk4::Button::with_label("Cancel");
    let report_btn = gtk4::Button::with_label("Report All");
    report_btn.add_css_class("suggested-action");

    button_box.append(&cancel_btn);
    button_box.append(&report_btn);
    main_box.append(&button_box);

    // Actions
    let errors_clone = errors.to_vec();
    report_btn.connect_clicked(move |_| {
        println!("=== ERROR REPORT ===");
        for (i, e) in errors_clone.iter().enumerate() {
            println!("--- Error #{} ---", i + 1);
            println!("{}", serde_json::to_string_pretty(e).unwrap_or_default());
        }
        println!("=== END REPORT ===");
    });

    let win = window.clone();
    cancel_btn.connect_clicked(move |_| win.close());

    window.set_content(Some(&main_box));
    window.present();
}

fn create_error_row(index: usize, error: &Value) -> adw::ExpanderRow {
    let title = format!("Error #{}", index + 1);
    let subtitle = get_preview(error);

    let row = adw::ExpanderRow::builder()
        .title(&title)
        .subtitle(&subtitle)
        .build();

    // Expanded content: full JSON
    let content = gtk4::ListBoxRow::builder()
        .activatable(false)
        .selectable(false)
        .build();

    let label = gtk4::Label::builder()
        .label(&serde_json::to_string_pretty(error).unwrap_or_default())
        .wrap(true)
        .xalign(0.0)
        .selectable(true)
        .css_classes(vec!["monospace"])
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    content.set_child(Some(&label));
    row.add_row(&content);

    row
}

fn get_preview(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            map.iter()
                .take(2)
                .map(|(k, v)| format!("{}: {}", k, short_value(v)))
                .collect::<Vec<_>>()
                .join(" | ")
        }
        _ => short_value(value),
    }
}

fn short_value(v: &Value) -> String {
    match v {
        Value::String(s) if s.len() > 30 => format!("{}...", &s[..30]),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(a) => format!("[{}]", a.len()),
        Value::Object(o) => format!("{{{}...}}", o.len()),
    }
}
