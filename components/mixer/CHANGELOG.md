Ephyr mixer changelog
=====================

All user visible changes to this project will be documented in this file. This project uses [Semantic Versioning 2.0.0].




## [0.4.0] · 2021-01-24
[0.4.0]: /../../tree/mixer-v0.4.0

[Diff](https://github.com/ALLATRA-IT/ephyr/compare/v0.3.0..mixer-v0.4.0)

### BC Breaks

- CLI:
    - `ephyr mix` sub-command moved out into a separate `ephyr-mixer` binary.




## [0.3.0] · 2020-08-01
[0.3.0]: /../../tree/v0.3.0

[Diff](https://github.com/ALLATRA-IT/ephyr/compare/v0.2.0..v0.3.0)

### BC Breaks

- CLI:
    - Default mixing `ephyr` command moved to `ephyr mix` sub-command.




## [0.2.0] · 2020-07-11
[0.2.0]: /../../tree/v0.2.0

[Diff](https://github.com/ALLATRA-IT/ephyr/compare/v0.1.0..v0.2.0)

### BC Breaks

- `teamspeak::Input` now produces a constant 48kHz sample rate;
- `silence::Filter` is removed (`teamspeak::Input` produces silence samples itself, when there is no audio in a [TeamSpeak] channel).

### Changed

- `ffmpeg::Mixer` now re-samples [RTMP] stream's audio to 48kHz _before_ mixing with [TeamSpeak] audio.

### Improved

- `teamspeak::Input`:
    - Emit 2-channels stereo audio ([#2]);
    - Use [Opus] FEC (forward error correction) ([#3]).
    
[#2]: /../../issues/2
[#3]: /../../issues/3




## [0.1.0] · 2020-07-04
[0.1.0]: /../../tree/v0.1.0

### Implemented

- `teamspeak::Input`:
    - Capturing audio from a [TeamSpeak] channel;
    - Mixing audio streams of multiple talkers.
- `silence::Filter`:
    - Filling audio stream with a silence if it produces no data.
- `ffmpeg::Mixer`:
    - Mixing STDIN audio stream with [RTMP] stream;
    - Correcting STDIN audio stream sample rate;
    - Delaying audio stream;
    - On-fly toggling of audio stream volume via [ZeroMQ].
- Mixing `Spec`:
    - Multiple mixing schemes for a single [RTMP] stream;
    - Multiple outputs for a single mixing result.
- [Docker] image:
    - [FFmpeg] 4.3 for mixing;
    - [SRS] 3.0 media server for receiving [RTMP] stream and optionally serving the mixing result.





[Docker]: https://www.docker.com
[FFmpeg]: https://ffmpeg.org
[Opus]: https://opus-codec.org
[RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
[Semantic Versioning 2.0.0]: https://semver.org
[SRS]: https://ossrs.net
[TeamSpeak]: https://teamspeak.com
[ZeroMQ]: https://zeromq.org
