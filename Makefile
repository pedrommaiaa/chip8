run:
	cd web && \
	python3 -m http.server

install:
	cd wasm && \
	cargo build
