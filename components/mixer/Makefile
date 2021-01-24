###############################
# Common defaults/definitions #
###############################

# Checks two given strings for equality.
eq = $(if $(or $(1),$(2)),$(and $(findstring $(1),$(2)),\
                                $(findstring $(2),$(1))),1)

OS_NAME := $(shell uname -s)




###########
# Aliases #
###########

down: docker.down


up: docker.up




########################
# Interaction commands #
########################

# List to STDOUT available audio/video devices with FFmpeg.
#
# Usage:
#	make devices.list

devices.list:
ifeq ($(OS_NAME),Darwin)
	-ffmpeg -f avfoundation -list_devices true -i ''
else
	$(error "'devices.list' command is not implemented for your OS")
endif


# Play mixed RTMP stream from Origin SRS.
#
# Usage:
#	make play [stream=(output/musicify_mic|<app>/<stream>)]

play-stream = $(if $(call eq,$(stream),),output/musicify_mic,$(stream))

play:
	ffplay -rtmp_live 1 rtmp://127.0.0.1:1935/$(play-stream)


# Publish raw local camera RTMP stream to Origin SRS.
#
# Usage:
#	make publish [stream=(input/mic|<app>/<stream>)]

publish-stream = $(if $(call eq,$(stream),),input/trance,$(stream))

publish:
ifeq ($(OS_NAME),Darwin)
	ffmpeg -f avfoundation -video_device_index 0 -audio_device_index 0 -i '' \
	       -f flv rtmp://127.0.0.1:1935/$(publish-stream)
else
	$(error "'publish' command is not implemented for your OS")
endif


# Tune audio filters on-fly for mixed RTMP stream.
#
# Usage:
#	make tune volume=<volume-rate> [track=(music|original)]

tune-track = $(if $(call eq,$(track),),music,$(track))
tune-volume-port = $(if $(call eq,$(tune-track),music),60002,60001)

tune:
ifneq ($(volume),)
	docker run --rm --network=host --entrypoint sh \
		$(IMAGE_NAME):$(IMAGE_TAG) -c \
			'echo "volume@$(tune-track) volume $(volume)" \
			 | zmqsend -b tcp://127.0.0.1:$(tune-volume-port)'
endif




###################
# Docker commands #
###################

# Stop project in Docker Compose development environment
# and remove all related containers.
#
# Usage:
#	make docker.down [app=(mix|vod)]

docker.down:
	docker-compose down --rmi=local -v


# Run project in Docker Compose development environment.
#
# Usage:
#	make docker.up [background=(no|yes)]

docker.up: docker.down
	docker-compose up \
		$(if $(call eq,$(background),yes),-d,--abort-on-container-exit)




##################
# .PHONY section #
##################

.PHONY: down up \
        play publish tune \
        devices.list \
        docker.down docker.up
