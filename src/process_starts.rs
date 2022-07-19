use super::Author;

use std::process;
use std::sync::Mutex;
use std::env;
use std::path::Path;

use users::os::unix::UserExt;

const DISPLAY: &str = ":1";
const VT: &str = "vt01";

static X_SERVER: Mutex<Option<process::Child>> = Mutex::new(None);

pub fn start_x_server() {
    let mut x_server = X_SERVER.lock().unwrap();
    if x_server.is_some() {
        return;
    }
    std::env::set_var("DISPLAY", DISPLAY);
    let child = process::Command::new("/usr/bin/X")
        .arg(DISPLAY)
        .arg(VT)
        .spawn().expect("Could not start the X server");
    *x_server = Some(child);
}

pub fn stop_x_server() {
    let mut x_server_lock = X_SERVER.lock().unwrap();
    if let Some(mut s) = x_server_lock.take() {
        s.kill().expect("Could not kill the X server");
    }
}

pub fn start_session(mut author: Author, username: String) {
    let user = users::get_user_by_name(&username).expect("Could not find user");
    env::set_var("HOME", user.home_dir());
    env::set_var("PWD", user.home_dir());
    env::set_var("SHELL", user.shell());
    env::set_var("USER", user.name());
    env::set_var("LOGNAME", user.name());
    env::set_var("PATH", "/usr/local/sbin:/usr/local/bin:/usr/bin:/bin");
    env::set_var("MAIL", format!("/var/spool/mail/{}", user.name().to_string_lossy()));
    env::set_var("XAUTHORITY", user.home_dir().join(".Xauthority"));

    author.open_session().expect("Could not open session");
}
