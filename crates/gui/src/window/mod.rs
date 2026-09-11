pub mod report;

use crate::{get_log_tail, window::report::UploadError};

use fluent_zero::t;
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
    adw::{self, prelude::*},
    gtk::{self, glib},
    main_application,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use zbus::zvariant::Type;

#[derive(Clone, Debug, Serialize, Deserialize, Type, Default)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}
// impl Modal {
//     fn file_name(&self) -> &str {
//         self.exe.split("/").last().unwrap_or_default()
//     }
//     fn display_message(&self) -> String {
//         format!("{} {}", self.file_name(), self.message)
//     }
// }

#[derive(Debug)]
pub struct App {
    computing: bool,
    modal: Modal,
    report_page: ReportPage,
    status_message: String,
    show_details: bool,
    log: String,
}

// App state pages
#[derive(Debug)]
pub enum ReportPage {
    Welcome,
    Crash,
    Progress,
    Success,
    Fail,
    Register,
}

#[derive(Debug)]
pub enum AppInput {
    // Report with user provided context
    Report(Option<String>),
    Dismiss,
    Retry,
    Register,
    CrashCmd(CmdCrashOut),
    ShowDetails,
}

#[derive(Debug)]
pub enum CmdOut {
    CrashCmd(CmdCrashOut),
}

#[derive(Debug)]
pub enum CmdCrashOut {
    Progress { fraction: f64, message: String },
    Finished { bytes: u64 },
    Error(String),
    UploadFailed(UploadError),
    SetupKeyDone,
}

impl From<CmdCrashOut> for CmdOut {
    fn from(val: CmdCrashOut) -> Self {
        Self::CrashCmd(val)
    }
}

#[derive(Debug)]
pub enum Output {
    Clicked(u32),
}

#[relm4::component(pub)]
impl Component for App {
    type Init = Modal;
    type Input = AppInput;
    type Output = Output;
    type CommandOutput = CmdOut;

