//! Inner state of a [VOD] meta server.
//!
//! It holds a collection of [`Playlist`]s, each of which has a week-based
//! schedule of [`Clip`]s (each weekday has its own collection of [`Clip`]s).
//!
//! The total duration of all [`Clip`]s in the one weekday hasn't to be exactly
//! 24 hours, but cannot be more than that, and has to be a fraction of 24
//! hours. This is this dictated by the necessity to correctly loop the
//! weekday's playlist to fill the whole 24 hours.
//!
//! [`Clip`]: crate::vod::meta::state::Clip
//! [`Playlist`]: crate::vod::meta::state::Playlist
//! [VOD]: https://en.wikipedia.org/wiki/Video_on_demand

pub mod manager;

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
    time::Duration,
};

use anyhow::anyhow;
use chrono::{
    DateTime, Datelike as _, Duration as DateDuration, FixedOffset as TimeZone,
    Utc, Weekday,
};
use derive_more::{Deref, DerefMut, Display, Into};
use ephyr_serde::{timelike, timezone};
use futures::{stream, StreamExt as _, TryFutureExt as _, TryStreamExt as _};
use isolang::Language;
use mime::Mime;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use smart_default::SmartDefault;
use url::Url;

use crate::{
    api::{self, allatra, nginx},
    vod::file,
};

pub use crate::api::allatra::video::{Resolution, YoutubeId};

pub use self::manager::Manager;

/// State of the server, representing a set of [`Playlist`]s for different
/// audiences.
#[derive(Clone, Debug, Default, Deref, DerefMut, Deserialize, Serialize)]
pub struct State(HashMap<PlaylistSlug, Playlist>);

impl State {
    /// Parses new [`State`] from the given API request.
    ///
    /// # Errors
    ///
    /// If some [`Playlist`] fails to parse.
    pub async fn parse_request(
        req: api::vod::meta::Request,
    ) -> Result<Self, anyhow::Error> {
        // We don't process each playlist concurrently to avoid performing too
        // many concurrent requests to `allatra::video::Api`.
        Ok(Self(
            stream::iter(req.into_iter())
                .then(|(pl_slug, pl)| Playlist::parse_request(pl_slug, pl))
                .map_ok(|pl| (pl.slug.clone(), pl))
                .try_collect()
                .await?,
        ))
    }
}

/// Playlist of [`Clip`]s to be played for some audience.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Playlist {
    /// [URL slug][1] of this [`Playlist`] to display in URLs referring to it.
    ///
    /// [1]: https://en.wikipedia.org/wiki/Clean_URL#Slug
    pub slug: PlaylistSlug,

    /// Human-readable title of this [`Playlist`].
    pub title: String,

    /// Language of the audience this [`Playlist`] is intended for.
    pub lang: Language,

    /// Timezone of the audience this [`Playlist`] is intended for.
    ///
    /// [`Playlist::clips`] are scheduled in this timezone according to the
    /// provided [`Weekday`]s.
    #[serde(with = "timezone")]
    pub tz: TimeZone,

    /// Duration of segments to serve [`Playlist`]'s [`Clip`]s with.
    ///
    /// All [`Clip`]s in [`Playlist`] are mandatory cut to this duration
    /// segments when are served. That's why [`Clip`]'s duration should divide
    /// on [`SegmentDuration`] without any fractions.
    #[serde(default)]
    pub segment_duration: SegmentDuration,

    /// Set of [`Clip`]'s [`Resolution`]s provided by this [`Playlist`].
    ///
    /// If empty then all available [`Clip`]'s [`Resolution`]s will be used,
    /// considering the [`Playlist::mutual_resolutions`] limitation.
    #[serde(default)]
    pub resolutions: HashSet<Resolution>,

    /// Initial position of this [`Playlist`] to start building
    /// [`nginx::vod_module::mapping`] schedule from.
    ///
    /// If [`None`] then today in the [`Playlist`]'s timezone will be used as
    /// the starting point.
    ///
    /// Any call of [`Playlist::schedule_nginx_vod_module_set`] method may
    /// update this field and will initialize it automatically in case it's
    /// [`None`].
    #[serde(default)]
    pub initial: Option<PlaylistInitialPosition>,

    /// [`Clip`]s which form this [`Playlist`], distributed by [`Weekday`]s.
    ///
    /// The total duration of all [`Clip`]s in the one [`Weekday`] hasn't to be
    /// exactly 24 hours, but cannot be more than that. Also, 24 hours should
    /// divide on that duration without any fractions. This is this dictated by
    /// the necessity to correctly loop the weekday's playlist to fill the whole
    /// 24 hours.
    ///
    /// All the [`Clip`]s provided for a single [`Weekday`] will be scheduled
    /// one after another sequentially, in the order they were provided, and
    /// without any gaps between them.
    pub clips: HashMap<Weekday, Vec<Clip>>,
}

