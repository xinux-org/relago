use reqwest::blocking::{Client, multipart};
use utils::config::CONFIG;

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("Failed to read reporter ID at {path}: {source}")]
    MissingId {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Client error (HTTP {status}): {body}")]
    Client { status: u16, body: String },

    #[error("Server error (HTTP {status}): {body}")]
    Server { status: u16, body: String },
}

pub fn upload(file_path: String, context: Option<String>) -> Result<(), UploadError> {
    let server = CONFIG.get().server.clone();
    let uuid_path = CONFIG.get().data_dir.join("uuid");
    let uuid = std::fs::read_to_string(&uuid_path)?.trim().to_owned();

    let mut form = multipart::Form::new().file("report", file_path)?;

    if let Some(context) = context {
        form = form.text("context", context);
    }

    let res = Client::new()
        .post(format!("{}/reports/runtime", &server))
        .header("Reporter-ID", &uuid)
        .multipart(form);

    let res = res.send()?;
    let status = res.status();
    let body = res.text()?;

    if status.is_success() {
        Ok(())
    } else if status.is_client_error() {
        Err(UploadError::Client {
            status: status.as_u16(),
            body,
        })
    } else {
        Err(UploadError::Server {
            status: status.as_u16(),
            body,
        })
    }
}
