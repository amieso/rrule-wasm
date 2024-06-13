install-wasm-pack:
	curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

build-wasm-nodejs:
	wasm-pack build --release --target nodejs --out-dir pkg/nodejs --features "wasm"

test-wasm-on-nodejs:
	node examples/wasm/nodejs/app.js

build-wasm-web:
	wasm-pack build --release --target web --out-dir pkg/web --features "wasm"

test-wasm-on-web-browser:
	npx http-server -o /examples/wasm/web/index.html

build-wasm-bundler:
	wasm-pack build --release --target bundler --features "wasm"

pack:
	wasm-pack pack pkg