impl Playlist {
    /// Hydrates the intersection of video resolutions provided by all
    /// [`Playlist`]'s [`Clip`]s returning a set of mutual resolutions (such
    /// ones that all [`Clip`]s have them).
    #[must_use]
    pub fn mutual_resolutions(&self) -> HashSet<Resolution> {
        let mut mutual: Option<HashSet<Resolution>> = None;
        for clips in self.clips.values() {
            for clip in clips {
                #[allow(clippy::option_if_let_else)]
                if let Some(m) = &mut mutual {
                    m.retain(|r| clip.sources.contains_key(r))
                } else {
                    mutual = Some(clip.sources.keys().copied().collect())
                }
            }
        }
        mutual.unwrap_or_default()
    }

    /// Parses new [`Playlist`] from the given `vod-meta` server API request.
    ///
    /// # Errors
    ///
    /// - If [`Playlist`] has empty title.
    /// - If all [`Clip`]s in [`Playlist`] don't fit well into 24 hours.
    /// - If any weekday doesn't have at least one clip.
    /// - If some [`Clip`] fails to parse.
    pub async fn parse_request(
        slug: PlaylistSlug,
        req: api::vod::meta::Playlist,
    ) -> Result<Self, anyhow::Error> {
        // We limit concurrent requests to `allatra::video::Api` to avoid
        // possible rate-limiting.
        const CONCURRENT_REQUESTS: usize = 10;
        const SECS_IN_DAY: u64 = 86400;

        if req.title.is_empty() {
            return Err(anyhow!(
                "Playlist '{}' shouldn't have empty title",
                slug,
            ));
        }

        let segment_duration = req.segment_duration.unwrap_or_default();
        let resolutions = &req.resolutions;
        let clips =
            stream::iter(req.clips.into_iter().flat_map(|(day, clips)| {
                clips.into_iter().map(move |c| (day, c))
            }))
            .map(|(day, req)| {
                Clip::parse_request(req, segment_duration, resolutions)
                    .map_ok(move |c| (day, c))
            })
            .buffered(CONCURRENT_REQUESTS)
            .try_fold(
                <HashMap<_, Vec<_>>>::new(),
                |mut all, (day, clip)| async move {
                    all.entry(day).or_default().push(clip);
                    Ok(all)
                },
            )
            .await?;

        if clips.len() != 7 {
            return Err(anyhow!(
                "Playlist '{}' should have all weekdays filled",
                slug,
            ));
        }
        for (weekday, clips) in &clips {
            if clips.is_empty() {
                return Err(anyhow!(
                    "Day {} of playlist '{}' has no clips, but should have at \
                     least one",
                    weekday,
                    req.title,
                ));
            }
            let total_duration: Duration =
                clips.iter().map(|c| c.view.to - c.view.from).sum();
            if total_duration.as_secs() > SECS_IN_DAY {
                return Err(anyhow!(
                    "Total duration of all clips in day {} of playlist '{}' \
                     is more than 24 hours",
                    weekday,
                    req.title,
                ));
            }
            if SECS_IN_DAY % total_duration.as_secs() != 0 {
                return Err(anyhow!(
                    "Total duration of all clips in day {} of playlist '{}' \
                     is not fraction of 24 hours",
                    weekday,
                    req.title,
                ));
            }
        }

        Ok(Playlist {
            slug,
            title: req.title,
            lang: req.lang,
            tz: req.tz,
            segment_duration,
            resolutions: req.resolutions,
            initial: None,
            clips,
        })
    }

