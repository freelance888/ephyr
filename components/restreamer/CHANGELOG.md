Ephyr re-streamer changelog
===========================

All user visible changes to this project will be documented in this file. This project uses [Semantic Versioning 2.0.0].

## [0.7.0] · Unreleased
[0.7.0]: /../../tree/restreamer-v0.7.0

[Diff](/../../compare/restreamer-v0.6.0...restreamer-v0.7.0)

### Added
- Deploy:
    - Open ports 80, 8000 and 1935 ports by default ([#267]);
- Option to record audio-only files ([#239]);
### Fixed
- State disapers after restart ([#235], [#267]);
- Not possible to write DVR on local disk ([#236], [#267]);
### Miscellaneous
- Server updates:
  - [SRS] server updated to v4.0-r4 ([#244]);


[#235]: /../../issues/235
[#236]: /../../issues/236
    
[#239]: /../../pull/244
[#244]: /../../pull/244
[#267]: /../../pull/267



## [0.6.0] · 02-10-2022
[0.6.0]: /../../tree/restreamer-v0.6.0

[Diff](/../../compare/restreamer-v0.5.0...restreamer-v0.6.0)


### Added

- Web UI:
  - Output.Mixin:
    - Allow to set up to 3 Teamspeak mixers per output ([#199]);
    - Allow to specify `identity` with Teamspeak ([#6], [#39]);
    - Add `sidechain` option ([#70], [#203]);
    - Smooth `Delay` change ([#23], [#212]). 
  - Input:
    - Allow to set multiple backups per single input ([#208], [#224]);
    - Rename `origin -> primary` in case single input mode;
    - Rename `origin -> playback` and `main -> primary` in case multi input mode ([#89], [#225]);
    - Add\Cancel input endpoint label edit by Enter\Esc keypress ([#179], [#230]);
    - Make input endpoint label more visible ([#227], [#230]).

- GraphQL API:
  - Types:
    - Add `sidechain` scalar to `Mixin` ([#203]);


### Fixed
- Web UI:
  - Input:
    - Input labels disappearing after Edit Input action ([#179], [#230]);
  - Output:
    - Fast `Delay` adjust makes blaming icon constantly ([#116]).

### Miscellaneous
- Server updates:
  - [FFmpeg] from 4.4 to 5.1 ([e1faef9]);
  - [SRS] server updated to v4.0-r1 ([e1faef9], [#200]);
  - Dockerfile image moved from CentOS 7 to Ubuntu 20.04 ([#200]).

- Tech debt:
  - Split `ffmpeg.rs` to separate modules ([#202]);
  - Split `server.rs` to separate modules ([#219]);
  - Update Tokio to v1+, Actix to v4+ and related libs ([#193]);
  - Updates to webpack config and frontend tooling ([#181], [#206], [a1dcd4d]);
  - Remove state.ts in favor inner component state ([#224]);
  - Update Rust to 1.64 ([c4d7de5]).

- Gracefully shutdown [FFmpeg] processes ([#204]);
- Added test for check file recording ([#197]);
- Use FIFO for feeding data into FFmpeg in mixer output ([#199]);


[#6]: /../../issues/6
[#23]: /../../issues/23
[#70]: /../../issues/70
[#89]: /../../issues/89
[#116]: /../../issues/116
[#164]: /../../issues/164
[#179]: /../../issues/179
[#208]: /../../issues/208
[#227]: /../../issues/227

[e1faef9]: /../../commit/e1faef91cc8551505afdf7fc4622c530f9e2c6f6
[c4d7de5]: /../../commit/c4d7de52783cb6ccc69577468763119e8a9879de
[a1dcd4d]: /../../commit/a1dcd4dfc40878af1a520fd0901f4bcf1496c7fe
[#39]: /../../pull/39
[#181]: /../../pull/181
[#193]: /../../pull/193
[#197]: /../../pull/197
[#199]: /../../pull/199
[#200]: /../../pull/200
[#202]: /../../pull/202
[#203]: /../../pull/203
[#204]: /../../pull/204
[#206]: /../../pull/206
[#212]: /../../pull/212
[#219]: /../../pull/219
[#224]: /../../pull/224
[#225]: /../../pull/225
[#230]: /../../pull/230



## [0.5.0] · 2022-04-20
[0.5.0]: /../../tree/restreamer-v0.5.0

[Diff](/../../compare/restreamer-v0.4.0...restreamer-v0.5.0)

### Added

- Deploy:
  - Add support deploy from custom docker registry ([#153]);
- Web UI:
  - Update link on logo ([#156]);
  - Add server statistics for CPU, Mem and Net ([#140]);
  - Add filters and export/import for Dashboard ([#152], [#157]);
  - Show circles next to input numbers ([#154], [#157]);
  - Show in title indicator if connection to server lost ([#157]);
  - Add search by label ([#163], [#176]).

### Miscellaneous
- Server updates:
  - `SRS` server updated to v4 ([#160]).

[#154]: /../../issues/154
[#163]: /../../issues/163

[#140]: /../../pull/140
[#152]: /../../pull/152
[#153]: /../../pull/153
[#156]: /../../pull/156
[#157]: /../../pull/157
[#160]: /../../pull/160
[#176]: /../../pull/176



## [0.4.0] · 2021-11-27
[0.4.0]: /../../tree/restreamer-v0.4.0

[Diff](/../../compare/restreamer-v0.3.0...restreamer-v0.4.0)


### Added

- Deploy:
  - Replace Podman CLI with Docker CLI ([#126]);
  - Add support for VScale VPS provider ([#62], [#82]).
- CI:
  - Release to `restreamer-dev` tag with `[release]` message in commit ([#79], [#99]).
- Web UI:
  - Input:
      - Add `multiple-json` input mode ([#60], [#92]);
      - Add custom labels for inputs ([#69], [#127]).
  - Output:
    - Add `[Preview]` link to video broadcast ([#60], [#77]);
    - Add Youtube video iframe to Public Mixer Output ([#95], [#97], [#101]);
    - Allow access without password for mixing single output page ([#50], [#90]);
  - Make status filter items do not jump when status changes ([#84], [#99]);
  - Add Copy button to for Input\Output URL ([#76]);
  - Add `unstable` status output and input ([#125]).
- GraphQL API:
    - Types:
        - Update `InputKey` max key length from 20 to 50 ([#80], [#81]);
        - Add `enableConfirmation` into `Settings` ([#52], [#78]);
        - Add `id` to `Input` and `Output` ([#96], [#102]).
- Dashboard application ([#142]).

### Fixed

- Deploy:
  - Volume is not detected during Ephyr installation ([#64], [#82]).
- Web UI:
  - Error message does not disappear ([#41], [#60]);
  - Trim long output label width with ellipsis ([#107], [#111]).
- Input remains active after deletion ([#59], [#139]);
- Wrong output indication ([#65], [#139]);
- Resetting volume after and edit Output ([#72], [#143], [#148], [#150]).

### Miscellaneous

- Client updates:
    - `dns-packet` from 1.3.1 to 1.3.4 ([#75]);
    - `postcss` from 7.0.35 to 7.0.36 ([#86]);
    - `ssri` from 6.0.1 to 6.0.2 ([#87]);
    - `ws` from 5.2.2 to 5.2.3 ([#88]);
    - `tar` from 6.1.5 to 6.1.11 ([#119]);
    - `url-parse` from 1.5.1 to 1.5.3 ([#109]);
    - `path-parse` from 1.0.6 to 1.0.7 ([#108]).
- Server updates:
    - `ffmpeg` from 4.3 to 4.4 ([#85]);
    - Code style to Rust 1.54 ([#61], [#98]).

- Split implementation for Graphql Schema ([#112]);
- Move fronted to separate folder ([#117]);
- Add Cypress tests and commands to run for npm ([#121], [#123], [#124], [#133], [#144], [#145], [#151]);
- Add data-test-ids for more convenient testing ([#131]);
- Temporary disable security audit because issue with `chrono` package ([#141]).

[#41]: /../../issues/41
[#52]: /../../issues/52
[#54]: /../../issues/54
[#59]: /../../issues/59
[#62]: /../../issues/62
[#64]: /../../issues/64
[#65]: /../../issues/65
[#69]: /../../issues/69
[#72]: /../../issues/72
[#79]: /../../issues/79
[#80]: /../../issues/80
[#84]: /../../issues/84
[#95]: /../../issues/95
[#97]: /../../issues/97
[#99]: /../../issues/99
[#107]: /../../issues/107

[#60]: /../../pull/60
[#61]: /../../pull/61
[#75]: /../../pull/75
[#76]: /../../pull/76
[#77]: /../../pull/77
[#78]: /../../pull/78
[#81]: /../../pull/81
[#82]: /../../pull/82
[#85]: /../../pull/85
[#86]: /../../pull/86
[#87]: /../../pull/87
[#88]: /../../pull/88
[#90]: /../../pull/90
[#91]: /../../pull/91
[#92]: /../../pull/92
[#96]: /../../pull/96
[#98]: /../../pull/98
[#99]: /../../pull/99
[#101]: /../../pull/101
[#102]: /../../pull/102
[#108]: /../../pull/108
[#109]: /../../pull/109
[#111]: /../../pull/111
[#112]: /../../pull/112
[#117]: /../../pull/117
[#119]: /../../pull/119
[#121]: /../../pull/121
[#123]: /../../pull/123
[#124]: /../../pull/124
[#125]: /../../pull/125
[#126]: /../../pull/126
[#127]: /../../pull/127
[#131]: /../../pull/131
[#133]: /../../pull/133
[#139]: /../../pull/139
[#141]: /../../pull/141
[#142]: /../../pull/142
[#143]: /../../pull/143
[#144]: /../../pull/144
[#145]: /../../pull/145
[#148]: /../../pull/148
[#150]: /../../pull/150
[#151]: /../../pull/151



## [0.3.0] · 2021-05-11
[0.3.0]: /../../tree/restreamer-v0.3.0

[Diff](/../../compare/restreamer-v0.2.0...restreamer-v0.3.0)

### Added

- Web UI:
    - Settings:
        - Add setting title for Ephyr server and info popup ([#43], [#35]);
        - Add setting for optional strict mode for input and output deletion ([#57], [#45]);
    - Add section with `Start all` and `Stop all` buttons and amounts of `Input`s and `Output`s ([#49], [#35]);
    - Restrict error message length and allow copying text ([#53], [#50]).
- GraphQL API:
    - Types:
        - Create `Settings` struct with fields `password_hash` and `title` ([#43], [#35]);
        - Add `settings` field into `State` ([#43], [#35]);
        - Add `delete_confirmation` into `Settings` ([#57], [#45]).
  - Mutations:
        - Add `setSettings` with argument `title` and `deleteConfirmation` ([#43], [#35], [#57], [#45]);
        - Add `disableAllOutputsOfRestreams` and `enablesAllOutputsOfRestreams` ([#49], [#35]).
  - Subscriptions:
        - Add `title` field to `Info` subscription ([#43], [#35]);
        - Add `delete_confirmation` field to `Info` subscription ([#57], [#45]).

### Fixed

- Web UI:
    - Layout issues on small screen ([#49]).

[#35]: /../../issues/35
[#45]: /../../issues/45
[#50]: /../../issues/50
[#43]: /../../pull/43
[#49]: /../../pull/49
[#53]: /../../pull/53
[#57]: /../../pull/57



## [0.2.0] · 2021-03-18
[0.2.0]: /../../tree/restreamer-v0.2.0

[Diff](/../../compare/restreamer-v0.1.2...restreamer-v0.2.0)

### BC Breaks

- Web UI:
    - Input:
        - Remove distinguishing between pull or push endpoint in add/edit modal window ([9e1ac1c7]).
- GraphQL API:
    - Types:
        - Rename root types to `Query`, `Mutation` and `Subscription` ([9e1ac1c7]);
        - Rework fields of `Restream` and `Input` objects ([9e1ac1c7]);
        - Remove `PushInput` and `PullInput` objects ([9e1ac1c7]).
    - Mutations:
        - Replace `addPullInput` and `addPushInput` with `setRestream` ([9e1ac1c7]);
        - Replace `addOutput` with `setOutput` ([740fa998], [#28]);
        - Rename `removeInput` to `removeRestream` an change its argument's type ([9e1ac1c7]);
        - Add `restreamId` argument to `enableInput` and `disableInput` ([9e1ac1c7]);
        - Replace `inputId` argument with `restreamId` in `addOutput`, `removeOutput`, `enableOutput`, `disableOutput`, `enableAllOutputs` and `disableAllOutputs` ([9e1ac1c7]);
        - Rename `outputId` argument to `id` in `removeOutput`, `enableOutput` and `disableOutput` ([9e1ac1c7]);
        - Use `OutputDstUrl` and `MixinSrcUrl` scalars instead of `Url` in `addOutput` ([9e1ac1c7]);
        - Use `Label` scalar instead of `String` in `addOutput` ([9e1ac1c7]).
    - Queries:
        - Rename `restreams` to `allRestreams` ([9e1ac1c7]).
    - Subscriptions:
        - Rename `restreams` to `allRestreams` ([9e1ac1c7]).

### Added

- Web UI:
    - Input:
        - Optional backup endpoint (push or pull) ([a3236808], [9e1ac1c7]);
        - Ability to export/import as JSON spec ([9e1ac1c7]);
        - Optional [HLS] endpoint ([65f8b86e]);
        - Ability to pull from [HLS] HTTP URL ([2e4d46ae], [#27]);
        - Confirmation window on removing ([9acf42e2]).
    - Output:
        - Specifying [TeamSpeak] URL for mixing ([77d25dd7], [#23]);
        - Specifying [MP3] HTTP URL for mixing ([e96b39f1], [#30]);
        - Tuning and toggling volume rate of tracks ([77d25dd7], [a2c5f83f], [#23]);
        - Tuning delay of a mixed-in [TeamSpeak] track ([77d25dd7], [#23]);
        - Separate page for mixing a single output ([8103cb32], [#29]);
        - [Icecast] URL as supported destination ([5dabcfdc]);
        - [SRT] URL as supported destination ([d397aaaf], [#21]);
        - [FLV] `file:///` URL as supported destination ([46c85d4d], [#26]);
        - Ability to show, download and remove recorded [FLV] files ([46c85d4d], [#26]);
        - Ability to edit an existing output ([740fa998], [#28]);
        - Confirmation window on removing ([9acf42e2]).
    - Copying URLs to clipboard by double-click ([62355b8f]).
- GraphQL API:
    - Types:
        - `Mixin` object ([77d25dd7], [#23]);
        - `MixinId`, `Volume` and `Delay` scalars ([77d25dd7], [#23]);
        - `RestreamId` scalar ([9e1ac1c7]);
        - `Label` scalar ([9e1ac1c7]);
        - `InputSrcUrl`, `OutputDstUrl` and `MixinSrcUrl` scalars ([5dabcfdc], [9e1ac1c7]);
        - `RestreamKey` and `InputKey` scalars ([9e1ac1c7]);
        - `InputSrc` union with `RemoteInputSrc` and `FailoverInputSrc` variants ([9e1ac1c7]);
        - `InputEndpoint` object, `InputEndpointKind` enum and `EndpointId` scalar ([65f8b86e]).
    - Mutations:
        - `enableRestream` and `disableRestream` ([9e1ac1c7]);
        - `tuneVolume` and `tuneDelay` ([77d25dd7], [#23]);
        - `mix` argument to `addOutput` ([77d25dd7], [#23]);
        - `import` ([9e1ac1c7]);
        - `removeDvrFile` ([46c85d4d], [#26]).
    - Queries:
        - `Output.volume` and `Output.mixins` fields ([77d25dd7], [#23]);
        - `export` ([9e1ac1c7]);
        - `dvrFiles` ([46c85d4d], [#26]).
- Spec (export/import):
    - `v1` version ([9e1ac1c7]).
- Config:
    - `--srs-http-dir` CLI option and `EPHYR_RESTREAMER_SRS_HTTP_DIR` env var ([65f8b86e]).
- Deployment:
    - Provision script for [Ubuntu] 20.04:
        - Optional [firewalld] installation via `WITH_FIREWALLD` env var ([bbccc004]);
        - Auto-detection and usage of [DigitalOcean] and [Hetzner Cloud] mounted external volumes ([46c85d4d], [#26]).
- Documentation:
    - Deployment instructions:
        - [Oracle Cloud Infrastructure] on English and Russian languages ([9c7a9c71]);
        - Mounting additional volume on [DigitalOcean] and [Hetzner Cloud] ([46c85d4d], [#26]).

[#21]: /../../issues/21
[#23]: /../../issues/23
[#26]: /../../issues/26
[#27]: /../../issues/27
[#28]: /../../issues/28
[#29]: /../../issues/29
[#30]: /../../issues/30
[2e4d46ae]: /../../commit/2e4d46ae929da87fb78f3fa312768bd4e3693e38
[46c85d4d]: /../../commit/46c85d4d67e7b8a0efb91444f94f3575f9dfa665
[5dabcfdc]: /../../commit/5dabcfdce2420fdd43a8f4c20c2eff497e884ac3
[62355b8f]: /../../commit/62355b8f8b8f10fa3c1b6f21c9cfc86eef519211
[65f8b86e]: /../../commit/65f8b86eebad0396ef37f1df27548e70952eef63
[740fa998]: /../../commit/740fa9985feae057ecea758292bcf1c2d2758988
[77d25dd7]: /../../commit/77d25dd739d4f05b319769eddd83c01bd3a490a4
[8103cb32]: /../../commit/8103cb32c1f0e71f13907fc9917c8bcf66c51696
[9acf42e2]: /../../commit/9acf42e26aa3089688378a25871cc341cd0ab04e
[9c7a9c71]: /../../commit/9c7a9c7105324ca198eb322071ced35f53413b00
[9e1ac1c7]: /../../commit/9e1ac1c7e576c22f6234777bf01d054adb9fe5db
[a2c5f83f]: /../../commit/a2c5f83ff55f078f242f3beb6d2310a24c835c98
[a3236808]: /../../commit/a3236808c43d1c5667cac4b3037d7c83edccc48f
[bbccc004]: /../../commit/bbccc0040d95d47a72c3bf7c6fc0908f32c89bd4
[d397aaaf]: /../../commit/d397aaafde43c98e34837273926b5672df2449fe
[e96b39f1]: /../../commit/e96b39f1fd3f249b1befd0db4db745e5a495b62d




## [0.1.2] · 2021-02-13
[0.1.2]: /../../tree/restreamer-v0.1.2

[Diff](/../../compare/restreamer-v0.1.1...restreamer-v0.1.2)

### Fixed

- Deployment:
    - Provision script for [Ubuntu] 20.04:
        - Incorrect default registry pick up by [Podman] ([43bb1948]).

[43bb1948]: /../../commit/43bb1948297a6864affbf098498e4e9810358e0e




## [0.1.1] · 2021-02-05
[0.1.1]: /../../tree/restreamer-v0.1.1

[Diff](/../../compare/restreamer-v0.1.0...restreamer-v0.1.1)

### Fixed

- Broken [GraphQL Playground] in debug mode ([3bcbfa07]).

[3bcbfa07]: /../../commit/3bcbfa073bdd13bb401d0f625509d4dea392f32e




## [0.1.0] · 2021-01-26
[0.1.0]: /../../tree/restreamer-v0.1.0

[Diff](/../../compare/v0.3.6...restreamer-v0.1.0)

### Implemented

- Web UI:
    - Input:
        - Push type to accept [RTMP] stream;
        - Pull type to automatically pull [RTMP] stream from remote server;
        - Optional label;
        - Status indication (offline, connecting, online);
        - Ability to enable/disable a single input;
        - Ability to enable/disable all outputs for a single input;
        - Editing an existing input;
        - Displaying a total count of outputs by their statuses along with the filtering. 
    - Output:
        - Optional label;
        - Adding multiple endpoints via CSV list;
        - Status indication (offline, connecting, online);
        - Ability to enable/disable a single output.
    - Optional password protection via [Basic HTTP auth].
- GraphQL API:
    - Types:
        - `Info` object;
        - `Restream` object;
        - `Input` union, `PushInput` and `PullInput` objects;
        - `Output` object;
        - `InputId`, `OutputId` scalars;
        - `Status` enum.
    - Mutations:
        - `addPullInput`, `addPushInput`, `removeInput`;
        - `enableInput`, `disableInput`;
        - `addOutput`, `removeOutput`;
        - `enableOutput`, `disableOutput`;
        - `enableAllOutputs`, `disableAllOutputs`;
        - `setPassword`.
    - Queries:
        - `info`;
        - `restreams`.
    - Subscriptions:
        - `info`;
        - `restreams`.
- Deployment:
    - [Docker] image;
    - Provision script for [Ubuntu] 20.04.
- Documentation:
    - Deployment instructions for [DigitalOcean] and [Hetzner Cloud] on English and Russian languages.





[Basic HTTP auth]: https://en.wikipedia.org/wiki/Basic_access_authentication
[DigitalOcean]: https://www.digitalocean.com
[Docker]: https://www.docker.com
[firewalld]: https://firewalld.org
[FLV]: https://en.wikipedia.org/wiki/Flash_Video
[GraphQL]: https://www.graphql.com
[GraphQL Playground]: https://github.com/graphql/graphql-playground
[Hetzner Cloud]: https://www.hetzner.com/cloud
[HLS]: https://en.wikipedia.org/wiki/HTTP_Live_Streaming
[Icecast]: https://icecast.org
[MP3]: https://en.wikipedia.org/wiki/MP3
[Oracle Cloud Infrastructure]: https://www.oracle.com/cloud
[Podman]: https://podman.io
[RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
[Semantic Versioning 2.0.0]: https://semver.org
[SRT]: https://en.wikipedia.org/wiki/Secure_Reliable_Transport
[TeamSpeak]: https://teamspeak.com 
[Ubuntu]: https://ubuntu.com
[FFmpeg]: https://ffmpeg.org
[SRS]: https://github.com/ossrs/srs

