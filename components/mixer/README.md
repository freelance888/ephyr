Ephyr mixer
===========

[Changelog](CHANGELOG.md)

Wrapper over [FFmpeg] binary, which performs mixing according to specified schema (see [example][1]). At the moment, it's intended to be called as [SRS] `exec.publish` directive, so performs mixing on-demand (when [RTMP] stream is pushed to [SRS]).

Ephyr mixer is able to capture audio from [TeamSpeak] server, and feed it to [FFmpeg] for mixing with [RTMP] stream.

See `ephyr-mixer --help` for details.




## Try it out


### [SRS] + [FFmpeg] server-side mixing

To boot up a simple example, run:
```bash
make up
```

Now, publish an RTMP stream to `rtmp://127.0.0.1:1935/input/mic` endpoint either with [OBS], or any other RTMP publisher. You may also use `FFmpeg` for that:
```bash
make publish
```

Finally, play the resulting mixed RTMP stream:
```bash
make play
```

Also, you may tune volume on-fly:
```bash
make tune volume=1 track=music
make tune volume=0.4 track=original
```




## License

Ephyr is subject to the terms of the [Blue Oak Model License 1.0.0](https://github.com/ALLATRA-IT/ephyr/blob/master/LICENSE.md). If a copy of the [BlueOak-1.0.0](https://spdx.org/licenses/BlueOak-1.0.0.html) license was not distributed with this file, You can obtain one at <https://blueoakcouncil.org/license/1.0.0>.

[SRS] is licensed under the [MIT license](https://github.com/ossrs/srs/blob/3.0release/LICENSE).

[FFmpeg] is generally licensed under the [GNU Lesser General Public License (LGPL) version 2.1](http://www.gnu.org/licenses/old-licenses/lgpl-2.1.html). To consider exceptions read the [FFmpeg License and Legal Considerations](https://www.ffmpeg.org/legal.html).

As with all Docker images, these likely also contain other software which may be under other licenses (such as Bash, etc from the base distribution, along with any direct or indirect dependencies of the primary software being contained), including libraries used by [FFmpeg].

As for any pre-built image usage, it is the image user's responsibility to ensure that any use of this image complies with any relevant licenses for all software contained within.





[FFmpeg]: https://ffmpeg.org
[OBS]: https://obsproject.com
[RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
[Rust]: https://www.rust-lang.org
[SRS]: https://github.com/ossrs/srs
[TeamSpeak]: https://teamspeak.com

[1]: example.mix.spec.json
