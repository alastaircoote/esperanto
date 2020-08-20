# Esperanto

#### A "universal" JavaScript worker runtime

Inspired by [Service Workers](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API), Esperanto is intended to be a portable JavaScript worker runtime for execution inside iOS and Android apps (and potentially other places too).

It is designed for flexibility wherever it connects: you can plug in different JavaScript engines (currently [JavaScriptCore](https://developer.apple.com/documentation/javascriptcore) and [QuickJS](https://bellard.org/quickjs/)) and use it as both a C API (with an accompanying Swift package) and through a Java JNI bridge.

## Current status

Very little is done so there isn't a whole lot to look at yet. But very basic bindings to both JS engines and both output APIs are working, though not in their final form.

## Packages into this repo

- [esperanto-javascriptcore](https://github.com/alastaircoote/esperanto/tree/master/esperanto-javascriptcore): bindings to run Esperanto with JavaScriptCore. Primarily intended for iOS, where JSC is bundled with the OS.
- [esperanto-quickjs](https://github.com/alastaircoote/esperanto/tree/master/esperanto-quickjs): bindings for the QuickJS JavaScript engine. Primarily intended for Android, but can be used anywhere since the runtime is built with the library.
- [esperanto-ffi](https://github.com/alastaircoote/esperanto/tree/master/esperanto-ffi): The exported C API for Esperanto. Is primarily used by...
- [esperanto-ios](https://github.com/alastaircoote/esperanto/tree/master/esperanto-ios): An XCode project that wraps the C API into a more usable Swift one.
- [esperanto-jni](https://github.com/alastaircoote/esperanto/tree/master/esperanto-ffi): The JNI interface to bind the library to Java. Primarily used by...
- [esperanto-android](https://github.com/alastaircoote/esperanto/tree/master/esperanto-android): An Android Studio project to create an AAR file that bundles the library, its interface and builds for all ABIs in one.
- [esperanto-shared](https://github.com/alastaircoote/esperanto/tree/master/esperanto-shared): traits and utilities used by the other packages.
- [~~esperanto-core~~](https://github.com/alastaircoote/esperanto/tree/master/esperanto-core): an early thought that didn't go anywere. Will be deleted once I've pulled out the stuff I want from it.

## How do I use it?

Right now the easiest way to actually test the thing out is to use the [iOS](https://github.com/alastaircoote/esperanto-example-ios) and [Android](https://github.com/alastaircoote/esperanto-example-android) example apps. You can also download this repo and, if you have Rust installed, run `cargo test` to see it run some tests (that may or may not currently pass).

## What still needs to be done?

A lot! But when this is done my hope is that it will have:

- A [Service Worker-like lifecycle](https://developers.google.com/web/fundamentals/primers/service-workers/lifecycle), allowing you to safely update with fallbacks
- An equivalent to the [Cache API](https://developer.mozilla.org/en-US/docs/Web/API/Cache)
- The ability to dispatch `respondWith` events (e.g. [Fetch event](https://developer.mozilla.org/en-US/docs/Web/API/FetchEvent)) to return data from the worker back to the native context. Hopefully complete with streaming.
- An IndexedDB API (I _guess_, maybe start with something simpler than that...)
- An OffscreenCanvas API

I'm also interested to explore using Esperanto in a web server to bootstrap service workers before they are installed on client devices. But we're a long way from that right now.

---

<sub><sup>[what is Esperanto?](https://en.wikipedia.org/wiki/Esperanto)</sup></sub>
