generate-license-file:
	cargo install cargo-bundle-licenses
	cargo bundle-licenses --format json --output rust_licenses.json

dev-preinit:
	make generate-license-file
	cd frontend && yarn && yarn run build

linux-dev:
	# Generate the Rust license file.
	make generate-license-file

	# Ensure the frontend/dist directory exists.
	mkdir -p frontend/dist

	# Build core_embedded as a debug build.
	cd core_embedded && cargo build

	# Load foreman.
	foreman start -f Procfile.linux-dev

macos-dev:
	# Generate the Rust license file.
	make generate-license-file

	# Ensure the frontend/dist directory exists.
	mkdir -p frontend/dist

	# Build core_embedded as a debug build.
	cd core_embedded && cargo build

	# rm -rf MagicCap Dev.app in case it exists.
	rm -rf macos/MagicCap\ Dev.app

	# Copy the core_embedded binary into the app bundle.
	mkdir -p macos/MagicCap\ Dev.app/Contents/MacOS
	cp target/debug/core_embedded macos/MagicCap\ Dev.app/Contents/MacOS/MagicCap

	# Copy the Info.plist file into the app bundle.
	cp macos/Info.plist.tmpl macos/MagicCap\ Dev.app/Contents/Info.plist

	# Substitute {version} for 0.0.1 in development.
	sed -i '' 's/{version}/0.0.1/g' macos/MagicCap\ Dev.app/Contents/Info.plist

	# Load foreman.
	foreman start -f Procfile.macos-dev

build:
	node ./production_build.js

.DEFAULT_GOAL := build
.PHONY: generate-license-file dev-preinit linux-dev macos-dev build