    view! {
        #[root]
        main_window = adw::ApplicationWindow {
            #[watch]
            set_default_height: -1,
            // #[watch]
            // set_size_request: (model.size.0, model.size.1),
            set_visible: true,
            set_title: Some(&t!("app-title")),
            set_deletable: false,
            set_modal: true,
            connect_close_request[sender] => move |_| {
                sender.input(AppInput::Dismiss);
                glib::Propagation::Stop
            },
            match model.report_page {
                ReportPage::Welcome => adw::StatusPage {
                    set_icon_name: Some("airplane-mode-symbolic"),
                    set_title: &t!("welcome-title"),
                    set_description: Some(&t!("welcome-description")),
                },
                ReportPage::Crash => gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 24,
                    set_width_request: 600,
                    set_margin_all: 16,
                    gtk::Image {
                        set_icon_name: Some("dialog-warning-symbolic"),
                        set_pixel_size: 80,
                    },
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        add_css_class: "heading",
                        set_text: &model.modal.message,
                    },
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_text: &t!("Click \"Send Report\" to submit the report. This information is collected anonymously."),
                    },
                    append: context_box = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 4,
                        gtk::Label {
                            set_label: &t!("additional-context"),
                            set_xalign: 0.0,
                            add_css_class: "caption",
                            add_css_class: "dim-label",
                        },
                        gtk::Frame {
                            set_hexpand: true,
                            gtk::ScrolledWindow {
                                set_hexpand: true,
                                set_height_request: 100,
                                add_css_class: "view",
                                #[wrap(Some)]
                                set_child: context_text_view = &gtk::TextView {
                                    set_wrap_mode: gtk::WrapMode::Word,
                                    set_margin_all: 8,
                                    set_accepts_tab: false,
                                },
                            },
                        },
                        gtk::Box {
                            #[watch]
                            set_visible: model.show_details,
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_top: 12,
                            set_spacing: 4,
                            gtk::Label {
                                set_label: &t!("report-body"),
                                set_xalign: 0.0,
                                add_css_class: "caption",
                                add_css_class: "dim-label",
                            },
                            gtk::Frame {
                                set_hexpand: true,
                                gtk::ScrolledWindow {
                                    set_hexpand: true,
                                    set_height_request: 100,
                                    add_css_class: "view",
                                    #[wrap(Some)]
                                    set_child: report_body = &gtk::TextView {
                                        set_wrap_mode: gtk::WrapMode::Word,
                                        set_margin_all: 8,
                                        set_can_focus: false,
                                        set_editable: false,
                                        set_cursor_visible: false,
                                        set_accepts_tab: false,
                                        #[iterate]
                                        add_css_class: ["monospace", "body"],

                                        #[wrap(Some)]
                                        set_buffer = &gtk::TextBuffer {
                                            #[watch]
                                            set_text: &model.log,
                                        },
                                    },
                                },
                            },
                            gtk::Label {
                                set_margin_top: 8,
                                set_label: &t!("report-metadata"),
                                set_xalign: 0.0,
                                add_css_class: "caption",
                                add_css_class: "dim-label",
                            },
                            adw::PreferencesGroup {
                                adw::ActionRow {
                                    set_title: &t!("action-row-unit"),
                                    add_css_class: "property",
                                    set_subtitle_selectable: true,
                                    set_subtitle: model.modal.unit.as_str(),
                                },
                                adw::ActionRow {
                                    set_title: &t!("action-row-exe"),
                                    add_css_class: "property",
                                    set_subtitle_selectable: true,
                                    set_subtitle: model.modal.exe.as_str(),
                                },
                            },
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_halign: gtk::Align::BaselineFill,
                            set_margin_top: 16,
                            set_spacing: 8,
                            set_hexpand: true,

                            append: show_details = &gtk::Button {
                                #[watch]
                                set_label: &if model.show_details { t!("hide-details") } else { t!("show-details") },
                                add_css_class: "pill",
                                connect_clicked => AppInput::ShowDetails,
                            },
                            gtk::Button {
                                set_valign: gtk::Align::Center,
                                add_css_class: "circular",
                                set_icon_name: "question-symbolic",
                            },

                            gtk::Box {
                                set_halign: gtk::Align::End,
                                set_hexpand: true,
                                set_spacing: 8,
                                append: button_close = &gtk::Button {
                                    set_label: &t!("dont-send"),
                                    add_css_class: "pill",
                                    connect_clicked => AppInput::Dismiss,
                                },
                                append: send_button = &gtk::Button {
                                    set_label: &t!("send-report"),
                                    add_css_class: "suggested-action",
                                    add_css_class: "pill",
                                    connect_clicked[sender, context_text_view] => move |_| {
                                        let buf = context_text_view.buffer();
                                        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                                        let ctx = if text.is_empty() { None } else { Some(text.to_string()) };
                                        sender.input(AppInput::Report(ctx));
                                    },
                                },
                            }
                        },
                    },

                },
                ReportPage::Progress => gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_margin_all: 12,
                    append: progress = &gtk::ProgressBar {
                        set_visible: true,
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_margin_top: 16,
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
                },
                ReportPage::Success => adw::StatusPage {
                    add_css_class: "success",
                    set_icon_name: Some("object-select-symbolic"),
                    set_title: &t!("success-title"),
                    #[watch]
                    set_description: Some(model.status_message.as_str()),
                    #[wrap(Some)]
                    set_child = &gtk::Button {
                        set_halign: gtk::Align::Center,
                        set_label: &t!("close"),
                        add_css_class: "pill",
                        connect_clicked => AppInput::Dismiss,
                    },
                },
                ReportPage::Fail => adw::StatusPage {
                    add_css_class: "error",
                    set_icon_name: Some("dialog-error-symbolic"),
                    set_title: &t!("fail-title"),
                    #[watch]
                    set_description: Some(model.status_message.as_str()),
                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_halign: gtk::Align::Center,
                        append = &gtk::Button {
                            set_label: &t!("close"),
                            add_css_class: "pill",
                            connect_clicked => AppInput::Dismiss,
                        },
                        append = &gtk::Button {
                            set_label: &t!("retry"),
                            add_css_class: "suggested-action",
                            add_css_class: "pill",
                            connect_clicked => AppInput::Retry,
                        },
                    },
                },
                ReportPage::Register => adw::StatusPage {
                    set_icon_name: Some("dialog-password-symbolic"),
                    set_title: &t!("register-title"),
                    set_description: Some(&t!("register-description")),
                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_halign: gtk::Align::Center,
                        append = &gtk::Button {
                            set_label: &t!("close"),
                            add_css_class: "pill",
                            connect_clicked => AppInput::Dismiss,
                        },
                        append = &gtk::Button {
                            set_label: &t!("register"),
                            add_css_class: "suggested-action",
                            add_css_class: "pill",
                            connect_clicked => AppInput::Register,
                        },
                    },
                },
            } -> {
                // You can set the `gtk::Stack`'s properties here.
                set_transition_type: gtk::StackTransitionType::Crossfade,
                set_halign: gtk::Align::Center,
                set_vhomogeneous: false,
            },
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            computing: false,
            modal: init,
            report_page: ReportPage::Crash,
            status_message: String::new(),
            show_details: false,
            log: get_log_tail(50).unwrap_or_default(),
        };
        let widgets = view_output!();

        // report::run_setup_key(sender.clone());
        ComponentParts { model, widgets }
    }
    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CmdOut::CrashCmd(cmd_crash_out) => {
                sender.input(AppInput::CrashCmd(cmd_crash_out));
            }
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppInput::Dismiss => main_application().quit(),
            AppInput::ShowDetails => {
                self.show_details = !self.show_details;
                if self.log.is_empty() {
                    self.log = get_log_tail(50).unwrap_or_default();
                }
                // let buffer = gtk::TextBuffer::default();
                // buffer.set_text(&self.log);
                // widgets.report_body.set_buffer(Some(&buffer));
            }
            AppInput::Retry => {
                self.computing = false;
                self.status_message.clear();
                self.report_page = ReportPage::Crash;
            }
            AppInput::Register => {
                self.computing = true;
                self.report_page = ReportPage::Progress;
                widgets.label.set_label(&t!("progress-register"));
                widgets.progress.set_fraction(0.0);
                report::run_setup_key(&sender);
            }
            AppInput::Report(context) => {
                println!("Clicked button report: {context:?}");
                self.report_page = ReportPage::Progress;

                self.computing = true;
                report::run(&sender, context);
            }
            AppInput::CrashCmd(cmd_crash_out) => match cmd_crash_out {
                CmdCrashOut::Progress { fraction, message } => {
                    self.report_page = ReportPage::Progress;
                    // self.size = (300, -1);
                    // widgets.main_window.set_default_size(300, 150);
                    // widgets.main_window.set_size_request(300, 150);

                    widgets.progress.set_fraction(fraction);
                    widgets.label.set_label(&message);
                    widgets
                        .label_pct
                        .set_label(&format!("{:.0}%", fraction * 100.0));
                    // widgets.button_close.set_label(&t!("cancel"));
                }
                CmdCrashOut::Finished { bytes } => {
                    self.report_page = ReportPage::Success;
                    self.computing = false;
                    self.status_message = String::from(
                        t!("progress-crash-finished", { "bytes" => format!("{:.1}", bytes / 1024) }),
                    );
                }
                CmdCrashOut::Error(e) => {
                    self.report_page = ReportPage::Fail;
                    self.computing = false;
                    self.status_message =
                        String::from(t!("progress-crash-error", { "error" => e }));
                }
                CmdCrashOut::UploadFailed(why) => {
                    self.computing = false;
                    match why {
                        UploadError::MissingId { .. } => {
                            // Needs attention. Do this work?
                            self.computing = false;
                            self.status_message.clear();
                            self.report_page = ReportPage::Register;
                        }
                        UploadError::Io(io) => {
                            self.status_message = String::from(
                                t!("progress-crash-upload-io-error", { "error" => format!("{io}") }),
                            );
                            self.report_page = ReportPage::Fail;
                        }
                        UploadError::Network(net) => {
                            self.status_message = String::from(
                                t!("progress-crash-upload-network-error", { "error" => format!("{net}") }),
                            );
                            self.report_page = ReportPage::Fail;
                        }
                        UploadError::Client { status, body } => {
                            self.status_message = String::from(
                                t!("progress-crash-upload-client-error", { "status" => format!("{status}"), "body" => format!("{body}") }),
                            );
                            self.report_page = ReportPage::Fail;
                        }
                        UploadError::Server { status, .. } => {
                            self.status_message = String::from(
                                t!("progress-crash-upload-server-error", { "status" => format!("{status}") }),
                            );
                            self.report_page = ReportPage::Fail;
                        }
                    }
                }
                CmdCrashOut::SetupKeyDone => {
                    self.computing = false;
                    self.status_message.clear();
                    self.report_page = ReportPage::Crash;
                }
            },
        }

        widgets.send_button.set_sensitive(!self.computing);
        widgets.button_close.set_sensitive(true);

        self.update_view(widgets, sender);
    }
}
