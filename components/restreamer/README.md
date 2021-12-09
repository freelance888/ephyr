Ephyr re-streamer
=================

[Changelog](CHANGELOG.md)

🚀 Deploy to [DigitalOcean][101] ([ru][102]), [Hetzner Cloud][111] ([ru][112]), [Oracle Cloud][121] ([ru][122]), [VScale\Selectel (ru)][123].

Simple web application allowing to forward [RTMP] streams in a similar way as [facecast.io] does. It uses [SRS] to accept [RTMP] streams and [FFmpeg] to forward them.


## Contributing
Read [CONTRIBUTING.md](https://github.com/ALLATRA-IT/ephyr/blob/master/components/restreamer/CONTRIBUTING.md)

## License

Ephyr is subject to the terms of the [Blue Oak Model License 1.0.0](/../../blob/master/LICENSE.md). If a copy of the [BlueOak-1.0.0](https://spdx.org/licenses/BlueOak-1.0.0.html) license was not distributed with this file, You can obtain one at <https://blueoakcouncil.org/license/1.0.0>.

[SRS] is licensed under the [MIT license](https://github.com/ossrs/srs/blob/3.0release/LICENSE).

[FFmpeg] is generally licensed under the [GNU Lesser General Public License (LGPL) version 2.1](http://www.gnu.org/licenses/old-licenses/lgpl-2.1.html). To consider exceptions read the [FFmpeg License and Legal Considerations](https://www.ffmpeg.org/legal.html).

As with all Docker images, these likely also contain other software which may be under other licenses (such as Bash, etc from the base distribution, along with any direct or indirect dependencies of the primary software being contained).

As for any pre-built image usage, it is the image user's responsibility to ensure that any use of this image complies with any relevant licenses for all software contained within.





[facecast.io]: https://facecast.io
[FFmpeg]: https://ffmpeg.org
[RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
[SRS]: https://github.com/ossrs/srs

[101]: docs/deploy_digitalocean_EN.md
[102]: docs/deploy_digitalocean_RU.md
[111]: docs/deploy_hcloud_EN.md
[112]: docs/deploy_hcloud_RU.md
[121]: docs/deploy_oracle_EN.md
[122]: docs/deploy_oracle_RU.md
[123]: docs/deploy_vscale_RU.md
