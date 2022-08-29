//! Kind of a [FFmpeg] re-streaming process that re-streams a live stream from
//! one URL endpoint to another one "as is", without performing any live stream
//! modifications, optionally transmuxing it to the destination format.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::path::Path;

use gst::{prelude::*, Pipeline};
use gstreamer as gst;
use tokio::{io, process::Command};
use url::Url;
use uuid::Uuid;

use crate::dvr;

/// Kind of a [FFmpeg] re-streaming process that re-streams a live stream from
/// one URL endpoint to another one "as is", without performing any live stream
/// modifications, optionally transmuxing it to the destination format.
///
/// [FFmpeg]: https://ffmpeg.org
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
    pub fn new(id: Uuid, from_url: Url, to_url: Url) -> Self {
        Self {
            id,
            from_url,
            to_url,
        }
    }

    /// Checks whether this [`CopyRestreamer`] process must be restarted, as
    /// cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    #[inline]
    #[must_use]
    pub fn needs_restart(&self, actual: &Self) -> bool {
        self.from_url != actual.from_url || self.to_url != actual.to_url
    }

    /// Properly setups the given [FFmpeg] [`Command`] for this
    /// [`CopyRestreamer`] before running it.
    ///
    /// # Errors
    ///
    /// If the given [FFmpeg] [`Command`] fails to be setup.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub(crate) async fn setup_ffmpeg(
        &self,
        cmd: &mut Command,
    ) -> io::Result<()> {
        let _ = match self.from_url.scheme() {
            "http" | "https"
                if Path::new(self.from_url.path()).extension()
                    == Some("m3u8".as_ref()) =>
            {
                cmd.arg("-re")
            }

            "rtmp" | "rtmps" => cmd,

            _ => unimplemented!(),
        }
        .args(&["-i", self.from_url.as_str()]);

        let _ = match self.to_url.scheme() {
            "file"
                if Path::new(self.to_url.path()).extension()
                    == Some("flv".as_ref()) =>
            {
                cmd.args(&["-c", "copy"])
                    .arg(dvr::new_file_path(&self.to_url).await?)
            }

            "icecast" => cmd
                .args(&["-c:a", "libmp3lame", "-b:a", "64k"])
                .args(&["-f", "mp3", "-content_type", "audio/mpeg"])
                .arg(self.to_url.as_str()),

            "rtmp" | "rtmps" => cmd
                .args(&["-c", "copy"])
                .args(&["-f", "flv"])
                .arg(self.to_url.as_str()),

            "srt" => cmd
                .args(&["-c", "copy"])
                .args(&["-strict", "-2", "-y", "-f", "mpegts"])
                .arg(self.to_url.as_str()),

            _ => unimplemented!(),
        };

        Ok(())
    }

    pub(crate) fn setup_pipeline(
        &self,
        pipeline: &mut Pipeline,
    ) -> io::Result<()> {
        let source = match self.from_url.scheme() {
            "rtmp" | "rtmps" => {
                let src =
                    gst::ElementFactory::make("rtmp2src", Some("copy-src"))
                        .expect("Could not create rtmp2src");
                src.set_property_from_str("location", self.from_url.as_str());
                src
            }
            _ => unimplemented!(),
        };
        let sink = match self.to_url.scheme() {
            "rtmp" | "rtmps" => {
                let sink =
                    gst::ElementFactory::make("rtmp2sink", Some("copy-out"))
                        .expect("Could not create rtmp2sink");
                sink.set_property_from_str("location", self.to_url.as_str());
                sink
            }
            _ => unimplemented!(),
        };

        let gst_elements = &[&source, &sink];

        pipeline.add_many(gst_elements).unwrap();

        gst::Element::link_many(gst_elements)
            .expect("Elements could not be linked");

        source.connect_pad_added(move |src, src_pad| {
            println!("Received new pad {:?} from {:?}", src_pad, src);
        });
        Ok(())
    }
}
