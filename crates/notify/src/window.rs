use adw::prelude::*;
use futures::FutureExt;
use gtk::prelude::*;
use relm4::*;
use report::create_report;
use reqwest::blocking::multipart;
use serde::Serialize;
use std::error::Error;

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    #[serde(rename = "Unit")]
    pub unit: String,
    #[serde(rename = "Executable")]
    pub exe: String,
    #[serde(rename = "Message")]
    pub message: String,
}

#[derive(Default)]
pub struct App {
    computing: bool,
    task: Option<CmdOut>,
}

pub struct Widgets {
    button: gtk::Button,
    spinner: adw::Spinner,
    label: gtk::Label,
    textview: gtk::TextView,
}

#[derive(Debug)]
pub enum Input {
    Report,
}

#[derive(Debug)]
pub enum Output {
    Clicked(u32),
}

#[derive(Debug)]
pub enum CmdOut {
    Spinner,
    Finished,
}

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

                    append: spinner = &adw::Spinner {
                        set_visible: false,
                        set_width_request: 24,
                        set_height_request: 24,
                    },

                    append: label = &gtk::Label {
                        set_label: "",
                        set_margin_bottom: 4,
                        add_css_class: "caption",
                        add_css_class: "dim-label",
                    },

                    append: button = &gtk::Button {
                        set_label: "Send Report",
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_margin_bottom: 16,
                        add_css_class: "suggested-action",
                        add_css_class: "pill",
                        connect_clicked => Input::Report,
                    },
                },
            }
        }

        root.set_content(Some(&toolbar_view));

        ComponentParts {
            model: App::default(),
            widgets: Widgets {
                button,
                spinner,
                label,
                textview,
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Report => {
                self.computing = true;
                sender.command(|out, shutdown| {
                    out.send(CmdOut::Spinner).unwrap();
                    shutdown
                        .register(async move {
                            let path = create_report("tmp", Some("~/.config/nix"), None)
                                .map(|f| format!("{}.zip", f.file.display()))
                                .unwrap_or_else(|_| "error.json".to_string());

                            let result = tokio::task::spawn_blocking(move || report_srv(path))
                                .await
                                .unwrap();

                            if result.is_err() {
                                eprintln!("Failed to send report");
                            }

                            out.send(CmdOut::Finished).unwrap();
                        })
                        .drop_on_shutdown()
                        .boxed()
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let CmdOut::Finished = message {
            self.computing = false;
        }
        self.task = Some(message);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.button.set_sensitive(!self.computing);

        if let Some(ref task) = self.task {
            match task {
                CmdOut::Spinner => {
                    widgets.label.set_label("Ma'lumotlar yuborilmoqda");
                    widgets.textview.set_visible(false);
                    widgets.spinner.set_visible(true);
                    widgets.button.set_visible(false);
                }
                CmdOut::Finished => {
                    widgets.spinner.set_visible(false);
                    widgets
                        .label
                        .set_label("Ma'lumotlar muvaffaqiyatli yuborildi");
                }
            }
        }
    }
}

fn report_srv(file_path: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let form = multipart::Form::new().file("report", file_path)?;
    reqwest::blocking::Client::new()
        .post("http://localhost:5678/upload/report")
        .multipart(form)
        .send()?;
    Ok(())
}
