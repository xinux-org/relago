use crate::window::Modal;

#[derive(Debug)]
pub enum Input {
    // Report with user provided context
    Report(Option<String>),
    Dismiss,
}

#[derive(Debug)]
pub enum Output {
    Clicked(u32),
}

#[derive(Debug)]
pub enum CmdOut {
    CrashDetected(Modal),
    CrashCmd(CmdCrashOut),
}

#[derive(Debug)]
pub enum CmdCrashOut {
    Progress { fraction: f64, message: String },
    Finished { bytes: u64 },
    Error(String),
}

impl From<CmdCrashOut> for CmdOut {
    fn from(val: CmdCrashOut) -> Self {
        CmdOut::CrashCmd(val)
    }
}
