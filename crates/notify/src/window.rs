use std::{error::Error, path::PathBuf, sync::Arc};

use futures::FutureExt;
use gtk::prelude::*;
use relm4::{gtk::gio, *};
use report::create_report;
use reqwest::blocking::multipart;
use serde::Serialize;

pub fn open(rr: Modal) {
    let app = RelmApp::new("org.relm4.Modal");
    app.run::<App>(rr.into());
}

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}

#[derive(Default)]
pub struct App {
    /// Tracks progress status
    computing: bool,

    /// Contains output of a completed task.
    task: Option<CmdOut>,
}

pub struct Widgets {
    button: gtk::Button,
    gtkbox: gtk::Box,
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
    /// Progress update from a command.
    Spinner,
    /// The final output of the command.
    Finished,
}

impl Component for App {
    type Init = Modal;
    type Input = Input;
    type Output = Output;
    type CommandOutput = CmdOut;
    type Widgets = Widgets;
    type Root = gtk::Window;

    fn init_root() -> Self::Root {
        gtk::Window::default()
    }

    fn init(
        error: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        relm4::view! {
            container = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_vexpand: true,
                set_hexpand: true,

                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,

                    append: spinner = &adw::Spinner {
                        set_visible: false,
                        set_height_request: 180,
                        set_width_request: 180,
                    },

                    append: label = &gtk::Label {
                        // set_xalign: 0.0,
                        set_label: "Error"
                    },

                    append: gtkbox = &gtk::Box{
                        set_orientation: gtk::Orientation::Vertical,
                        append: textview = &gtk::TextView {
                            set_monospace: true,
                            set_editable: false,
                            set_cursor_visible: false,
                            set_hexpand: true,
                            set_vexpand: true,
                            set_visible: true,

                            #[wrap(Some)]
                            set_buffer = &gtk::TextBuffer {
                                set_text: &format! (
                                    "{}",
                                    serde_json::to_string_pretty(&error)
                                        .unwrap_or("Invalid data".to_string())),
                            }
                        },
                    },
                },

                append: button = &gtk::Button {
                    set_label: "Report",
                    set_hexpand: true,
                    set_vexpand: true,
                    set_visible: true,
                    connect_clicked => Input::Report
                },
            }
        }

        root.set_child(Some(&container));

        ComponentParts {
            model: App::default(),
            widgets: Widgets {
                gtkbox,
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
                println!("report");
                self.computing = true;
                sender.command(|out, shutdown| {
                    out.send(CmdOut::Spinner).unwrap();
                    shutdown
                        .register(async move {
                            let rep_file = create_report("tmp", Some("~/.config/nix"), None);

                            let path = match rep_file {
                                Ok(f) => format!("{}.zip", f.file.display().to_string()),
                                Err(_) => "error.json".to_string(),
                            };

                            println!("{}", path);
                            let res = tokio::task::spawn_blocking(move || report_srv(path))
                                .await
                                .unwrap();

                            match res {
                                Ok(_) => {
                                    out.send(CmdOut::Finished).unwrap();
                                }
                                Err(_) => {
                                    out.send(CmdOut::Finished).unwrap();
                                }
                            }
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

        if let Some(ref spinner) = self.task {
            match spinner {
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
                        .set_label("Ma'lumotlar muvaffaqiyatli yuborildi")
                }
            }
        }
    }
}

fn report_srv(file_path: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = "http://localhost:5678/upload/report";

    let form = multipart::Form::new()
        // .text("username", "seanmonstar")
        .file("report", file_path)?;

    let client = reqwest::blocking::Client::new();
    client.post(url).multipart(form).send()?;
    Ok(())
}
