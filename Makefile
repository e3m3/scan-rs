.PHONY: all container format format-check native clean clean-native

PWD=$(shell pwd)
CONTAINER_BIN=podman
CONTAINER_FILE=${PWD}/containers/Containerfile
CONTAINER_NAME=scan-rs
CARGO_TOML=${PWD}/Cargo.toml

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
	cargo fmt --all --manifest-path ${PWD}/driver/Cargo.toml
	cargo fmt --all --manifest-path ${PWD}/test-scan/Cargo.toml
	for d in ${PWD}/impls/*; do cargo fmt --all --manifest-path $$d/Cargo.toml; done
	for d in ${PWD}/deps/*; do cargo fmt --all --manifest-path $$d/Cargo.toml; done

format-check:
	cargo fmt --all --check --manifest-path ${CARGO_TOML}
	cargo fmt --all --check --manifest-path ${PWD}/driver/Cargo.toml
	cargo fmt --all --check --manifest-path ${PWD}/test-scan/Cargo.toml
	for d in ${PWD}/impls/*; do cargo fmt --all --check --manifest-path $$d/Cargo.toml; done
	for d in ${PWD}/deps/*; do cargo fmt --all --check --manifest-path $$d/Cargo.toml; done
