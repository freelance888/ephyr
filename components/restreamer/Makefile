###############################
# Common defaults/definitions #
###############################

comma := ,

# Checks two given strings for equality.
eq = $(if $(or $(1),$(2)),$(and $(findstring $(1),$(2)),\
                                $(findstring $(2),$(1))),1)

OS_NAME := $(shell uname -s)




######################
# Project parameters #
######################

IMAGE_NAME ?= allatra/ephyr
IMAGE_TAG ?= dev




###########
# Aliases #
###########

down: docker.down


fmt: cargo.fmt


image: docker.image


lint: cargo.lint


up: docker.up




##################
# Cargo commands #
##################

# Format Rust sources with rustfmt.
#
# Usage:
#	make cargo.fmt [check=(no|yes)]

cargo.fmt:
	cargo +nightly fmt --all $(if $(call eq,$(check),yes),-- --check,)


# Lint Rust sources with Clippy.
#
# Usage:
#	make cargo.lint

cargo.lint:
	cargo clippy --all -- -D clippy::pedantic -D warnings




#########################
# DigitalOcean commands #
#########################

do-droplet-name ?= ephyr-restreamer-dev

# Creates new DigitalOcean droplet.
define do.droplet.create
	$()
	sed "s/EPHYR_VER:-edge/EPHYR_VER:-dev/g;\
		$(if $(call eq,$(debug),no),,\
			s/EPHYR_CLI_ARGS:-''/EPHYR_CLI_ARGS:-'--debug'/g)" \
		deploy/provision/ubuntu-20-04-x64.sh \
		> do.deploy.sh
	doctl -t $$DO_API_TOKEN compute droplet create \
		--image ubuntu-20-04-x64 \
		--size s-1vcpu-1gb \
		--region ams3 \
		--user-data-file=do.deploy.sh \
		$(if $(call eq,$(DO_SSH_KEY),),,\
			--ssh-keys $(DO_SSH_KEY)) \
		$(do-droplet-name)
	sleep $(if $(call eq,$(wait),),10,$(wait))
endef

# Destroys existing DigitalOcean droplet.
define do.droplet.destroy
	$()
	doctl -t $$DO_API_TOKEN compute droplet delete $(do-droplet-name) --force
endef

# Checks whether DigitalOcean droplet exists.
define do.droplet.exists
	$(strip $(shell doctl -t $$DO_API_TOKEN -o json compute droplet list \
	                | jq -r '.[].name' \
	                | grep -Fx $(do-droplet-name)))
endef


# Stop project in DigitalOcean droplet and destroy the droplet.
#
# Usage:
#	make do.down

do.down: do.setup do.token.verify
	$(if $(call eq,$(strip $(call do.droplet.exists)),),,\
		$(call do.droplet.destroy))


# Setup toolchain to interact with DigitalOcean droplets.
#
# Usage:
#	make do.setup

do.setup:
ifeq ($(OS_NAME),Darwin)
ifeq ($(strip $(shell which doctl)),)
	brew install doctl
endif
ifeq ($(strip $(shell which jq)),)
	brew install jq
endif
endif


# Ensure DigitalOcean API token is defined and correct.
#
# Usage:
#	make do.token.verify

do.token.verify:
ifeq ($(DO_API_TOKEN),)
	$(error "Specify $$DO_API_TOKEN: export DO_API_TOKEN=<do-token-here>"),)
else
	$(if $(call eq,$(strip \
		$(shell doctl -t $$DO_API_TOKEN -o json compute droplet list \
		        | grep '401 Unable to authenticate')),),,\
			$(error "Specified $$DO_API_TOKEN is invalid"))
endif


# Run project in DigitalOcean droplet.
#
# Usage:
#	make do.up [debug=(yes|no)] [fresh=(no|yes)]

do.up: do.setup do.token.verify
ifeq ($(fresh),yes)
	@make docker.image tag=dev
	@make docker.push tags=dev
	$(if $(call eq,$(strip $(call do.droplet.exists)),),,\
		$(call do.droplet.destroy))
	$(call do.droplet.create)