    /// Inspects all [`Src`]s of this [`Playlist`] and fills them with
    /// information about [VOD] files available in the given `cache`.
    ///
    /// # Errors
    ///
    /// If some [`Src`] is not supported to reside in `cache`.
    ///
    /// [VOD]: https://en.wikipedia.org/wiki/Video_on_demand
    pub async fn fill_with_cache_files(
        &mut self,
        cache: &file::cache::Manager,
    ) -> Result<(), anyhow::Error> {
        for clips in self.clips.values_mut() {
            for cl in clips.iter_mut() {
                for src in cl.sources.values_mut() {
                    if src.url.local.is_some() {
                        continue;
                    }
                    if let Some(path) = cache
                        .get_cached_path(&src.url.upstream)
                        .await
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get cached file path for '{}' \
                                 URL: {}",
                                src.url.upstream,
                                e,
                            )
                        })?
                    {
                        src.url.local = Some(Url::parse(&format!(
                            "file:///{}",
                            path.display(),
                        ))?);
                    }
                }
            }
        }
        Ok(())
    }

    /// Schedules the given [`Playlist`] to be played by [`nginx-vod-module`][1]
    /// starting from `at` time with at least `count` [`Clip`]s scheduled.
    ///
    /// If `at` is [`None`] then now (current time) will be used.
    ///
    /// If `count` is invalid (zero or more than
    /// [`mapping::Set::MAX_DURATIONS_LEN`][2]) then it will be reset to the
    /// [`mapping::Set::MAX_DURATIONS_LEN`][2] value.
    ///
    /// # Preconditions
    ///
    /// The [`Playlist`] is assumed to have at least one [`Clip`] in all
    /// weekdays.
    ///
    /// [`Clip`]s duration is assumed to divide on
    /// [`Playlist::segment_duration`] without remainder.
    ///
    /// Total [`Clip`]s duration in a single weekday is not bigger than 24
    /// hours.
    ///
    /// # Algorithm
    ///
    /// Schedule is created starting from the given `at` time and until the
    /// `count` limitation allows.
    ///
    /// Each day is fully filled with clips without any gaps (looping the
    /// weekday's [`Clip`]s), if it has at least one [`Clip`].
    ///
    /// All [`Clip`]s are scheduled in the [`Playlist`]'s timezone.
    ///
    /// Algorithm automatically cares about segment indexing in
    /// [`nginx-vod-module`][1] being monotonically increasing in a correct way,
    /// without any gaps, basing on the provided [`PlaylistInitialPosition`] and
    /// updating it whenever it's feasible.
    ///
    /// [1]: https://github.com/kaltura/nginx-vod-module
    /// [2]: crate::api::nginx::vod_module::mapping::Set::MAX_DURATIONS_LEN
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn schedule_nginx_vod_module_set(
        &mut self,
        at: Option<DateTime<Utc>>,
        mut count: usize,
    ) -> nginx::vod_module::mapping::Set {
        use nginx::vod_module::mapping;

        let mut set = mapping::Set {
            id: Some(self.slug.clone().into()),
            playlist_type: mapping::PlaylistType::Live,
            discontinuity: true,
            segment_duration: Some(self.segment_duration.as_duration().into()),
            ..mapping::Set::default()
        };

        if !(1..=mapping::Set::MAX_DURATIONS_LEN).contains(&count) {
            count = mapping::Set::MAX_DURATIONS_LEN;
        }

        let resolutions = if self.resolutions.is_empty() {
            // Because all `mapping::Set::sequences` must have the same length,
            // we should define the minimal mutual intersection of all
            // resolutions and use only them to form a `mapping::Set`.
            self.mutual_resolutions()
        } else {
            self.resolutions.clone()
        };
        if resolutions.is_empty() {
            return set;
        }
        // Preserving the same order matters here, that's why we use `BTreeMap`.
        let mut sequences: BTreeMap<_, _> = resolutions
            .iter()
            .map(|&r| {
                let sequence = mapping::Sequence {
                    id: Some(format!("{}p", r as u16)),
                    language: Some(self.lang),
                    label: Some(format!("{}p", r as u16)),
                    ..mapping::Sequence::default()
                };
                (r, sequence)
            })
            .collect();

        let segment_duration_secs =
            self.segment_duration.as_duration().as_secs();

        let now = at.unwrap_or_else(Utc::now).with_timezone(&self.tz);
        let today = now.date().and_hms(0, 0, 0);

        let (mut clip_index, mut segment_index, mut start_time) =
            self.initial.as_ref().map_or_else(
                || (0, 0, today),
                |init| {
                    let at = init.at.with_timezone(&self.tz);
                    (init.clip_index, init.segment_index, at)
                },
            );

        'whole_loop: loop {
            let day = start_time.date().and_hms(0, 0, 0);
            let next_day = day + DateDuration::days(1);

            if let Some(day_clips) = self.clips.get(&day.weekday()) {
                let mut time = day;

                // Unfortunately, nginx-vod-module loops the whole playlist
                // only, and is unable to loop a part of playlist in the given
                // time window. That's why, to loop all clips of the current day
                // without affecting next day's playlist, we need to repeat the
                // playlist manually, until the next day comes.
                'day_loop: while time < next_day {
                    for clip in day_clips {
                        let clip_duration = clip.view.to - clip.view.from;
                        let next_time = time
                            + DateDuration::from_std(clip_duration).unwrap();

                        // There is no sense to return clips, which have been
                        // already finished. Instead, we start from the first
                        // non-finished today's clip. This way we reserve more
                        // space for future clips, considering the
                        // nginx-vod-module's `mapping::Set::MAX_DURATIONS_LEN`
                        // limitation.
                        //
                        // A drift in 1 minute is required to omit "clip is
                        // absent" errors when its playing segment is requested
                        // slightly after the current clip changes (due to the
                        // fact that HTTP requests from client are not an
                        // immediate thing). This way the metadata for all
                        // requested segments remains valid at any time.
                        if (next_time + DateDuration::minutes(1)) > now {
                            if set.initial_clip_index.is_none() {
                                set.initial_clip_index = Some(clip_index);
                                set.initial_segment_index = Some(segment_index);

                                // Update the playlist's initial position to the
                                // most recent one.
                                self.initial = Some(PlaylistInitialPosition {
                                    clip_index,
                                    segment_index,
                                    at: time.with_timezone(&Utc),
                                });
                            }

                            for (size, seq) in &mut sequences {
                                if let Some(src) = clip.sources.get(&size) {
                                    let path =
                                        mapping::SourceClip::get_url_path(
                                            src.url
                                                .local
                                                .as_ref()
                                                .unwrap_or(&src.url.upstream),
                                        );
                                    seq.clips.push(mapping::Clip {
                                        r#type: mapping::SourceClip {
                                            path,
                                            from: Some(clip.view.from.into()),
                                            to: Some(clip.view.to.into()),
                                        }
                                        .into(),
                                    });
                                }
                            }

                            set.clip_times
                                .push(time.clone().with_timezone(&Utc).into());

                            set.durations.push(clip_duration.into());
                            if set.durations.len() >= count {
                                break 'whole_loop;
                            }
                        }

                        // If there is some `self.initial` state, then we should
                        // ensure that we count indices starting from the
                        // specified initial time, not the day's beginning.
                        if time >= start_time {
                            clip_index += 1;
                            segment_index +=
                                clip_duration.as_secs() / segment_duration_secs;
                        }

                        time = next_time;
                        if time >= next_day {
                            break 'day_loop;
                        }
                    }
                }
            }

            start_time = next_day;
        }

        set.sequences = sequences.into_iter().map(|(_, seq)| seq).collect();
        set
    }
}

