use systemd::journal::Journal;

/// Minimal interface the generated detect() code calls.
pub trait JournalExt {
    fn field(&mut self, name: &str) -> Option<String>;
}

impl JournalExt for Journal {
    fn field(&mut self, name: &str) -> Option<String> {
        let data = self.get_data(name).ok()??;
        data.value()
            .map(|b| String::from_utf8_lossy(b).into_owned())
    }
}