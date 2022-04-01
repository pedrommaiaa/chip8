run:
	cd web && \
	python3 -m http.server --bind 127.0.0.1

install:
	cd wasm && \
	wasm-pack build --target web && \
	mv pkg/wasm_bg.wasm ../web && \
	mv pkg/wasm.js ../web

clean:
	rm -f web/wasm_bg.wasm && \
	rm -f web/wasm.js && \
	cd wasm && \
	cargo clean
