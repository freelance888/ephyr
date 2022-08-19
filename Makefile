###############################
# Common defaults/definitions #
###############################

comma := ,

# Checks two given strings for equality.
eq = $(if $(or $(1),$(2)),$(and $(findstring $(1),$(2)),\
                                $(findstring $(2),$(1))),1)




######################
# Project parameters #
######################

IMAGE_NAME ?= allatra/ephyr
IMAGE_TAG ?= dev
GH_REPO ?= ALLATRA-IT/ephyr



###########
# Aliases #
###########

fmt: cargo.fmt


image: docker.image




##################
# Cargo commands #
##################

# Format Rust sources with rustfmt.
#
# Usage:
#	make cargo.fmt [check=(no|yes)]

cargo.fmt:
	cargo +nightly fmt --all $(if $(call eq,$(check),yes),-- --check,)




###################
# Docker commands #
###################

docker-comp = $(if $(call eq,$(comp),),restreamer,$(comp))


# Stop project in Docker Compose development environment
# and remove all related containers.
#
# Usage:
#	make docker.down [app=(mix|vod)]

docker.down:
	docker-compose --file=$(docker-compose-file) down --rmi=local -v


# Build project Docker image.
#
# Usage:
#	make docker.image [comp=(restreamer|vod-meta-server)]
#	                  [tag=($(IMAGE_TAG)|<tag>)]
#	                  [no-cache=(no|yes)]

docker-image-tag = $(if $(call eq,$(tag),),$(IMAGE_TAG),$(tag))

docker.image:
ifeq ($(docker-comp),restreamer)
	@cd components/$(docker-comp)/ && make docker.image tag=$(tag)
else
	docker build --network=host --force-rm \
		$(if $(call eq,$(no-cache),yes),--no-cache --pull,) \
		--file=components/$(docker-comp)/Dockerfile \
		-t $(IMAGE_NAME):$(docker-comp)$(if \
			$(call eq,$(docker-image-tag),latest),,-$(docker-image-tag)) \
		./
endif


# Push project Docker images to Container Registry.
#
# Usage:
#	make docker.push [tags=($(IMAGE_TAG)|<t1>[,<t2>...])]
#	                 [comp=(restreamer|vod-meta-server)]

docker-push-tags = $(if $(call eq,$(tags),),$(IMAGE_TAG),$(tags))

docker.push:
	$(foreach t,$(subst $(comma), ,$(docker-push-tags)),\
		$(call docker.push.do,\
			$(IMAGE_NAME):$(docker-comp)$(if $(call eq,$(t),latest),,-$(t))))
define docker.push.do
	$(eval image-full := $(strip $(1)))
	docker push $(image-full)
endef


# Tag project Docker image with given tags.
#
# Usage:
#	make docker.tag [of=($(IMAGE_TAG)|<tag>)]
#	                [tags=($(IMAGE_TAG)|<with-t1>[,<with-t2>...])]
#	                [comp=(restreamer|vod-meta-server)]

docker-tag-of = $(if $(call eq,$(of),),$(IMAGE_TAG),$(of))
docker-tag-with = $(if $(call eq,$(tags),),$(IMAGE_TAG),$(tags))

docker.tag:
	$(foreach tag,$(subst $(comma), ,$(docker-tag-with)),\
		$(call docker.tag.do,$(tag)))
define docker.tag.do
	$(eval tag := $(strip $(1)))
	docker tag \
		$(IMAGE_NAME):$(docker-comp)-$(docker-tag-of) \
		$(IMAGE_NAME):$(docker-comp)$(if $(call eq,$(tag),latest),,-$(tag))
endef


# Save project Docker images to a tarball file.
#
# Usage:
#	make docker.tar [to-file=(.cache/image.tar|<file-path>)]
#	                [comp=(restreamer|vod-meta-server)]
#	                [tags=($(IMAGE_TAG)|<t1>[,<t2>...])]

docker-tar-file = $(if $(call eq,$(to-file),),.cache/image.tar,$(to-file))
docker-tar-tags = $(if $(call eq,$(tags),),$(IMAGE_TAG),$(tags))

docker.tar:
	@mkdir -p $(dir $(docker-tar-file))
	docker save -o $(docker-tar-file) \
		$(foreach tag,$(subst $(comma), ,$(docker-tar-tags)),\
			$(IMAGE_NAME):$(docker-comp)$(if $(call eq,$(tag),latest),,-$(tag)))


# Load project Docker images from a tarball file.
#
# Usage:
#	make docker.untar [from-file=(.cache/image.tar|<file-path>)]

docker.untar:
	docker load -i $(if $(call eq,$(from-file),),.cache/image.tar,$(from-file))




###########################
# .Github Actions section #
###########################


# Clear Github Acions usage cache.
#
# Need to install github cli first [https://cli.github.com/] and authorize
#
# Usage:
#	make gh.clear

gh.clear:
	gh api -H "Accept: application/vnd.github+json" \
		/repos/$(GH_REPO)/actions/caches \
		| for ID in `jq '.actions_caches[].id'`; \
		  do echo "Deleting $$ID"; \
		     gh api --method DELETE /repos/$(GH_REPO)/actions/caches/$$ID | echo; done





##################
# .PHONY section #
##################

.PHONY: fmt image \
        cargo.fmt \
        docker.image docker.push docker.tag docker.tar docker.untar
