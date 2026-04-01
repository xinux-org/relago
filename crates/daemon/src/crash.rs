
use crash_event::{CrashEvent};

use utils::journal_ext::JournalExt;
#[derive(Debug, CrashEvent)]
#[journal(filter(SYSLOG_IDENTIFIER = "systemd-coredump"))]
pub struct CoredumpCrash {
    #[journal(field = "COREDUMP_EXE", required)]
    pub exe: String,

    #[journal(field = "COREDUMP_PID", required)]
    pub pid: u32,

    #[journal(field = "COREDUMP_SIGNAL")]
    pub signal: Option<u32>,

    #[journal(field = "COREDUMP_CMDLINE")]
    pub cmdline: Option<String>,

    #[journal(field = "COREDUMP_UID")]
    pub uid: Option<u32>,

    #[journal(field = "COREDUMP_UNIT")]
    pub unit: Option<String>,

    #[journal(field = "COREDUMP_FILENAME")]
    pub core_file: Option<String>,
}


#[derive(Debug, CrashEvent)]
#[journal(filter(SYSLOG_IDENTIFIER = "systemd"))]
pub struct ServiceFailureCrash {
    #[journal(field = "UNIT", required)]
    pub unit: String,

    // "done" | "failed" | "timeout" | "canceled" | "dependency" | "skipped"
    // #[journal(field = "JOB_RESULT", required)]
    // pub job_result: String,

    #[journal(field = "EXIT_CODE")]
    pub exit_code: Option<u32>,

    #[journal(field = "EXIT_STATUS")]
    pub exit_status: Option<String>,

    #[journal(field = "_SYSTEMD_INVOCATION_ID")]
    pub invocation_id: Option<String>,
}


#[derive(Debug, CrashEvent)]
#[journal(filter(MESSAGE_ID = "fe6bda9e7f4a4f5593682fcbcf9ee3f9"))]
pub struct OomCrash {
    #[journal(field = "_PID", required)]
    pub pid: u32,

    #[journal(field = "_COMM")]
    pub comm: Option<String>,

    // systemd unit that was killed (e.g. "firefox.service", "user@1000.service")
    #[journal(field = "_SYSTEMD_UNIT")]
    pub unit: Option<String>,

    #[journal(field = "KILLING_PROC_NAME")]
    pub killing_proc_name: Option<String>,

    #[journal(field = "KILLING_PROC_UID")]
    pub killing_proc_uid: Option<u32>,
}


#[derive(Debug)]
pub enum Crash {
    Coredump(CoredumpCrash),
    ServiceFailure(ServiceFailureCrash),
    Oom(OomCrash),
}
