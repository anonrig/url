## URL in WebAssembly and Rust

This is a small experiment to replace URL and URLSearchParams using WebAssembly and Rust. For all missing functionality and the to-do list of this package follow [this link.](https://github.com/anonrig/url/issues/1)

### Goals

- Increase the performance of URL and URLSearchParams
- Replace internal implementation of Node.js for URL and URLSearchParams

### Installation

Easiest way to test this package is to install it from NPM. Currently, every change made to the main branch is released under the alpha tag.

```bash
npm i --save url-wasm@alpha
```

### Benchmarks

Referencing `url-wasm-0.1.1-ci-1883989259.0` release

```
====================
  URL vs. Rust URL
====================

Platform info:
==============
   Darwin 21.2.0 arm64
   Node.JS: 17.5.0
   V8: 9.6.180.15-node.13
   CPU: Apple M1 × 8
   Memory: 16 GB

Suite: URL
✔ URL               1,073,591 rps
✔ Rust::URL           819,374 rps

   URL                  0%      (1,073,591 rps)   (avg: 931ns)
   Rust::URL       -23.68%        (819,374 rps)   (avg: 1μs)
-----------------------------------------------------------------------

Suite: URLSearchParams.set
✔ URLSearchParams.set                  29,112 rps
✔ Rust::URLSearchParams.set             3,902 rps

   URLSearchParams.set                  0%         (29,112 rps)   (avg: 34μs)
   Rust::URLSearchParams.set        -86.6%          (3,902 rps)   (avg: 256μs)
-----------------------------------------------------------------------

Suite: URLSearchParams.append
✔ URLSearchParams.append                  94,838 rps
✔ Rust::URLSearchParams.append             3,792 rps

   URLSearchParams.append                  0%         (94,838 rps)   (avg: 10μs)
   Rust::URLSearchParams.append          -96%          (3,792 rps)   (avg: 263μs)
-----------------------------------------------------------------------
```
