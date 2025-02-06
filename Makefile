.PHONY: all container format format-check native clean clean-native

CONTAINER_BIN=podman
CONTAINER_FILE=containers/Containerfile
CONTAINER_NAME=scan-rs

CARGO_TOML=Cargo.toml

all: container

container: 
	${CONTAINER_BIN} build -t ${CONTAINER_NAME} -f ${CONTAINER_FILE} .

clean:
	${CONTAINER_BIN} image rm ${CONTAINER_NAME}

native:
	cargo build

clean-native:
	cargo clean

format:
	cargo fmt --all --manifest-path ${CARGO_TOML}

format-check:
	cargo fmt --all --check --manifest-path ${CARGO_TOML}
