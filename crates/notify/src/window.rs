use relm4::{gtk::{self, prelude::*}, adw::{self, prelude::*}, component::{*}, main_application, *};
use serde_json::{Value, json};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}

struct AppModel {
    error: Modal,
    report_label: String,
    main_box: gtk::Box,
    spinner: bool,
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
                        set_title: &model.report_label,
                    }
                },

                #[name(main_box)]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,
                    set_hexpand: true,

                    gtk::Spinner {
                        #[watch]
                        set_spinning: model.spinner,
                    },

                    gtk::TextView {
                        set_monospace: true,
                        set_editable: false,
                        set_cursor_visible: false,

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
        let mut model = AppModel {
            error: error.clone(),
            report_label: "sidebar".to_string(),
            main_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            spinner: false,
        };

        let widgets = view_output!();
        let main_box = widgets.main_box.clone();
        model.main_box = main_box;

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AppMsg::Report => {
                println!("Hello chigga");
                // relm4::main_
                // main_application::quit();
                // self.window.close();
                self.report_label = "changed".to_string();
                self.spinner = !(self.spinner);
                // relm4::main_application().quit();
            }
        }
    }
}

pub fn open(error: Modal) {
    let app = RelmApp::new("relm4.test.simple");
    app.run_async::<AppModel>(error);
}
