use relm4::{gtk::{self, prelude::*}, adw::{self, prelude::*}, component::{*}, *};
use std::future::Future;
use serde_json::Value;
use sserde_core::ser::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}

struct AppModel {
    error: Modal,
}

#[derive(Debug)]
enum AppMsg {
    Report
}

#[relm4::component(async)]
impl AsyncComponent for AppModel {
    type Init = Modal;

    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::Window {
            set_title: Some("Simple app"),
            set_default_size: (400, 200),


            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Sidebar",
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::TextView {
                        set_monospace: true,
                        set_editable: false,
                        set_cursor_visible: false,

                        #[wrap(Some)]
                        set_buffer = &gtk::TextBuffer {
                            set_text: &format! ("{}", self.error.clone()),
                        }
                    },

                    gtk::Button {
                        set_label: "Report",
                        set_hexpand: true,
                        connect_clicked => AppMsg::Report
                    }
                },

            },
        }
    }

    async fn init(
        error: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AppModel { error };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AppMsg::Report => {
                // println!("{}", self.error);
                report(self.error.clone()).await;
            }
        }
    }
}

async fn report(error: Modal) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:5678/")
        .header("Content-Type", "application/json; charset=utf-8")
        .json(&error)
        .send()
        .await?;
    let body = res.text().await?;
    println!("{:?}", body);
    Ok(())
}

pub fn open(error: Modal) {
    let app = RelmApp::new("relm4.test.simple");
    app.run_async::<AppModel>(error);
}