else
	$(if $(call eq,$(strip $(call do.droplet.exists)),),\
		$(call do.droplet.create),)
endif
	@echo "Checkout http://$$(doctl -t $$DO_API_TOKEN -o json compute droplet list \
	                          | jq -r '.[] | select(.name == "$(do-droplet-name)") \
	                                       | .networks.v4[] | select(.type == "public") \
	                                       | .ip_address')/"




###################
# Docker commands #
###################

# Stop project in Docker development environment and remove all related
# containers.
#
# Usage:
#	make docker.down

docker.down:
	-docker stop ephyr-restreamer-dev


# Build project Docker image.
#
# Usage:
#	make docker.image [tag=($(IMAGE_TAG)|<tag>)] [no-cache=(no|yes)]

docker-image-tag = $(if $(call eq,$(tag),),$(IMAGE_TAG),$(tag))

docker.image:
	cd ../../ && \
	docker build --network=host --force-rm --platform linux/amd64 \
		$(if $(call eq,$(no-cache),yes),--no-cache --pull,) \
		--file=components/restreamer/Dockerfile \
		-t $(IMAGE_NAME):restreamer$(if \
			$(call eq,$(docker-image-tag),latest),,-$(docker-image-tag)) \
		./


# Push project Docker images to container registry.
#
# Usage:
#	make docker.push [tags=($(IMAGE_TAG)|<t1>[,<t2>...])]

docker-push-tags = $(if $(call eq,$(tags),),$(IMAGE_TAG),$(tags))

docker.push:
	$(foreach tag,$(subst $(comma), ,$(docker-push-tags)),\
		$(call docker.push.do,$(tag)))
define docker.push.do
	$(eval tag := $(strip $(1)))
	docker push $(IMAGE_NAME):restreamer$(if $(call eq,$(tag),latest),,-$(tag))
endef


# Run project in Docker development environment.
#
# Usage:
#	make docker.up [debug=(yes|no)] [background=(no|yes)]
#	               [( [rebuild=no]
#	                | rebuild=yes [tag=($(IMAGE_TAG)|<tag>)]
#	                              [no-cache=(no|yes)] )]

docker.up: docker.down
ifeq ($(rebuild),yes)
	@make docker.image tag=$(tag) no-cache=$(no-cache)
endif
	docker run --rm --name ephyr-restreamer-dev --platform linux/amd64 \
	           $(if $(call eq,$(background),yes),-d,-it) \
	           -p 80:80 -p 1935:1935 -p 8000:8000 \
	           -v '$(PWD)/state.json:/state.json' \
	           -v '$(PWD)/../../.cache/restreamer/dvr/:/var/www/srs/dvr/' \
		$(IMAGE_NAME):restreamer-$(IMAGE_TAG) \
			$(if $(call eq,$(debug),no),,--debug)




####################
# GraphQL commands #
####################

# Generate Client GraphQL schema JSON from project sources.
#
# Usage:
#	make graphql.schema.client

graphql.schema.client:
	cargo run --bin export_schema -- --api=client --out-dir=./


# Generate Mix GraphQL schema JSON from project sources.
#
# Usage:
#	make graphql.schema.mix

graphql.schema.mix:
	cargo run --bin export_schema -- --api=mix --out-dir=./


# Generate Dashboard GraphQL schema JSON from project sources.
#
# Usage:
#	make graphql.schema.dashboard

graphql.schema.dashboard:
	cargo run --bin export_schema -- --api=dashboard --out-dir=./


# Generate All GraphQL schema JSON from project sources.
#
# Usage:
#	make graphql.schema

graphql.schema:	graphql.schema.client graphql.schema.dashboard graphql.schema.mix


##################
# .PHONY section #
##################

.PHONY: down fmt lint up \
        cargo.fmt cargo.lint \
        do.down do.setup do.token.verify do.up \
        docker.down docker.image docker.push docker.up \
        graphql.schema.client graphql.schema.mix
