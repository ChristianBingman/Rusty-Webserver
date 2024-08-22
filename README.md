# Rust HTTP 1.0 Server

Provides an implementation of an http 1.0 file server. Essentially like python3's httpServer module.

## Features
- [x] Handles GET and HEAD requests
- [x] Handle encodings other than UTF-8
- [x] ThreadPool implementation
  - Allows you to specify the amount of threads that can be used to accept requests, then uses a queue to handle requests as they are received
- [x] Command line args with clap
- [x] Show directory listings
- [ ] Logging
- [ ] Implement basic authentication
- [ ] Handle passing of multiple of the same header type
- [ ] Handle last-modified, if-modified-since, and 304
- [ ] Handle return codes better
  - [ ] Implement a server error and 404 page
- [-] Testing
  - [x] ThreadPool implementation
  - [ ] Request parsing
  - [ ] Responses
- [ ] Handle content codings

The server accepts HTTP 1.1 requests as well, but it will only ever respond with HTTP 1.0, and doesn't support HTTP 1.0 features such as

## What is this project for?

This is for me to get up to speed with Rust, and gain familiarity with popular tools. Also allows me to implement an RFC that allows me to learn about encoding, error handling, and testing.
