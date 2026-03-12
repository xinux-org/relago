use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt, TextViewExt};
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

struct AppModel {
    error: String,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = String;

    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_size: (400, 200),

            gtk::TextView {
                set_monospace: true,
                set_editable: false,
                set_cursor_visible: false,
                set_margin_all: 20,

                set_buffer: Some(&gtk::TextBuffer::builder()
                    .text(&format! ("{}", model.error))
                    .build()),
            }
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
}

pub fn open(error: String) {
    let app = RelmApp::new("relm4.test.simple");
    app.run::<AppModel>(error);
}
