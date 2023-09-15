# CC Bulletin Board

A lightweight short-term HTTP data store

## Philosophy

This started as a project to implement a server to communicate between the
Minecraft mod CC: Tweaked — which is only capable of limited HTTP communication
for security reasons — and a web browser. I ended up adding configuration
options in case someone else might find use out of it.

The server acts as a "bulletin board" where one client can `PUT` up a posting,
and another client can then `GET` the same data. Data is only persisted in
memory.

## Configuration

The server is configured with a TOML file whose path is provided with the
`--config` argument. See `doc` directory for an example config and systemd unit.

* `bind_address` - Sets the address and port to which the server is bound, e.g.
  `"127.0.0.1:3000"`
* `password` - Sets the `Authorization: Bearer` token that clients must use

The `RUST_LOG` environment variable can be used as described in `env_logger`'s
documentation, for instance `error` to log errors, `info` to log info and
`trace` to log everything.

## HTTP API

All requests require the `Authorization: Bearer <password>` header to be present
and correct.

### `PUT /{id}`

Creates/updates a post at `id`.

### `GET /{id}`

Gets the post at `id`. Returns a 404 if the post does not exist.
