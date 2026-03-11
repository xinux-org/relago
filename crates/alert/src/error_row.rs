use gtk4::prelude::*;
use gtk4::{self as gtk, Label, Orientation, pango::WrapMode};
use libadwaita::prelude::ExpanderRowExt;
use libadwaita as adw;
use serde_json::Value;

pub fn create_error_row(index: usize, error: &Value) -> adw::ExpanderRow {
    let title = extract_title(error, index);
    let subtitle = extract_subtitle(error);

    let row = adw::ExpanderRow::builder()
        .title(&title)
        .subtitle(&subtitle)
        .show_enable_switch(false)
        .build();

    let content_row = create_content_row(error);
    row.add_row(&content_row);

    row
}

fn extract_title(error: &Value, index: usize) -> String {
    let title_candidates = ["title", "name", "error", "type", "kind", "code"];

    for candidate in title_candidates {
        if let Some(val) = error.get(candidate) {
            if let Some(s) = val.as_str() {
                return s.to_string();
            } else if let Some(n) = val.as_i64() {
                return format!("{}: {}", candidate, n);
            }
        }
    }

    if let Some(msg) = error.get("message").and_then(|m| m.as_str()) {
        let shorted: String = msg.chars().take(50).collect();
        if msg.len() > 50 {
            return format!("{}...", shorted);
        }
        return shorted;
    }

    format!("Error {}", index + 1)
}

fn extract_subtitle(error: &Value) -> String {
    // Try timestamp first
    if let Some(ts) = error.get("timestamp").and_then(|t| t.as_str()) {
        return ts.to_string();
    }

    // Try source
    if let Some(src) = error.get("source").and_then(|s| s.as_str()) {
        return src.to_string();
    }

    // Try level/severity
    if let Some(level) = error.get("level").and_then(|l| l.as_str()) {
        return level.to_string();
    }

    String::new()
}

/// Create the content row showing full JSON
fn create_content_row(error: &Value) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::builder()
        .activatable(false)
        .selectable(false)
        .build();

    let content_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    // Format JSON with indentation
    let json_text = serde_json::to_string_pretty(error)
        .unwrap_or_else(|_| error.to_string());

    let label = Label::builder()
        .label(&json_text)
        .wrap(true)
        .wrap_mode(WrapMode::WordChar)
        .xalign(0.0)
        .selectable(true)
        .css_classes(vec!["monospace"])
        .build();

    content_box.append(&label);
    row.set_child(Some(&content_box));

    row
}
