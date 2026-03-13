use relm4::{gtk::{self, prelude::*}, adw::{self, prelude::*}, *};

struct AppModel {
    error: String,
}

#[derive(Debug)]
enum AppMsg {
    Report
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = String;

    type Input = AppMsg;
    // type Input = ();
    type Output = ();

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
    fn init(
        error: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel { error };

        // Insert the macro code generation here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Report => {
                println!("{}", self.error);
            }
        }
    }
}

pub fn open(error: String) {
    let app = RelmApp::new("relm4.test.simple");
    app.run::<AppModel>(error);
}
