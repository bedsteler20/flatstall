use {
    flatpak::prelude::*,
    gtk::{
        gio::{self, Cancellable},
        glib::{self, ControlFlow},
    },
    std::{
        fs,
        sync::mpsc::{self, Sender},
        thread,
    },
};

enum RequestMessage {
    AddRefFile(String),
    AddBundleFile(String),
    RunTransaction,
}
pub enum Message {
    Err(String),
    Progress(f64),
    Done,
}

pub struct Transaction {
    in_sender: Sender<RequestMessage>,
}

impl Transaction {
    pub fn new<F>(installation: &flatpak::Installation, mut handle: F) -> Self
    where
        F: FnMut(Message) + 'static,
    {
        let (in_sender, in_receiver) = mpsc::channel::<RequestMessage>();
        let (out_sender, out_receiver) = mpsc::channel::<Message>();
        let (err_sender, err_receiver) = mpsc::channel::<String>();
        let install_path = installation.path().unwrap().path().unwrap();
        let install_is_user = installation.is_user();

        glib::idle_add_local(move || {
            match out_receiver.try_recv() {
                Ok(message) => {
                    handle(message);
                }
                _ => (),
            }
            match err_receiver.try_recv() {
                Ok(message) => {
                    handle(Message::Err(message));
                }
                _ => (),
            }

            ControlFlow::Continue
        });

        thread::spawn(move || {
            let cancel = Cancellable::new();
            // GObjects cant be sent between threads so copy the data and
            // recreate the object
            let installation = match flatpak::Installation::for_path(
                &gio::File::for_path(install_path),
                install_is_user,
                Some(&cancel),
            ) {
                Ok(install) => install,
                Err(e) => {
                    let _ = out_sender.send(Message::Err(e.to_string()));
                    return;
                }
            };
            let transaction =
                match flatpak::Transaction::for_installation(&installation, Cancellable::NONE) {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = out_sender.send(Message::Err(e.to_string()));
                        return;
                    }
                };

            let out_sender_clone = out_sender.clone();
            transaction.connect_new_operation(move |transaction, operation, progress| {
                let out_sender_clone = out_sender_clone.clone();
                let transaction = transaction.clone();
                let operation = operation.clone();
                progress.connect_changed(move |progress| {
                    let _ = &out_sender_clone.send(Message::Progress(get_progress(
                        &transaction,
                        &operation,
                        &progress,
                    )));
                });
            });

            // Handle inputs incoming from the main thread
            loop {
                match in_receiver.try_recv() {
                    Ok(RequestMessage::RunTransaction) => match transaction.run(Some(&cancel)) {
                        Ok(_) => {
                            let _ = out_sender.send(Message::Done);
                        }
                        Err(e) => {
                            let _ = err_sender.send(e.to_string());
                            // let _ = out_sender.send(Message::Err(e.to_string()));
                        }
                    },
                    Ok(RequestMessage::AddBundleFile(file)) => {
                        let file = gio::File::for_path(&file);
                        match transaction.add_install_bundle(&file, None) {
                            Ok(_) => (),
                            Err(e) => {
                                let _ = err_sender.send(e.to_string());
                            }
                        }
                    }
                    Ok(RequestMessage::AddRefFile(file)) => {
                        let bytes: glib::Bytes = (&fs::read(&file).unwrap()).into();
                        match transaction.add_install_flatpakref(&bytes) {
                            Ok(_) => (),
                            Err(e) => {
                                let _ = err_sender.send(e.to_string());
                            }
                        }
                    }
                    _ => (),
                }
            }
        });

        Transaction { in_sender }
    }

    pub fn add_ref_file(&self, file: String) {
        self.in_sender
            .send(RequestMessage::AddRefFile(file))
            .unwrap();
    }

    pub fn run(&self) {
        self.in_sender.send(RequestMessage::RunTransaction).unwrap();
    }

    pub fn add_bundle_file(&self, file: String) {
        self.in_sender
            .send(RequestMessage::AddBundleFile(file))
            .unwrap();
    }
}

fn get_progress(
    transaction: &flatpak::Transaction,
    operation: &flatpak::TransactionOperation,
    progress: &flatpak::TransactionProgress,
) -> f64 {
    let total = transaction
        .operations()
        .iter()
        .fold(0, |acc, x| acc + x.installed_size() + x.download_size());
    let install_size = operation.installed_size() + operation.download_size();
    let bytes_transferred =
        install_size as f64 - (install_size as f64 / progress.progress() as f64);
    let prev_ops_size = transaction
        .operations()
        .iter()
        .take_while(|x| x.to_owned() != operation)
        .fold(0, |acc, x| acc + x.installed_size() + x.download_size());

    let weight = (prev_ops_size as f64 + bytes_transferred) / total as f64;
    return weight;
}
