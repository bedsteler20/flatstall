use std::fs;

use adw::prelude::*;
use flatpak::prelude::*;
use gtk::gio;
use gtk::gio::Cancellable;
use gtk::gio::File;


mod ref_file;
mod spawn_app;
mod transaction;
mod ui;

enum FileType {
    Bundle, // .flatpak
    // Repo,   // .flatpakrepo
    Ref, // .flatpakref
}

fn main() {
    let app = adw::Application::new(
        Some("com.github.gtk-rs.examples.basic"),
        gio::ApplicationFlags::HANDLES_OPEN,
    );
    app.connect_activate(active);
    app.connect_open(open);

    app.run();
}
#[allow(deprecated)]
#[allow(unused_assignments)]
fn open(app: &adw::Application, files: &[File], _hint: &str) {
    let file = files[0].clone().path().unwrap();
    let file_type = match file.extension().unwrap().to_str().unwrap() {
        "flatpak" => FileType::Bundle,
        // "flatpakrepo" => FileType::Repo,
        "flatpakref" => FileType::Ref,
        _ => {
            app.quit();
            panic!("Unsupported file type");
        }
    };

    let mut app_id = String::new();
    let mut app_title = String::new();

    let installation = flatpak::Installation::new_user(Cancellable::NONE).unwrap();

    let ui = ui::AppUi::new(app.clone());

    let ui1 = ui.clone();
    let transaction = transaction::Transaction::new(&installation, move |msg| match msg {
        transaction::Message::Err(err) => {
            ui1.show_error(&err);
        }
        transaction::Message::Progress(progress) => {
            ui1.show_progress(progress);
        }
        transaction::Message::Done => {
            ui1.show_installed(&app_id, &app_title);
        }
    });

    match file_type {
        FileType::Bundle => {
            let bundle = flatpak::BundleRef::new(&files[0]).unwrap();
            // TODO: add permissions with bundle.metadata
            app_id = bundle.name().unwrap().to_string();
            app_title = bundle.name().unwrap().to_string();
            ui.set_icon_bundle(&bundle);
            ui.set_name(&bundle.name().unwrap());
            transaction.add_bundle_file(file.to_str().unwrap().to_string());
        }
        FileType::Ref => {
            let ref_ = ref_file::RefFileIni::new(file.to_str().unwrap().to_string()).unwrap();
            app_id = ref_.app_id().to_string();
            app_title = ref_.title().to_string();
            ui.set_icon_ref(&ref_, &installation);
            ui.set_summery(&ref_.description().unwrap_or(""));
            ui.set_name(&ref_.title());
            transaction.add_ref_file(file.to_str().unwrap().to_string());
        }
    }

    ui.present();

    ui.connect_install(Box::new(move || {
        transaction.run();
    }));
}

pub fn active<'a>(_app: &'a adw::Application) {}
