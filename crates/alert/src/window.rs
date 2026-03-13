use relm4::{gtk::{self, prelude::*}, adw::{self, prelude::*}, component::{*}, *};
use std::future::Future;
use serde_json::Value;

struct AppModel {
    error: String,
}

#[derive(Debug)]
enum AppMsg {
    Report
}

#[relm4::component(async)]
impl AsyncComponent for AppModel {
    type Init = String;

    type Input = AppMsg;
    // type Input = ();
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
                            set_text: &format! ("{}", model.error),
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

    // Initialize the UI.
    async fn init(
        error: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AppModel { error };

        // Insert the macro code generation here
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AppMsg::Report => {
                println!("{}", self.error);
                report(self.error.clone()).await;
            }
        }
    }
}

async fn report(error: String) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:5678/")
        .header("content-type", "application/json")
        .body(error)
        .send()
        .await?;
    let body = res.text().await?;
    println!("{:?}", body);
    Ok(())
}

pub fn open(error: String) {
    let app = RelmApp::new("relm4.test.simple");
    app.run_async::<AppModel>(error);
}