/// Position of a [`Playlist`] indicating a fixed point in time to start
/// building [`nginx::vod_module::mapping`] schedule from and initial [`Clip`]
/// and segment indices that should be used for that.
///
/// This position is intended to be continuously updated in time following the
/// schedule updates, so any repeated requests from [`nginx-vod-module`][1] will
/// receive a correct schedule guaranteeing smooth transitions between [`Clip`]s
/// without any interruptions ever, even when [`Clip`]s are removed from the
/// schedule's head.
///
/// [1]: https://github.com/kaltura/nginx-vod-module
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaylistInitialPosition {
    /// Index of a [`Clip`] that should play at the
    /// [`PlaylistInitialPosition::at`] time.
    pub clip_index: u64,

    /// Index of a segment that should play at the
    /// [`PlaylistInitialPosition::at`] time.
    pub segment_index: u64,

    /// Fixed point in time that this [`PlaylistInitialPosition`] is applicable
    /// at.
    pub at: DateTime<Utc>,
}

/// [URL slug][1] of a [`Playlist`].
///
/// [1]: https://en.wikipedia.org/wiki/Clean_URL#Slug
#[derive(Clone, Debug, Display, Eq, Hash, Into, PartialEq, Serialize)]
pub struct PlaylistSlug(String);

impl PlaylistSlug {
    /// Creates new [`PlaylistSlug`] from the given `slug` string if it
    /// represents a [valid slug][1].
    ///
    /// [1]: https://en.wikipedia.org/wiki/Clean_URL#Slug
    #[must_use]
    pub fn new<S: AsRef<str> + Into<String>>(slug: S) -> Option<Self> {
        if Self::validate(&slug) {
            Some(Self(slug.into()))
        } else {
            None
        }
    }

    /// Validates whether the given `slug` string represents a [valid slug][1].
    ///
    /// [1]: https://en.wikipedia.org/wiki/Clean_URL#Slug
    #[must_use]
    pub fn validate<S: AsRef<str> + ?Sized>(slug: &S) -> bool {
        static SLUG_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap());

        let slug = slug.as_ref();
        !slug.is_empty() && SLUG_REGEX.is_match(slug)
    }
}

impl<'de> Deserialize<'de> for PlaylistSlug {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;
        Self::new(<Cow<'_, str>>::deserialize(deserializer)?)
            .ok_or_else(|| D::Error::custom("not a valid URL slug"))
    }
}

/// Clip in a [`Playlist`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Clip {
    /// ID of this [`Clip`] on [YouTube].
    ///
    /// [YouTube]: https://youtube.com
    pub youtube_id: YoutubeId,

    /// Human-readable title of this [`Clip`].
    pub title: String,

    /// Time window of this [`Clip`] in its source file to be played.
    pub view: ClipView,

    /// Source files of this [`Clip`] distributed by their video [`Resolution`].
    pub sources: HashMap<Resolution, Src>,
}

