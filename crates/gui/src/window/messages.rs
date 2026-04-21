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
    Progress { fraction: f64, message: String },
    Finished { bytes: u64 },
    Error(String),
}
