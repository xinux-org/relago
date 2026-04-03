pub mod messages;
pub mod model;
pub mod report;

pub use model::Modal;

use adw::prelude::*;
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
        let grid = gtk::Grid::builder()
            .row_spacing(8)
            .column_spacing(16)
            .margin_start(4)
            .margin_end(4)
            .build();

        for (i, (key, val)) in [
            ("Unit", error.unit.as_str()),
            ("Executable", error.exe.as_str()),
            ("Message", error.message.as_str()),
        ]
        .into_iter()
        .enumerate()
        {
            let k = gtk::Label::builder()
                .label(key)
                .xalign(0.0)
                .width_chars(12)
                .build();
            k.add_css_class("dim-label");
            k.add_css_class("caption");

            let v = gtk::Label::builder()
                .label(val)
                .xalign(0.0)
                .hexpand(true)
                .ellipsize(gtk::pango::EllipsizeMode::Middle)
                .selectable(true)
                .build();
            v.add_css_class("monospace");

            grid.attach(&k, 0, i as i32, 1, 1);
            grid.attach(&v, 1, i as i32, 1, 1);
        }

        let scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(false)
            .propagate_natural_height(true)
            .margin_start(16)
            .margin_end(16)
            .margin_bottom(12)
            .child(&grid)
            .build();
        scroll.add_css_class("card");

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
                        set_margin_top: 8,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_margin_top: 4,
                        set_margin_bottom: 12,

                        append: label = &gtk::Label {
                            set_hexpand: true,
                            set_xalign: 0.0,
                            add_css_class: "caption",
                            add_css_class: "dim-label",
                        },

                        append: label_pct = &gtk::Label {
                            set_xalign: 1.0,
                            add_css_class: "caption",
                            add_css_class: "monospace",
                        },
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
                label_pct,
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
                    widgets.button.set_visible(false);
                    widgets.progress.set_visible(true);
                    widgets.progress.set_fraction(*fraction);
                    widgets.label.set_label(message);
                    widgets
                        .label_pct
                        .set_label(&format!("{:.0}%", fraction * 100.0));
                    widgets.button_close.set_label("Cancel");
                }
                CmdOut::Finished { bytes } => {
                    widgets.scroll.set_visible(false);
                    widgets.progress.set_fraction(1.0);
                    widgets.label.set_label("Sent successfully");
                    widgets
                        .label_pct
                        .set_label(&format!("{:.1} KB", *bytes as f64 / 1024.0));
                    widgets.button.set_visible(false);
                    widgets.button_close.set_label("Close");
                }
                CmdOut::Error(e) => {
                    widgets.scroll.set_visible(true);
                    widgets.progress.set_visible(false);
                    widgets.label.set_label(&format!("Error: {e}"));
                    widgets.button.set_visible(true);
                    widgets.button.set_label("Retry");
                    widgets.button_close.set_label("Close");
                }
            }
        }
    }
}
