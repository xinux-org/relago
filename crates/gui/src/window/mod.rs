mod locales;
pub mod report;

use crate::window::locales::{CACHE, LOCALES};
use crate::window::report::UploadError;

use fluent_zero::t;
use relm4::{
    adw::{self, prelude::*},
    gtk::{self, glib},
    main_application, Component, ComponentParts, ComponentSender,
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

#[derive(Debug)]
pub struct App {
    pub computing: bool,
    pub modal: Modal,
    pub report_stack_page: ReportStack,
    pub status_message: String,
}

// App state pages
#[derive(Debug)]
pub enum ReportStack {
    Crash,
    Progress,
    Success,
    Fail,
    Register,
    Welcome,
}

#[derive(Debug)]
pub enum AppInput {
    // Report with user provided context
    Report(Option<String>),
    Dismiss,
    Retry,
    Register,
    CrashCmd(CmdCrashOut),
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
        CmdOut::CrashCmd(val)
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
            set_visible: true,
            set_title: Some(&t!("app-title")),
            connect_close_request[sender] => move |_| {
                sender.input(AppInput::Dismiss);
                glib::Propagation::Stop
            },
            #[name(toolbar_view)]
            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},
                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    adw::PreferencesPage {
                        set_width_request: 700,
                        set_width_request: 500,
                        adw::PreferencesGroup {
                            #[name(report_stack)]
                            gtk::Stack {
                                set_transition_type: gtk::StackTransitionType::Crossfade,
                                set_hhomogeneous: false,
                                set_vhomogeneous: false,
                                // donʻt translate
                                add_named: (&welcome, Some("welcome")),
                                add_named: (&crash, Some("crash")),
                                add_named: (&progress_page, Some("progress")),
                                add_named: (&success, Some("success")),
                                add_named: (&fail, Some("fail")),
                                add_named: (&register, Some("register")),
                                #[watch]
                                set_visible_child_name: match model.report_stack_page {
                                // donʻt translate
                                ReportStack::Welcome => "welcome",
                                ReportStack::Crash => "crash",
                                ReportStack::Progress => "progress",
                                ReportStack::Success => "success",
                                ReportStack::Fail => "fail",
                                ReportStack::Register => "register",
                                },
                            },
                        },
                    },
                },
            }
        },
        welcome = &adw::StatusPage {
            set_icon_name: Some("airplane-mode-symbolic"),
            set_title: &t!("welcome-title"),
            set_description: Some(&t!("welcome-description")),
        },
        crash = &adw::PreferencesGroup {
            set_title: &model.modal.message,
            set_width_request: 600,
            #[name(unit)]
            adw::ActionRow {
                set_title: &t!("action-row-unit"),
                add_css_class: "property",
                set_subtitle_selectable: true,
                set_subtitle: model.modal.unit.as_str(),
            },
            #[name(exe)]
            adw::ActionRow {
                set_title: &t!("action-row-exe"),
                add_css_class: "property",
                set_subtitle_selectable: true,
                set_subtitle: model.modal.exe.as_str(),
            },
            #[name(message)]
            adw::ActionRow {
                set_title: &t!("action-row-message"),
                add_css_class: "property",
                set_subtitle_selectable: true,
                set_subtitle: model.modal.message.as_str(),
            },
            add: context_box = &gtk::Box {
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
                    set_height_request: 200,
                    add_css_class: "card",
                    #[wrap(Some)]
                    set_child: context_text_view = &gtk::TextView {
                        set_wrap_mode: gtk::WrapMode::Word,
                        set_top_margin: 8,
                        set_left_margin: 8,
                        set_right_margin: 8,
                        set_bottom_margin: 8,
                        set_accepts_tab: false,
                    },
                },
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::BaselineFill,
                append: button_close = &gtk::Button {
                    set_label: &t!("close"),
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
            },
            },
        },
        progress_page = &gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            // todo add gtk::Stack
            append: progress = &gtk::ProgressBar {
                set_visible: true,
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

        },
        success = &adw::StatusPage {
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
        fail = &adw::StatusPage {
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
        register = &adw::StatusPage {
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
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            computing: false,
            modal: init,
            report_stack_page: ReportStack::Crash,
            status_message: String::new(),
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
        // match &message {
        //     CmdOut::CrashCmd(cmd_crash_out) => match &cmd_crash_out {
        //         // TODO: refactor error page
        //         CmdCrashOut::UploadFailed(why) => {}
        //         CmdCrashOut::Error(why) => {
        //             sender.input(AppInput::CrashCmd(CmdCrashOut::Error(why.to_owned())));
        //         }
        //         CmdCrashOut::SetupKeyDone => self.computing = false,
        //         CmdCrashOut::Progress { fraction, message } => {
        //             sender.input(AppInput::CrashCmd(CmdCrashOut::Progress {
        //                 fraction: fraction.to_owned(),
        //                 message: message.to_owned(),
        //             }));
        //         }
        //         CmdCrashOut::Finished { bytes: size } => {
        //             sender.input(AppInput::CrashCmd(CmdCrashOut::Finished {
        //                 bytes: size.to_owned(),
        //             }));
        //         }
        //     },
        // }

        match message {
            CmdOut::CrashCmd(cmd_crash_out) => {
                sender.input(AppInput::CrashCmd(cmd_crash_out));
            }
        }
    }

    // TODO: check ʼupdate_cmd_with_viewʼ instead of using two fn update_cmd and update_with_view

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // widgets.send_button.set_sensitive(!self.computing);
        // widgets.button_close.set_sensitive(true);
        match message {
            AppInput::Dismiss => main_application().quit(),
            AppInput::Retry => {
                self.computing = false;
                self.status_message.clear();
                self.report_stack_page = ReportStack::Crash;
            }
            AppInput::Register => {
                self.computing = true;
                self.report_stack_page = ReportStack::Progress;
                widgets.label.set_label(&t!("progress-register"));
                widgets.progress.set_fraction(0.0);
                report::run_setup_key(&sender);
            }
            AppInput::Report(context) => {
                println!("Clicked button report: {:?}", context);
                self.report_stack_page = ReportStack::Progress;

                self.computing = true;
                report::run(&sender, context);
            }
            AppInput::CrashCmd(cmd_crash_out) => match cmd_crash_out {
                CmdCrashOut::Progress { fraction, message } => {
                    // widgets.send_button.set_visible(false);
                    self.report_stack_page = ReportStack::Progress;

                    widgets.progress.set_fraction(fraction);
                    widgets.label.set_label(&message);
                    widgets
                        .label_pct
                        .set_label(&format!("{:.0}%", fraction * 100.0));
                    widgets.button_close.set_label(&t!("cancel"));
                }
                CmdCrashOut::Finished { bytes } => {
                    self.report_stack_page = ReportStack::Success;

                    self.computing = false;
                    // widgets.progress.set_fraction(1.0);
                    // widgets.label.set_label("Sent successfully");
                    self.status_message = String::from(
                        t!("progress-crash-finished", { "bytes" => format!("{:.1}", bytes / 1024) }),
                    );
                    // widgets.send_button.set_visible(false);
                    // widgets.button_close.set_label("Close");
                }
                CmdCrashOut::Error(e) => {
                    self.report_stack_page = ReportStack::Fail;
                    self.computing = false;
                    self.status_message =
                        String::from(t!("progress-crash-error", { "error" => e }));

                    // widgets.send_button.set_visible(true);
                    // widgets.send_button.set_label("Retry");
                    // widgets.button_close.set_label("Close");
                }
                CmdCrashOut::UploadFailed(why) => {
                    self.computing = false;
                    // let (text, button_label, context_visible) =
                    match why {
                        UploadError::MissingId { .. } => {
                            // Needs attention. Do this work?
                            self.computing = false;
                            self.status_message.clear();
                            self.report_stack_page = ReportStack::Register;
                            // (
                            //     "Reporter not registered.".to_string(),
                            //     Some("Register reporter"),
                            //     false,
                            // )
                        }
                        UploadError::Io(io) => {
                            // (format!("Local I/O error: {io}"), Some("Retry"), true)
                            self.status_message = String::from(
                                t!("progress-crash-upload-io-error", { "error" => format!("{io}") }),
                            );
                            self.report_stack_page = ReportStack::Fail;
                        }
                        UploadError::Network(net) => {
                            // (format!("Can't reach server: {net}"), Some("Retry"), true)
                            self.status_message = String::from(
                                t!("progress-crash-upload-network-error", { "error" => format!("{net}") }),
                            );
                            self.report_stack_page = ReportStack::Fail;
                        }
                        UploadError::Client { status, body } => {
                            // format!("Server rejected the report (HTTP {status}): {body}"),
                            // None,
                            // false,
                            self.status_message = String::from(
                                t!("progress-crash-upload-client-error", { "status" => format!("{status}"), "body" => format!("{body}") }),
                            );
                            self.report_stack_page = ReportStack::Fail;
                        }
                        UploadError::Server { status, .. } => {
                            // format!("Server error (HTTP {status}). Try again later."),
                            // Some("Retry"),
                            // true,
                            self.status_message = String::from(
                                t!("progress-crash-upload-server-error", { "status" => format!("{status}") }),
                            );
                            self.report_stack_page = ReportStack::Fail;
                        }
                    }

                    // widgets.context_box.set_visible(context_visible);
                    // widgets.progress.set_visible(false);
                    // widgets.label.set_label(&format!("Error: {text}"));
                    // if let Some(label) = button_label {
                    //     widgets.send_button.set_visible(true);
                    //     widgets.send_button.set_label(label);
                    // } else {
                    //     widgets.send_button.set_visible(false);
                    // }
                    // widgets.button_close.set_label("Close");
                }
                CmdCrashOut::SetupKeyDone => {
                    self.computing = false;
                    self.status_message.clear();
                    self.report_stack_page = ReportStack::Crash;

                    // widgets.context_box.set_visible(true);
                    // widgets.progress.set_visible(false);
                    // widgets
                    //     .label
                    //     .set_label("Reporter registered. Click Send Report to retry.");
                    // widgets.send_button.set_visible(true);
                    // widgets.send_button.set_label("Send Report");
                    // widgets.button_close.set_label("Close");
                }
            },
        }

        widgets.send_button.set_sensitive(!self.computing);
        widgets.button_close.set_sensitive(true);

        self.update_view(widgets, sender);
    }
}
