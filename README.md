# Websocket broadcast demo

This demo serves as a websocket server that supports high amounts of concurrent connections and allows clients broadcast messages to every other client connected.

![GIF demo](/demo/recording.gif?raw=true "Websocket connection")

## Try it out!
Install [Rust](https://rustup.rs/) and run:
```
$ cargo run
```

Then use your favorite Websocket client or use [websocat](https://github.com/vi/websocat):
```
$ cargo install websocat

$ websocat ws://127.0.0.1:9090
```
Connect with multiple clients to the same server to test message exchange