impl Clip {
    /// Parses new [`Clip`] from the given `vod-meta` server API request, with
    /// accordance to the given [`SegmentDuration`].
    ///
    /// # Errors
    ///
    /// - If [`Clip`] has empty title.
    /// - If incorrect [`Clip`]'s [YouTube] video URL is provided.
    /// - If [`Clip`] info cannot be retrieved from [`allatra::video::Api`].
    /// - If [`Clip`]'s duration is incorrect.
    ///
    /// [YouTube]: https://youtube.com
    pub async fn parse_request(
        req: api::vod::meta::Clip,
        segment_duration: SegmentDuration,
        resolutions: &HashSet<Resolution>,
    ) -> Result<Self, anyhow::Error> {
        if req.title.is_empty() {
            return Err(anyhow!(
                "Clip with URL '{}' shouldn't have empty title",
                req.url,
            ));
        }

        let youtube_id = Self::parse_youtube_id(&req.url).map_err(|e| {
            anyhow!(
                "Incorrect video link '{}' provided for clip '{}': {}",
                req.url,
                req.title,
                e,
            )
        })?;

        let resp = allatra::video::Api::get_videos_yt(&youtube_id)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to retrieve info about clip '{}' by the provided \
                     URL '{}': {}",
                    req.title,
                    req.url,
                    e,
                )
            })?;

        if req.from >= resp.duration {
            return Err(anyhow!(
                "Clip '{}' cannot start from {}, because video's total \
                 duration is {}",
                req.title,
                timelike::format(&req.from),
                timelike::format(&resp.duration),
            ));
        }
        if req.to > resp.duration {
            return Err(anyhow!(
                "Clip '{}' cannot finish at {}, because video's total duration \
                 is {}",
                req.title,
                timelike::format(&req.to),
                timelike::format(&resp.duration),
            ));
        }
        if req.to.checked_sub(req.from).unwrap_or_default()
            < Duration::from_secs(1)
        {
            return Err(anyhow!(
                "Clip '{}' should start before it ends at {}, but it starts \
                 from {}",
                req.title,
                timelike::format(&req.to),
                timelike::format(&req.from),
            ));
        }

        let clip_secs = (req.to - req.from).as_secs();
        let segment_secs = segment_duration.as_duration().as_secs();
        if clip_secs % segment_secs != 0 {
            return Err(anyhow!(
                "Duration of clip '{}' should be divisible on {} seconds \
                 segment duration, but it is {} seconds",
                req.title,
                segment_secs,
                clip_secs,
            ));
        }

        let sources: HashMap<_, _> = resp
            .sources
            .into_iter()
            .map(|source| {
                let src = Src {
                    url: SrcUrl {
                        upstream: source.src,
                        local: None,
                    },
                    mime_type: source.r#type,
                    size: source.size,
                };
                (source.size, src)
            })
            .collect();

        for r in resolutions {
            if !sources.contains_key(r) {
                return Err(anyhow!(
                    "Clip '{}' has no {}p resolution required by playlist",
                    req.title,
                    *r as u16,
                ));
            }
        }

        Ok(Self {
            youtube_id,
            title: req.title,
            view: ClipView {
                from: req.from,
                to: req.to,
            },
            sources,
        })
    }

    /// Validates whether the given [`Url`] is a correct [YouTube] video link
    /// and parses ID of the video from it.
    ///
    /// # Errors
    ///
    /// - If [`Url`]'s scheme is not `http`/`https`.
    /// - If [`Url`]'s host is not `youtube.com`.
    /// - If [`Url`]'s path is not `watch`.
    /// - If [`Url`]'s query misses `v` parameter.
    ///
    /// [YouTube]: https://youtube.com
    pub fn parse_youtube_id(url: &Url) -> Result<YoutubeId, anyhow::Error> {
        if !matches!(url.scheme().to_lowercase().as_str(), "http" | "https") {
            return Err(anyhow!("Only HTTP YouTube URLs are supported"));
        }
        if !matches!(
            url.host_str(),
            Some("youtube.com") | Some("www.youtube.com")
        ) {
            return Err(anyhow!("Only YouTube URLs are supported"));
        }
        if url.path().trim_end_matches('/') != "/watch" {
            return Err(anyhow!("Only full YouTube URLs are supported"));
        }
        url.query_pairs()
            .find_map(
                |(name, id)| if name == "v" { Some(id.into()) } else { None },
            )
            .ok_or_else(|| anyhow!("YouTube URL should contain video ID"))
    }
}

/// Time window in a source file to play in a [`Clip`]. Also, defines duration
/// of a [`Clip`].
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ClipView {
    /// Starting timing position in a source file to play from in a [`Clip`].
    #[serde(with = "timelike")]
    pub from: Duration,

    /// Finish timing position in a source file to play until in a [`Clip`].
    ///
    /// Obviously, should be always greater than [`ClipView::from`] for at least
    /// 1 second.
    #[serde(with = "timelike")]
    pub to: Duration,
}

/// Source file of a [`Clip`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Src {
    /// URL of this source file, where it can be read from.
    pub url: SrcUrl,

    /// [MIME type][1] of this source file.
    ///
    /// [1]: https://en.wikipedia.org/wiki/Media_type
    #[serde(rename = "type", with = "mime_serde_shim")]
    pub mime_type: Mime,

    /// Resolution of the video contained in this source file.
    pub size: Resolution,
}

