# Celenium WASM SDK

An integration wrapper around [`celenium-wasm`](https://www.npmjs.com/package/celenium-wasm) module that uses [`websocket`](https://www.npmjs.com/package/websocket) W3C adaptor for WebSocket communication.

This is a Node.js module that provides bindings to the Celenium WASM SDK strictly for use in the Node.js environment. The web browser version of the SDK is available as part of official SDK releases at [https://github.com/dimas405/celenium/releases](https://github.com/dimas405/celenium/releases)

## Usage

Celenium NPM module exports include all WASM32 bindings.
```javascript
const celenium = require('celenium');
console.log(celenium.version());
```

## Documentation

TypeScript documentation for `celenium-wasm` is available at [https://docs.rs/celenium-wasm](https://docs.rs/celenium-wasm)


## Building from source & Examples

SDK examples as well as information on building the project from source can be found at [https://github.com/dimas405/celenium/tree/master/wasm](https://github.com/dimas405/celenium/tree/master/wasm)

## Releases

Official releases as well as releases for Web Browsers are available at [https://github.com/dimas405/celenium/releases](https://github.com/dimas405/celenium/releases).

Nightly / developer builds: build from source (see "Building from source & Examples") or check the project releases.

