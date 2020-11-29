build:
	wasm-pack build --target nodejs
	cp package.json pkg/package.json
	cp index.js pkg/index.js
	cp generate_config pkg/generate_config
