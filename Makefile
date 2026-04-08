.PHONY: build install uninstall clean dev

PLUGIN_DIR := $(HOME)/.config/opendeck/plugins/dev.penguin.kick.sdPlugin

build:
	. $$HOME/.cargo/env && cargo build --release

install: build
	mkdir -p $(PLUGIN_DIR)
	cp target/release/kick-opendeck $(PLUGIN_DIR)/kick-opendeck-x86_64-unknown-linux-gnu
	cp -r plugin/* $(PLUGIN_DIR)/

uninstall:
	rm -rf $(PLUGIN_DIR)

clean:
	cargo clean

dev:
	. $$HOME/.cargo/env && cargo build
	mkdir -p $(PLUGIN_DIR)
	cp target/debug/kick-opendeck $(PLUGIN_DIR)/kick-opendeck-x86_64-unknown-linux-gnu
	cp -r plugin/* $(PLUGIN_DIR)/
