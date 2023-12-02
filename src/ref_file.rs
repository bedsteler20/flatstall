use gtk::glib;

pub struct RefFileIni {
    from_flathub: bool,
    app_id: String,
    title: String,
    description: Option<String>,
    icon: Option<String>,
    remote_name: String,
}

#[derive(Debug)]
pub enum Error {
    UnableToReadFile,
    InvalidFile,
    UnableToTalkToFlathub(i32),
}

impl RefFileIni {
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn app_id(&self) -> &str {
        &self.app_id
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
    pub fn remote_name(&self) -> &str {
        &self.remote_name
    }
    pub fn from_flathub(&self) -> bool {
        self.from_flathub
    }

    pub fn new(file: String) -> Result<Self, Error> {
        // Load the flatpak ref file
        let key_file = glib::KeyFile::new();
        if key_file
            .load_from_file(file, glib::KeyFileFlags::NONE)
            .is_err()
        {
            return Err(Error::UnableToReadFile);
        }

        // Validate the flatpak ref file has the required sections
        if !key_file.has_group("Flatpak Ref") {
            return Err(Error::InvalidFile);
        }
        if !key_file.has_key("Flatpak Ref", "Title").unwrap_or_default() {
            return Err(Error::InvalidFile);
        }
        if !key_file.has_key("Flatpak Ref", "Name").unwrap_or_default() {
            return Err(Error::InvalidFile);
        }
        if !key_file
            .has_key("Flatpak Ref", "RuntimeRepo")
            .unwrap_or_default()
        {
            return Err(Error::InvalidFile);
        }

        // Get the required values from the flatpak ref file
        let title = key_file
            .string("Flatpak Ref", "Title")
            .unwrap_or_default()
            .to_string();
        let app_id = key_file
            .string("Flatpak Ref", "Name")
            .unwrap_or_default()
            .to_string();
        let remote_url = key_file
            .string("Flatpak Ref", "RuntimeRepo")
            .unwrap_or_default()
            .to_string();
        let remote_name = key_file
            .string("Flatpak Ref", "SuggestRemoteName")
            .map(|s| s.to_string())
            .unwrap_or(
                remote_url
                    .split('/')
                    .last()
                    .unwrap_or_default()
                    .replace(".flatpakrepo", ""),
            );
        // Validate the required values are not empty
        if title.is_empty() || app_id.is_empty() || remote_url.is_empty() {
            return Err(Error::InvalidFile);
        }

        // Check if the flatpak ref file is from flathub
        let from_flathub = remote_name == "flathub" || remote_name == "flathub-beta";

        // Get the optional values from the flatpak ref file
        let description = match key_file.string("Flatpak Ref", "Description") {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        };

        let icon = match key_file.string("Flatpak Ref", "Icon") {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        };

        // Get extra metadata from flathub
        if from_flathub {
            let metadata_url = format!("https://flathub.org/api/v2/appstream/{}", app_id);
            match reqwest::blocking::get(&metadata_url) {
                Ok(response) => {
                    let metadata: serde_json::Value = response.json().unwrap();
                    let description = metadata["summary"].as_str().map(|s| s.to_string());
                    let icon = metadata["icon"].as_str().map(|s| s.to_string());
                    let title = metadata["name"]
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or(title);
                    return Ok(Self {
                        title,
                        app_id,
                        remote_name,
                        from_flathub,
                        description,
                        icon,
                    });
                }
                Err(response) => {
                    return Err(Error::UnableToTalkToFlathub(
                        response.status().unwrap_or_default().as_u16() as i32,
                    ));
                }
            }
        }

        return Ok(Self {
            title,
            app_id,
            remote_name,
            from_flathub,
            description,
            icon,
        });
    }
}
