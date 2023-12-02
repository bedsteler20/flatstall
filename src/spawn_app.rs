pub fn spawn_app(app: &str) {
    let in_flatpak = std::env::var("FLATPAK_ID").is_ok();
    if in_flatpak {
        let mut command = std::process::Command::new("flatpak-spawn");
        command.arg("--host");
        command.arg("flatpak");
        command.arg("run");
        command.arg(app);
        command.spawn().unwrap();
    } else {
        let mut command = std::process::Command::new("flatpak");
        command.arg("run");
        command.arg(app);
        command.spawn().unwrap();
    }
}
