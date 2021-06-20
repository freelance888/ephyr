Contribution Guide
==================

Any helpful information regarding developing this project and contributing to it is documented in this file.




## Prerequisites

Any commands provided below should be run in `components/restreamer/` directory of this project.




## Local development

As `ephyr-restreamer` requires binaries of correctly compiled [SRS] and [FFmpeg] to be present, it's quite difficult to run it via `cargo run` directly. Build [Docker] image containing all the required environment.
```bash
make image

# and run it
make up
make up rebuild=yes  # or re-build and run

# or in background
make up background=yes

# and to stop it
make down
```

[GraphQL Playground] is accessible on <http://127.0.0.1/api/playground>.

To re-run changes of backend part, unfortunately, you should re-build [Docker] image via `make image` command every time. So, consider to use `cargo check` as much as it gives, before re-running backend.

For Web UI, fortunately, it's possible to run [webpack-dev-server] and do not re-build [Docker] image on changes.
```bash
yarn dev  # http://127.0.0.1:8080
```




## Formatting, linting, documenting and testing

For backend part use:
```bash
make fmt
cargo lint  # or make lint
cargo doc   # to check docs integrity
cargo test
```

For Web UI use:
```bash
yarn fmt
yarn lint
yarn doc
```




## Update GraphQL schema

To regenerate `client.graphql.schema.json` use:
```bash
make graphql.schema
```




## DigitalOcean development

It's possible to run a development build on [DigitalOcean] droplet easily:
```bash
make do.up

# or re-build and use fresh Docker image 
# and fresh DigitalOcean droplet
make do.up fresh=yes

# and to destroy the droplet
make do.down
```

To do so, you should have `doctl` and `jq` binaries installed (on macOS they're installed automatically via [Homebrew]) and `DO_API_TOKEN` env var being set. Optionally, specifying `DO_SSH_KEY` will create [DigitalOcean] droplets with the specified [SSH] key, so you can access them for debugging.

To simplify `DO_API_TOKEN` and `DO_SSH_KEY` env vars setup, consider to create `.envrc` file with the following content:
```bash
#!/usr/bin/env bash

export DO_API_TOKEN='<your-digitalocean-token-here>'
export DO_SSH_KEY=$(curl -s -X GET -H 'Content-Type: application/json' \
                                   -H "Authorization: Bearer $DO_API_TOKEN" \
                         'https://api.digitalocean.com/v2/account/keys' \
                    | jq '(.ssh_keys[] | select(.name=="<your-ssh-key-name-on-digitalocean>")).id')
```
and use it anytime you need:
```bash
source .envrc
```




## Release process

By default, each commit in `master` branch deploys to `restreamer-edge` image on [ALLATRA IT Docker Hub].

For release specific version we need to:

1. Update version and description `CHAGNELOG.md`;
2. Update version in `Cargo.toml`;
3. Update versions in `docs`;
4. Add tag `restreamer-v0.0.0` or so;
5. Push branch with tags.
6. Add announce in https://t.me/ephyr_updates




## Commit message commands
By default we run all steps on CI except release. Release happens only if branch is `master` or commit pushed with tags `restreamer-v` and `vod-meta-server-v`.

We have configured several actions for CI that could be triggered by adding specific command in the beginning of the commit message.

Supported commands:
1. `[release]` -- release current commit version to `restreamer-dev` image;
2. `[skip ci]` -- skip run of all steps on GitHub CI;
3. `[fresh ci]` -- clean all previous cache and run CI.




[DigitalOcean]: https://digitalocean.com
[Docker]: https://docker.io
[FFmpeg]: https://ffmpeg.org
[GraphQL Playground]: https://github.com/graphql/graphql-playground
[Homebrew]: https://brew.sh
[SSH]: https://en.wikipedia.org/wiki/SSH_(Secure_Shell)
[SRS]: https://github.com/ossrs/srs
[webpack-dev-server]: https://www.npmjs.com/package/webpack-dev-server
[ALLATRA IT Docker Hub]: https://hub.docker.com/repository/docker/allatra/ephyr/tags
