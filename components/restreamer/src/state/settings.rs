//! Server's settings.
use crate::{spec, types::UNumber};
use serde::{Deserialize, Serialize};

/// Server's settings.
///
/// It keeps different settings not related to restreams but to whole server
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Settings {
    /// [`argon2`] hash of password which protects access to this application's
    /// public APIs.
    pub password_hash: Option<String>,

    /// [`argon2`] hash of password which protects access to single output
    /// application's public APIs.
    pub password_output_hash: Option<String>,

    /// Title for the server
    /// It is used for differentiating servers on UI side if multiple servers
    /// are used.
    pub title: Option<String>,

    /// Whether do we need to confirm deletion of inputs and outputs
    /// If `true` we should confirm deletion, `false` - do not confirm
    pub delete_confirmation: Option<bool>,

    /// Whether do we need to confirm enabling/disabling of inputs or outputs
    /// If `true` we should confirm, `false` - do not confirm
    pub enable_confirmation: Option<bool>,

    /// Google API key for file playback and downloading
    pub google_api_key: Option<String>,

    /// Max number of files allowed in [`Restream`]'s playlist
    ///
    /// [`Restream`]: crate::state::Restream
    pub max_downloading_files: Option<UNumber>,
}

impl Settings {
    /// Exports this [`Settings`] as a [`spec::v1::Settings`].
    #[inline]
    #[must_use]
    pub fn export(&self) -> spec::v1::Settings {
        spec::v1::Settings {
            delete_confirmation: self.delete_confirmation,
            enable_confirmation: self.enable_confirmation,
            title: self.title.clone(),
            google_api_key: self.google_api_key.clone(),
            max_downloading_files: self.max_downloading_files,
        }
    }

    // Applies the given [`spec::v1::Settings`] to this [`Settings`].
    ///
    pub fn apply(&mut self, new: spec::v1::Settings) {
        self.title = new.title;
        self.delete_confirmation = new.delete_confirmation;
        self.enable_confirmation = new.enable_confirmation;
        self.google_api_key = new.google_api_key;
        self.max_downloading_files = new.max_downloading_files;
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            password_hash: None,
            password_output_hash: None,
            title: None,
            delete_confirmation: Some(true),
            enable_confirmation: Some(true),
            google_api_key: None,
            max_downloading_files: None,
        }
    }
}
