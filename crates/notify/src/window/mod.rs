pub mod messages;
pub mod model;
pub mod report;

pub use model::Modal;

use adw::prelude::*;
use gtk::prelude::*;
use relm4::*;

use messages::{CmdOut, Input, Output};
use model::{App, Widgets};

impl Component for App {
    type Init = Modal;
    type Input = Input;
    type Output = Output;
    type CommandOutput = CmdOut;
    type Root = adw::ApplicationWindow;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        adw::ApplicationWindow::builder()
            .title("Crash Reporter")
            .default_width(480)
            .build()
    }

    fn init(
        error: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = gtk::TextBuffer::new(None);
        buffer.set_text(
            &serde_json::to_string_pretty(&error).unwrap_or_else(|_| "Invalid data".to_string()),
        );

        let textview = gtk::TextView::builder()
            .buffer(&buffer)
            .monospace(true)
            .editable(false)
            .cursor_visible(false)
            .hexpand(true)
            .vexpand(false)
            .top_margin(10)
            .bottom_margin(10)
            .left_margin(12)
            .right_margin(12)
            .build();

        let scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(false)
            .propagate_natural_height(true)
            .margin_start(16)
            .margin_end(16)
            .margin_bottom(12)
            .build();
        scroll.add_css_class("card");
        scroll.set_child(Some(&textview));

        relm4::view! {
            toolbar_view = adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 0,

                    gtk::Label {
                        set_label: &error.message,
                        set_xalign: 0.0,
                        set_margin_top: 12,
                        set_margin_bottom: 8,
                        set_margin_start: 16,
                        set_margin_end: 16,
                        add_css_class: "title-4",
                    },

                    append: &scroll,

                    append: progress = &gtk::ProgressBar {
                        set_visible: false,
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_margin_bottom: 4,
                        set_pulse_step: 0.1,
                    },

                    append: label = &gtk::Label {
                        set_label: "",
                        set_margin_bottom: 8,
                        add_css_class: "caption",
                        add_css_class: "dim-label",
                    },

                    append: button = &gtk::Button {
                        set_label: "Send Report",
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_margin_top: 8,
                        add_css_class: "suggested-action",
                        add_css_class: "pill",
                        connect_clicked => Input::Report,
                    },

                    append: button_close = &gtk::Button {
                        set_label: "Close",
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_margin_top: 4,
                        set_margin_bottom: 16,
                        add_css_class: "pill",
                        connect_clicked => Input::Dismiss,
                    },
                },
            }
        }

        root.set_content(Some(&toolbar_view));

        ComponentParts {
            model: App::default(),
            widgets: Widgets {
                button,
                button_close,
                progress,
                label,
                textview,
                scroll,
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            Input::Dismiss => root.close(),
            Input::Report => {
                self.computing = true;
                report::run(sender);
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match &message {
            CmdOut::Finished { .. } | CmdOut::Error(_) => self.computing = false,
            _ => {}
        }
        self.task = Some(message);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.button.set_sensitive(!self.computing);
        widgets.button_close.set_sensitive(true);

        if let Some(ref task) = self.task {
            match task {
                CmdOut::Progress { fraction, message } => {
                    widgets.scroll.set_visible(false);
                    widgets.textview.set_visible(false);
                    widgets.button.set_visible(false);
                    widgets.progress.set_visible(true);
                    widgets.progress.set_fraction(*fraction);
                    widgets.label.set_label(message);
                    widgets.button_close.set_label("Cancel");
                }
                CmdOut::Finished { bytes } => {
                    widgets.scroll.set_visible(false);
                    widgets.textview.set_visible(false);
                    widgets.progress.set_fraction(1.0);
                    widgets.progress.set_visible(true);
                    widgets.button.set_visible(false);
                    widgets
                        .label
                        .set_label(&format!("Sent — {:.1} KB uploaded", *bytes as f64 / 1024.0));
                    widgets.button_close.set_label("Close");
                }
                CmdOut::Error(e) => {
                    widgets.scroll.set_visible(true);
                    widgets.progress.set_visible(false);
                    widgets.textview.set_visible(true);
                    widgets.label.set_label(&format!("Error: {e}"));
                    widgets.button.set_visible(true);
                    widgets.button.set_label("Retry");
                    widgets.button_close.set_label("Close");
                }
            }
        }
    }
}
