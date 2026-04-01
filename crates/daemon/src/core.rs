use dbus_crossroads as crossroads;
use utils::{config::CONFIG, notify as Notify};

pub fn run() -> anyhow::Result<()> {
    let mut cr: crossroads::Crossroads = crossroads::Crossroads::new();

    let token = Notify::register_org_freedesktop_xinux_relago(&mut cr);
    cr.insert("/", &[token], ());

    let conn = dbus::blocking::Connection::new_session()?;
    conn.request_name(CONFIG.get().problems_interface.clone(), true, true, true)?;

    cr.serve(&conn)?;
    Ok(())
}
