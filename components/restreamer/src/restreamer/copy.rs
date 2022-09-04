//! Kind of a [GStD] re-streaming process that re-streams a live stream from
//! one URL endpoint to another one "as is", without performing any live stream
//! modifications, optionally transmuxing it to the destination format.
//!
//! [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon

use std::path::Path;

use ephyr_log::log;
use gst_client;
use url::Url;
use uuid::Uuid;

use crate::dvr;

/// Kind of a [GStD] re-streaming process that re-streams a live stream from
/// one URL endpoint to another one "as is", without performing any live stream
/// modifications, optionally transmuxing it to the destination format.
///
/// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Clone, Debug)]
pub struct CopyRestreamer {
    /// ID of an element in a [`State`] this [`CopyRestreamer`]
    /// process is related to.
    ///
    /// [`State`]: crate::state::State
    pub id: Uuid,

    /// [`Url`] to pull a live stream from.
    pub from_url: Url,

    /// [`Url`] to publish the pulled live stream onto.
    pub to_url: Url,
}

impl CopyRestreamer {
    /// Checks whether this [`CopyRestreamer`] process must be restarted, as
    /// cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    #[inline]
    #[must_use]
    pub fn needs_restart(&self, actual: &Self) -> bool {
        self.from_url != actual.from_url || self.to_url != actual.to_url
    }

    /// Properly setups the given [GStD] [`Command`] for this
    /// [`CopyRestreamer`] before running it.
    ///
    /// # Errors
    ///
    /// If the given [GStD] [`Command`] fails to be setup.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub(crate) async fn setup_pipeline(
        &self,
        client: gst_client::GstClient,
    ) -> String {
        let mut elems = Vec::new();

        let _ = match self.from_url.scheme() {
            "http" | "https"
                if Path::new(self.from_url.path()).extension()
                    == Some("m3u8".as_ref()) =>
            {
                elems.push(format!(
                    "souphttpsrc location={} ! hlsdemux",
                    self.from_url.as_str()
                ))
            }

            "rtmp" | "rtmps" => elems
                .push(format!("rtmp2src location={}", self.from_url.as_str())),

            _ => unimplemented!(),
        };

        let _ = match self.to_url.scheme() {
            "file"
                if Path::new(self.to_url.path()).extension()
                    == Some("flv".as_ref()) =>
            {
                elems.push(format!(
                    "filesink location={}",
                    dvr::new_file_path(&self.to_url).await.unwrap().display()
                ))
            }

            "icecast" => {
                let path = self.to_url.path();
                let username = self.to_url.username();
                let password = self.to_url.password().unwrap();
                let address = self.to_url.host_str().unwrap();
                let port = self.to_url.port().unwrap();
                elems.push(format!(
                    "audioconvert ! lamemp2enc ! \
                    shout2send mount={} port={} username={} password={} ip={}",
                    path, port, username, password, address,
                ))
            }

            "rtmp" | "rtmps" => elems
                .push(format!("rtmp2sink location={}", self.to_url.as_str())),

            // FIXME: doesnt want to play
            "srt" => {
                elems.push(format!("srtsink uri={}", self.to_url.as_str()))
            }

            _ => unimplemented!(),
        };
        let cmd = elems.join("!");
        log::debug!("CopyRestreamer CMD: {}", &cmd);
        cmd
    }
}
