# Thrift Pool Tutorial

This repo showcases the usage of [thrift-pool](https://github.com/midnightexigent/thrift-pool-rs) by using its capabilites to implement a Connection Pool for the client in the [thrift tutorial](https://github.com/apache/thrift/tree/master/tutorial)

- [lib.rs](src/lib.rs) to see how to easily implement a Connection Pool for the `CalculatorSync` client
- [main.rs](src/main.rs) to see how it's used with bb8 and r2d2

To test it out: 

- launch the server 
- Run `cargo run`

