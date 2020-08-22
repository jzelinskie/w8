# w8

So, you're writing some bash and you realize that before you can execute the next program, you need to wait until a port or HTTP endpoint becomes available. That's where _w8_ comes in.

## Usage 
```sh
# Waiting for a healthcheck to be up before using an API
$ w8 --http localhost:8080/v1/_healthz && curl localhost:8080/v1/data

# Wait for Postgres to be running before running a script for CI
$ w8 --tcp localhost:5432 && psql -c 'CREATE EXTENSION IF NOT EXISTS pg_trgm;'

# Mix and match! Is Deluge daemon, web UI, and the BitTorrent port ready?
$ w8 --tcp 192.168.1.100:58846 --http 192.168.1.100:8112 --tcp 192.168.1.100:8998
```

## Installation

```sh
$ cargo install w8
```