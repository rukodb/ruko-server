Ruko Server
====

This is a rewrite of [ruko-server_cpp](https://github.com/rukodb/ruko-server_cpp) in Rust. It is very much so still work in progress with only a small amount of boilerplate code.

The goal of this rewrite is to be as performant as possible (throughput and low latency), implementing arbitrary concurrency by using multiple worker threads sharded across keys.
