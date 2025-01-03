# Introduction

This is a Image Registry server implementing [OCI Image Format
Specification](https://github.com/opencontainers/image-spec) and [OCI
Distribution Specification](https://github.com/opencontainers/distribution-spec)
([pull workflow
only](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pull)).

# Purpose

The purpose of the server is to load, parse and verify a set of OCI images held
in a registry and upon successful verification to serve them over the network
using OCI distribution pull specification.

# Detailed description

The server upon starting looks for a directory containing the registry
([example](../registry)). The registry is in the format of:

```
registry/application1/
registry/application2/
registry/application3/
```

Each application should conform to the [OCI Image Layout
Specification](https://github.com/opencontainers/image-spec/blob/main/image-layout.md).

The server looks through the `registry` directory and tries to load each
application. It checks several things related to OCI compatibility and
consistency (if the content conforms with the OCI specification). It also checks
if all the indexed files exist, have proper types, file sizes and in some cases
hashes.

Only the applications that load without errors are served. Only files that are
linked through indexes and manifests are served. Files that are not indexed
(orphaned files) are ignored and logged.

When the registry is loaded the applications starts to serve those images
through HTTP OCI distribution protocol. The HTTP can be used directly
(unencrypted), using TLS or using RaTLS.

Each application is served under its own OCI namespace that is simply the name
of the application.

The examples for OCI distribution API that can be tested using any HTTP client
(a browser for example):

```
http://SERVER_NAME:PORT/v2/
http://SERVER_NAME:PORT/v2/application1/tags/list
http://SERVER_NAME:PORT/v2/application2/manifests/latest
http://SERVER_NAME:PORT/v2/application3/blobs/DIGEST
```

For TLS replace `http` with `https`. For RaTLS only RaTLS [client](../ir-client)
can be used.

# Usage

To run the server with default options it's enough to do:

```
image-registry/ir-server $ cargo run
```

The application accepts several command line switches:

```
image-registry/ir-server $ cargo run -- --help
Usage: ir-server [OPTIONS]

Options:
  -r, --root <ROOT>
          runtime server root directory, none to use {crate_root}/registry
  -c, --cert <CERT>
          path to server certificate, none to use {crate_root}/certs/server.crt
  -k, --key <KEY>
          path to server private key, none to use {crate_root}/certs/server.key
  -t, --tls <TLS>
          TLS variant to use [default: no-tls] [possible values: no-tls, tls, ra-tls]
  -p, --port <PORT>
          server port [default: 1337]
  -u, --veraison-url <VERAISON_URL>
          RA-TLS: Veraison verification service host [default: http://localhost:8080]
  -v, --veraison-pubkey <VERAISON_PUBKEY>
          RA-TLS: Veraisons public key, none to use {crate_root}/ratls/pkey.jwk
  -j, --reference-json <REFERENCE_JSON>
          RA-TLS: JSON containing reference values, none to use {crate_root}/ratls/example.json
  -h, --help
          Print help
  -V, --version
          Print version
```

The switches for Veraison and reference JSON are used for RaTLS and passed to
the [RaTLS library](https://github.com/islet-project/ratls) and its verifiers
([Veraison](https://github.com/islet-project/veraison-verifier),
[Realm](https://github.com/islet-project/realm-verifier)).
