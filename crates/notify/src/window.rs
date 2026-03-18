use relm4::{
    adw::{self, prelude::*},
    component::*,
    gtk::{self, prelude::*},
    main_application, *,
};
use reqwest::blocking::multipart;

use super::Modal;
use serde_json::{json, Value};

struct AppModel {
    error: Modal,
    title: String,
    state: AppState,
    url: String,
}

#[derive(Debug)]
enum AppState {
    Init,
    Spinning,
    Send,
}

#[derive(Debug)]
enum AppMsg {
    Report,
}

#[relm4::component(async)]
impl AsyncComponent for AppModel {
    type Init = Modal;

    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();
    // type Root;

    view! {
        adw::Window {
            set_title: Some("Simple app"),
            set_default_size: (400, 200),


            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_vexpand: true,
                set_hexpand: true,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        #[watch]
                        set_title: model.title.as_str(),
                    }
                },

                match model.state {
                    AppState::Init => gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_vexpand: true,
                        set_hexpand: true,

                        gtk::TextView {
                            set_monospace: true,
                            set_editable: false,
                            set_cursor_visible: false,
                            set_hexpand: true,
                            set_vexpand: true,

                            #[wrap(Some)]
                            set_buffer = &gtk::TextBuffer {
                                set_text: &format! (
                                    "{}",
                                    serde_json::to_string_pretty(&error)
                                        .unwrap_or("Invalid data".to_string())),
                            }
                        },

                        gtk::Button {
                            set_label: "Report",
                            set_hexpand: true,
                            connect_clicked => AppMsg::Report
                        },
                    },
                    AppState::Spinning => gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_vexpand: true,
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        // set_size_request: (120, 120),

                        adw::Spinner {
                            set_height_request: 180,
                            set_width_request: 180,
                        },
                        gtk::Label  {
                            set_text: "Xatoliklar bilan bog'liq barcha ma'lumotlar yig'ilmoqda. Iltimos kuting",
                            set_valign: gtk::Align::Center,
                        },
                    },
                    AppState::Send => gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,
                        gtk::Label  {
                            set_text: "Logs sent successfully"
                        },
                    },
                },
            },
        }
    }

    async fn init(
        error: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = AppModel {
            error: error.clone(),
            title: format!("Error on {}", error.unit),
            state: AppState::Init,
            url: "http://localhost:5678".to_string(),
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppMsg::Report => {
                println!("Hello chigga");
                self.title = "Spinner".to_string();
                self.state = AppState::Spinning;

                match report() {
                    Ok(()) => {
                        self.state = AppState::Send;
                        // FIXME: we need to set tmp directory, config path in env
                        match report::create_report("tmp", "~/.config/nix", None) {
                            Ok(rep) => self.state = AppState::Send,
                            Err(_) => self.state = AppState::Init,
                        }
                    }
                    Err(_) => self.state = AppState::Init,
                }
            }
        }
    }
}

pub fn open(error: Modal) {
    let app = RelmApp::new("relm4.test.simple");
    app.run_async::<AppModel>(error);
}

fn report(file_path: String) -> anyhow::Result<()> {
    let url = "http://localhost:5678";
    let file_path = "error.json";

    let form = multipart::Form::new()
        .text("username", "seanmonstar")
        .file("file", file_path)?;

    let client = reqwest::blocking::Client::new();
    client.post(url).multipart(form).send()?;
    Ok(())
}
