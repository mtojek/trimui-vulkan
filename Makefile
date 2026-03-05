.PHONY: shell clean build

IMAGE_NAME := trimui-smart-pro-toolchain
WORKSPACE_DIR := $(shell pwd)

build: Dockerfile
	docker build -t $(IMAGE_NAME) .

shell: build
	docker run -it --rm -v "$(WORKSPACE_DIR)":/workspace $(IMAGE_NAME) bash

clean:
	docker rmi $(IMAGE_NAME)
