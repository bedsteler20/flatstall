use crate::{ref_file, spawn_app::spawn_app};
use adw::prelude::*;
use flatpak::prelude::*;
use gtk::gdk_pixbuf;
use gtk::gtk4_macros::include_blueprint;
use std::rc::Rc;

#[derive(Clone)]
pub struct AppUi {
    application: adw::Application,
    window: adw::PreferencesWindow,
    install_page: adw::PreferencesPage,
    about_page: adw::PreferencesPage,
    icon: gtk::Image,
    name_label: gtk::Label,
    install_btn: gtk::Button,
    progress_bar: gtk::ProgressBar,
    summery_label: gtk::Label,
}

impl AppUi {
    pub fn new(application: adw::Application) -> Self {
        let builder = gtk::Builder::from_string(include_blueprint!("src/window.blp"));
        let window: adw::PreferencesWindow = builder.object("window").unwrap();
        let install_page: adw::PreferencesPage = builder.object("install_page").unwrap();
        let about_page: adw::PreferencesPage = builder.object("about_page").unwrap();
        let icon: gtk::Image = builder.object("icon").unwrap();
        let name_label: gtk::Label = builder.object("name_label").unwrap();
        let install_btn: gtk::Button = builder.object("install_btn").unwrap();
        let progress_bar: gtk::ProgressBar = builder.object("progress_bar").unwrap();
        let summery_label: gtk::Label = builder.object("summery_label").unwrap();

        window.set_application(Some(&application));
        window.add(&about_page);
        window.set_visible_page(&install_page);
        Self {
            application,
            window,
            install_page,
            about_page,
            icon,
            name_label,
            install_btn,
            progress_bar,
            summery_label,
        }
    }

    pub fn show_error(&self, message: &str) {
        self.window.close();
        let error_dialog = adw::MessageDialog::builder()
            .application(&self.application)
            .title("Error")
            .heading("Error")
            .body(format!("Error: {}", message).as_str())
            .build();
        error_dialog.add_response("ok", "Ok");

        error_dialog.connect_response(Some("ok"), move |dialog, _| {
            dialog.close();
            dialog.application().unwrap().quit();
        });
        error_dialog.present();
    }

    pub fn show_installed(&self, app_id: &str, title: &str) {
        let app_id = app_id.to_string();
        self.window.close();
        let installed_dialog = adw::MessageDialog::builder()
            .application(&self.application)
            .title("Installed")
            .heading("Installed")
            .body(format!("{} installed", title).as_str())
            .build();
        installed_dialog.add_response("close", "Close");
        installed_dialog.add_response("open", "Open App");

        installed_dialog.connect_response(Some("open"), move |dialog, _| {
            dialog.close();
            spawn_app(&app_id.to_string());
            if let Some(application) = dialog.application() {
                application.quit();
            }
        });

        installed_dialog.connect_response(Some("close"), |dialog, _| {
            dialog.close();
            dialog.application().unwrap().quit();
        });

        installed_dialog.present();
    }

    pub fn show_progress(&self, progress: f64) {
        self.progress_bar.set_fraction(progress);
    }

    #[allow(deprecated)]
    pub fn set_icon_ref(
        &self,
        app_ref: &ref_file::RefFileIni,
        installation: &flatpak::Installation,
    ) {
        if app_ref.from_flathub() {
            let icon = installation
                .path()
                .and_then(|path| path.path())
                .and_then(|path| {
                    let appstream_path = path
                        .join("appstream")
                        .join(app_ref.remote_name())
                        .join(flatpak::default_arch().unwrap())
                        .join("active")
                        .join("icons")
                        .join("128x128");
                    Some(appstream_path.join(format!("{}.png", app_ref.app_id())))
                });
            if let Some(icon) = icon {
                if icon.exists() {
                    let pb = gdk_pixbuf::Pixbuf::from_file(icon).unwrap();
                    self.icon.set_from_pixbuf(Some(&pb));
                }
            }
        }

        match app_ref.icon() {
            Some(url) => match reqwest::blocking::get(url) {
                Ok(response) => {
                    let bytes = response.bytes().unwrap();
                    let loader = gdk_pixbuf::PixbufLoader::new();
                    loader.write(&bytes).unwrap();
                    loader.close().unwrap();
                    let pb = loader.pixbuf().unwrap();
                    self.icon.set_from_pixbuf(Some(&pb));
                }
                Err(_) => (),
            },
            None => (),
        };
    }

    #[allow(deprecated)]
    pub fn set_icon_bundle(&self, app_bun: &flatpak::BundleRef) {
        if let Some(bytes) = app_bun.icon(128) {
            let loader = gdk_pixbuf::PixbufLoader::new();
            loader.write(&bytes).unwrap();
            loader.close().unwrap();
            let icon = loader.pixbuf();

            if let Some(icon) = icon {
                self.icon.set_from_pixbuf(Some(&icon));
            }
        };
    }

    pub fn set_name(&self, name: &str) {
        self.name_label.set_text(name);
    }

    pub fn set_summery(&self, summery: &str) {
        self.summery_label.set_text(summery);
    }

    pub fn connect_install(&self, f: Box<dyn Fn() + 'static>) {
        let f = Rc::new(f);
        let self_ = self.clone();
        self.install_btn.connect_clicked(move |_| {
            let f = Rc::clone(&f);
            self_.window.add(&self_.install_page);
            self_.window.set_visible_page(&self_.install_page);
            self_.window.remove(&self_.about_page);
            f();
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}
