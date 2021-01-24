###############################
# Common defaults/definitions #
###############################

# Checks two given strings for equality.
eq = $(if $(or $(1),$(2)),$(and $(findstring $(1),$(2)),\
                                $(findstring $(2),$(1))),1)




###########
# Aliases #
###########

down: docker.down


up: docker.up




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
        docker.down docker.up