/// [URL] of a [`Clip`]'s source file.
///
/// [URL]: https://en.wikipedia.org/wiki/URL
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SrcUrl {
    /// Remote URL of the original source file on upstream server.
    ///
    /// Supports `http://` and `https://` schemes only.
    pub upstream: Url,

    /// Local URL of the locally cached version of the source file in the
    /// [VOD] files cache directory (NOT the absolute path in filesystem).
    ///
    /// Supports `file://` scheme only.
    ///
    /// [VOD]: https://en.wikipedia.org/wiki/Video_on_demand
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local: Option<Url>,
}

/// Duration of a [`Clip`]'s segment.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Into, PartialEq, Serialize, SmartDefault,
)]
pub struct SegmentDuration(
    #[default(Duration::from_secs(10))]
    #[serde(with = "serde_humantime")]
    Duration,
);

impl SegmentDuration {
    /// Creates new [`SegmentDuration`] from the given [`Duration`] if it
    /// represents a [valid segment duration][1].
    ///
    /// [1]: SegmentDuration::validate
    #[must_use]
    pub fn new(dur: Duration) -> Option<Self> {
        if Self::validate(dur) {
            Some(Self(dur))
        } else {
            None
        }
    }

    /// Validates whether the given [`Duration`] represents a valid
    /// [`SegmentDuration`].
    ///
    /// Valid segment durations are between 5 and 30 seconds (inclusively).
    #[inline]
    #[must_use]
    pub fn validate(dur: Duration) -> bool {
        (5..=30).contains(&dur.as_secs())
    }

    /// Converts this [`SegmentDuration`] to a regular [`Duration`] value.
    #[inline]
    #[must_use]
    pub fn as_duration(&self) -> Duration {
        self.0
    }
}

impl<'de> Deserialize<'de> for SegmentDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;
        Self::new(serde_humantime::deserialize(deserializer)?)
            .ok_or_else(|| D::Error::custom("not a valid segment duration"))
    }
}

#[cfg(test)]
mod spec {
    use std::fs;

    use super::*;

    #[test]
    fn deserializes_example() {
        let serialized =
            fs::read("example.vod.meta.json").expect("No example file found");
        let res = serde_json::from_slice::<State>(&serialized);

        assert!(res.is_ok(), "deserialization fails: {}", res.unwrap_err());
    }

    mod playlist_slug {
        use super::*;

        #[test]
        fn allows_valid_slugs() {
            for (input, desc) in &[
                ("sdf0sdf0", "single segment"),
                ("0-0-0-df", "multiple segments"),
            ] {
                let actual = PlaylistSlug::new(*input);
                assert!(actual.is_some(), "disallows {}", desc);
            }
        }

        #[test]
        fn disallows_invalid_slugs() {
            for (input, desc) in &[
                ("", "empty value"),
                ("-djksf0", "starting with hyphen"),
                ("djksf0-", "ending with hyphen"),
                ("djk-s___f0", "incorrect symbols"),
            ] {
                let actual = PlaylistSlug::new(*input);
                assert!(actual.is_none(), "allows {}", desc);
            }
        }
    }

    mod segment_duration {
        use super::*;

        #[test]
        fn allows_valid_durations() {
            for input in &[5, 10, 30] {
                let actual = SegmentDuration::new(Duration::from_secs(*input));
                assert!(actual.is_some(), "disallows {} seconds", input);
            }
        }

        #[test]
        fn disallows_invalid_slugs() {
            for input in &[1, 31, 60] {
                let actual = SegmentDuration::new(Duration::from_secs(*input));
                assert!(actual.is_none(), "allows {} seconds", input);
            }
        }
    }

    mod clip {
        use super::*;

        #[tokio::test]
        async fn parses_valid_request() {
            let req = serde_json::from_str::<api::vod::meta::Clip>(
                r#"{
                  "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "1:50:20"
                }"#,
            )
            .expect("Failed to deserialize request");

            let res = Clip::parse_request(
                req,
                SegmentDuration::default(),
                &HashSet::default(),
            )
            .await;
            assert!(res.is_ok(), "failed to parse: {}", res.unwrap_err());

            let clip = res.unwrap();
            assert_eq!(clip.youtube_id, "0wAtNWA93hM".into());
            assert_eq!(&clip.title, "Круг Жизни");
            assert_eq!(clip.view.from, Duration::from_secs(0));
            assert_eq!(clip.view.to, Duration::from_secs(6620));
            assert_eq!(clip.sources.len(), 5);
        }

        #[tokio::test]
        async fn disallows_non_youtube_url() {
            for json in &[
                r#"{
                  "url": "https://vimeo.com/watch?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "0:00:10"
                }"#,
                r#"{
                  "url": "https://www.youtube.com/dfsdf?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "0:00:10"
                }"#,
                r#"{
                  "url": "https://www.youtube.com/watch?vd=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "0:00:10"
                }"#,
                r#"{
                  "url": "https://www.youtube.com/watch",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "0:00:10"
                }"#,
            ] {
                let req = serde_json::from_str::<api::vod::meta::Clip>(&json)
                    .expect("Failed to deserialize request");

                let res = Clip::parse_request(
                    req,
                    SegmentDuration::default(),
                    &HashSet::default(),
                )
                .await;
                assert!(res.is_err(), "allows non-YouTube URL in: {}", json);
            }
        }

        #[tokio::test]
        async fn disallows_invalid_duration() {
            for json in &[
                r#"{
                  "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "0:00:00"
                }"#,
                r#"{
                  "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:01",
                  "to": "0:00:00"
                }"#,
                r#"{
                  "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "02:00:00",
                  "to": "02:00:03"
                }"#,
                r#"{
                  "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "02:00:03"
                }"#,
                r#"{
                  "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                  "title": "Круг Жизни",
                  "from": "00:00:00",
                  "to": "1:50:23"
                }"#,
            ] {
                let req = serde_json::from_str::<api::vod::meta::Clip>(&json)
                    .expect("Failed to deserialize request");

                let res = Clip::parse_request(
                    req,
                    SegmentDuration::default(),
                    &HashSet::default(),
                )
                .await;
                assert!(res.is_err(), "allows invalid duration in: {}", json);
            }
        }
    }

    mod playlist {
        use super::*;

        #[tokio::test]
        async fn parses_valid_request() {
            let slug = PlaylistSlug::new("life").unwrap();
            let req = serde_json::from_str::<api::vod::meta::Playlist>(
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:30:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:30:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:30:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:30:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:30:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:30:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:30:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }]
                  }
                }"#,
            )
            .expect("Failed to deserialize request");

            let res = Playlist::parse_request(slug.clone(), req).await;
            assert!(res.is_ok(), "failed to parse: {}", res.unwrap_err());

            let pl = res.unwrap();
            assert_eq!(pl.slug, slug);
            assert_eq!(&pl.title, "Передачи с Игорем Михайловичем");
            assert_eq!(pl.lang, Language::from_639_1("ru").unwrap());
            assert_eq!(pl.tz, TimeZone::east(3 * 3600));
            assert_eq!(pl.clips.len(), 7);
            assert!(pl.clips.contains_key(&Weekday::Mon), "incorrect weekday");
            assert_eq!(pl.clips.get(&Weekday::Mon).unwrap().len(), 2);
        }

        #[tokio::test]
        async fn disallows_invalid_clip() {
            let slug = PlaylistSlug::new("life").unwrap();
            for json in &[
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:00:00"
                    }],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }]
                  }
                }"#,
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "tue": [{
                      "url": "https://vimeo.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }]
                  }
                }"#,
            ] {
                let req =
                    serde_json::from_str::<api::vod::meta::Playlist>(&json)
                        .expect("Failed to deserialize request");

                let res = Playlist::parse_request(slug.clone(), req).await;
                assert!(res.is_err(), "allows invalid clip in value: {}", json);
            }
        }

        #[tokio::test]
        async fn disallows_non_24_hours_fractioned_weekday_clips_duration() {
            let slug = PlaylistSlug::new("life").unwrap();
            for json in &[
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:32:57"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }]
                  }
                }"#,
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "1:51:26"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "1:00:00"
                    }]
                  }
                }"#,
            ] {
                let req =
                    serde_json::from_str::<api::vod::meta::Playlist>(&json)
                        .expect("Failed to deserialize request");

                let res = Playlist::parse_request(slug.clone(), req).await;
                assert!(
                    res.is_err(),
                    "allows non-24-hours fractioned total duration in: {}",
                    json,
                );
            }
        }

        #[tokio::test]
        async fn disallows_more_than_24_hours_weekday_clips_duration() {
            let slug = PlaylistSlug::new("life").unwrap();
            for json in &[
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=R29rL-CIsbo",
                      "title": "Сознание и Личность",
                      "from": "00:00:00",
                      "to": "7:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=R29rL-CIsbo",
                      "title": "Сознание и Личность",
                      "from": "00:00:00",
                      "to": "7:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=R29rL-CIsbo",
                      "title": "Сознание и Личность",
                      "from": "00:00:00",
                      "to": "7:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=R29rL-CIsbo",
                      "title": "Сознание и Личность",
                      "from": "00:00:00",
                      "to": "7:00:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }]
                  }
                }"#,
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }, {
                      "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI",
                      "title": "ПРАВДА ЖИЗНИ",
                      "from": "00:00:00",
                      "to": "2:00:00"
                    }],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }]
                  }
                }"#,
            ] {
                let req =
                    serde_json::from_str::<api::vod::meta::Playlist>(&json)
                        .expect("Failed to deserialize request");

                let res = Playlist::parse_request(slug.clone(), req).await;
                assert!(
                    res.is_err(),
                    "allows more than 24 hours total duration in: {}",
                    json,
                );
            }
        }

        #[tokio::test]
        async fn disallows_missing_weekdays() {
            let slug = PlaylistSlug::new("life").unwrap();
            for json in &[
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }]
                  }
                }"#,
                r#"{
                  "title": "Передачи с Игорем Михайловичем",
                  "lang": "rus",
                  "tz": "+03:00",
                  "segment_duration": "10s",
                  "clips": {
                    "mon": [],
                    "tue": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "wed": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "thu": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "fri": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sat": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }],
                    "sun": [{
                      "url": "https://www.youtube.com/watch?v=0wAtNWA93hM",
                      "title": "Круг Жизни",
                      "from": "00:00:00",
                      "to": "0:10:00"
                    }]
                  }
                }"#,
            ] {
                let req =
                    serde_json::from_str::<api::vod::meta::Playlist>(&json)
                        .expect("Failed to deserialize request");

                let res = Playlist::parse_request(slug.clone(), req).await;
                assert!(res.is_err(), "allows missing weekday in: {}", json);
            }
        }

        mod schedule {
            use chrono::TimeZone as _;

            use super::*;

            #[tokio::test]
            async fn considers_drift_for_delayed_requests() {
                let slug = PlaylistSlug::new("life").unwrap();
                let req = serde_json::from_str::<api::vod::meta::Playlist>(
                    r#"{
                      "title": "Life",
                      "lang": "eng",
                      "tz": "+02:00",
                      "clips": {
                        "mon": [{
                          "from": "00:00:00",
                          "to": "00:02:00",
                          "title": "Life circle",
                          "url": "https://www.youtube.com/watch?v=0wAtNWA93hM"
                        }, {
                          "from": "00:00:00",
                          "to": "00:01:00",
                          "title": "Truth of Life",
                          "url": "https://www.youtube.com/watch?v=Q69gFVmrCiI"
                        }],
                        "tue": [{
                          "from": "00:00:00",
                          "to": "00:01:00",
                          "title": "Life circle",
                          "url": "https://www.youtube.com/watch?v=0wAtNWA93hM"
                        }],
                        "wed": [{
                          "from": "00:00:00",
                          "to": "00:01:00",
                          "title": "Life circle",
                          "url": "https://www.youtube.com/watch?v=0wAtNWA93hM"
                        }],
                        "thu": [{
                          "from": "00:00:00",
                          "to": "00:01:00",
                          "title": "Life circle",
                          "url": "https://www.youtube.com/watch?v=0wAtNWA93hM"
                        }],
                        "fri": [{
                          "from": "00:00:00",
                          "to": "00:01:00",
                          "title": "Life circle",
                          "url": "https://www.youtube.com/watch?v=0wAtNWA93hM"
                        }],
                        "sat": [{
                          "from": "00:00:00",
                          "to": "00:01:00",
                          "title": "Life circle",
                          "url": "https://www.youtube.com/watch?v=0wAtNWA93hM"
                        }],
                        "sun": [{
                          "from": "00:00:00",
                          "to": "00:01:00",
                          "title": "Life circle",
                          "url": "https://www.youtube.com/watch?v=0wAtNWA93hM"
                        }]
                      }
                    }"#,
                )
                .expect("Failed to deserialize request");

                let mut pl = Playlist::parse_request(slug.clone(), req)
                    .await
                    .expect("Failed to parse playlist");

                // Prefill initial position.
                let at = Utc.ymd(2020, 9, 12).and_hms(22, 0, 0);
                let _ = pl.schedule_nginx_vod_module_set(Some(at), 1);

                assert_eq!(
                    pl.initial,
                    Some(PlaylistInitialPosition {
                        clip_index: 0,
                        segment_index: 0,
                        at,
                    }),
                );

                let at = Utc.ymd(2020, 9, 13).and_hms(22, 0, 4);
                let schedule = pl.schedule_nginx_vod_module_set(Some(at), 3);

                assert_eq!(schedule.durations.len(), 3);
                assert_eq!(schedule.clip_times.len(), 3);
                assert_eq!(schedule.sequences.len(), 5);
                for seq in &schedule.sequences {
                    assert_eq!(seq.clips.len(), 3);
                }

                assert_eq!(schedule.initial_clip_index, Some(1439));
                assert_eq!(schedule.initial_segment_index, Some(8634));

                assert_eq!(
                    *schedule.clip_times.get(0).unwrap(),
                    Utc.ymd(2020, 9, 13).and_hms(21, 59, 0).into(),
                );
                assert_eq!(
                    *schedule.clip_times.get(1).unwrap(),
                    Utc.ymd(2020, 9, 13).and_hms(22, 0, 0).into(),
                );
                assert_eq!(
                    *schedule.clip_times.get(2).unwrap(),
                    Utc.ymd(2020, 9, 13).and_hms(22, 2, 0).into(),
                );
            }
        }
    }
}
