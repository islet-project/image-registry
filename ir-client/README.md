# Introduction

This is an Image Registry client implementation compatible with an Image
Registry server implementing [OCI
Distribution Specification](https://github.com/opencontainers/distribution-spec)
([pull workflow
only](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pull))
and optionally
[Ra-TLS](https://github.com/islet-project/remote-attestation/tree/main/lib/ratls/ratls)
protocol.

# Purpose

The main  purpose of this library is to provide a high abstraction
client API to fetch an application image from OCI Registry server and
unpack it to a given directory. This library also provides means to
allow client verification using Ra-TLS protocol.

This crate also provides a lower level OCI Registry client, which
implements part of
[OCI Distribution](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#endpoints)
client API. This allows to separately fetch manifest and blob files
and list tags from the registry. This is not the main purpose of this
library, thus it will not be described here.


# Detailed description

The main client provides the following API:

## Client initialization

```rust
    pub fn from_config(config: Config) -> Result<Self, Error>
```

Initializes client from given configuration. This allows for client to
choose protocol for the connection (unsecured, tls or ra-tls), pass
host address and other tls related settings.

## Image info fetching

```rust
    pub async fn get_image_info(
        &self,
        app_name: &str,
        reference: Reference,
    ) -> Result<ImageInfo, Error>
```

This API was created to pre-fetch the `app_name` application
[`reference`](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests)
image info before downloading the image files. `ImageInfo` structure
provides user with
[`OciConfig`](https://github.com/opencontainers/image-spec/blob/main/config.md)
structure, which allows to recognize if updated version is available,
and thus if a redownload of the image files is required.

`ImageInfo` also contains a set of
[annotations](https://github.com/opencontainers/image-spec/blob/main/annotations.md),
which can contain custom information related to the image (e.g. image
signature, version, license, etc).

## Image files download

```rust
    pub async fn unpack_image(
        &self,
        image_info: &ImageInfo,
        dest: impl AsRef<Path>,
        temp: impl AsRef<Path>,
    ) -> Result<(), Error>
```

Fetches all image files (manifest, config and layers) and unpacks it
to `dest` directory following the [image layer filesystem
changeset](https://github.com/opencontainers/image-spec/blob/main/layer.md)
rules. This also verifies digest and
[`diff_id`](https://github.com/opencontainers/image-spec/blob/main/config.md#layer-diffid)
of every layer file.

# Usage

## Client API example

This is a client API usage example with ra-tls connection protocol
(please note that error and async handling is omitted):

```rust
    // Setup a root cert store for server verification.
    let root_cert_store: rustls::RootCertStore =
        load_root_cert_store();
    // Create token resolver for client certificate (impl ratls::InternalTokenResolver)
    let token_resolver = Arc::new(get_token_resolver());
    // Create ra-tls certificate resolver (impl rustls::ResolvesClientCert)
    let cert_resolver = Arc::new(RaTlsCertResolver::from_token_resolver(token_resolver));

    // Create client configuration
    let config = Config::builder()
        .host(conn.host)
        // Use ra-tls protocol for connection with server
        .ratls(root_cert_store, cert_resolver);

    let client = Client::from_config(config);

    // Choose image reference
    let reference = Reference::try_from("stable");

    // Fetch image info
    let image_info = client.get_image_info(&"com.samsung.example.app", reference);
    client.unpack_image(&image_info, "application_image_dir", "/tmp/");
```

## Utility binaries

This crate provides a set of utility binaries, which can fetch
separate OCI Image files (manifests and blobs), list server tags
or download the entire application image.

### `client` and `client_async`

Both binaries serve the same purpose, `client` is using the synchronous
[`oci::blocking::client`](https://github.com/islet-project/image-registry/blob/abf7de7b1b85a627820de40d83d02311ea35e605/ir-client/src/oci/blocking/client.rs)
API and `client_async` uses the asynchronous version
[`oci::client`](https://github.com/islet-project/image-registry/blob/abf7de7b1b85a627820de40d83d02311ea35e605/ir-client/src/oci/client.rs)

Both binaries support multiple commands:

```
Usage: client_async [OPTIONS] <COMMAND>

Commands:
  get-manifest  Print manifest of application image
  get-blob      Get blob file of application image
  list-tags     List tags of application images
  help          Print this message or the help of the given subcommand(s)

```

Every command requires the following options:

```
Options:
  -a, --host <HOST>            Host url address [default: localhost:1337]
  -t, --tls <mode>             Connection type [default: no-tls] [possible values: no-tls, tls, ra-tls]
      --token <token.bin>      CCA token file in binary CBOR format (used with ra-tls) [default: ./res/token.bin]
      --root-ca <root-ca.crt>  Root certificate file in PEM format (used with tls and ra-tls) [default: ./res/root-ca.crt]
```

Apart from them, each command has its separate options.

#### `get-manifest` command

Fetches `<reference>` manifest of the `<app-name>` application image.

```
Usage: client get-manifest --app-name <APP_NAME> --reference <REFERENCE>

Options:
  -a, --app-name <APP_NAME>    Repository namespace (application name)
  -r, --reference <REFERENCE>  Reference of manifest [digest or tag]
```

#### `get-blob` command

Fetches `<digest>` blob file (which can be both a config or a layer) from
the `<app-name>` application image.

```
Usage: client get-blob [OPTIONS] --app-name <APP_NAME> --digest <DIGEST>

Options:
  -a, --app-name <APP_NAME>  Repository namespace (application name)
  -d, --digest <DIGEST>      Digest of a blob
  -o, --out <OUT>            write blob to path
```

#### `list-tags` command

Fetches list of tags of the `<app-name>` application image(s).

```
Usage: client list-tags [OPTIONS] --app-name <APP_NAME>

Options:
  -a, --app-name <APP_NAME>  Repository namespace (application name)
  -n, --n <N>                List only N tags
  -l, --last <LAST>          Start listing tags after LAST
```

### `get-image`

This binary allows to fetch entire `<app-name>` application
`<reference>` image and save it to a `<dest>` directory.

```
Usage: get_image [OPTIONS] --app-name <APP_NAME> --reference <REFERENCE> --dest <DEST>

Options:
  -a, --host <HOST>            Host url address [default: localhost:1337]
  -t, --tls <mode>             Connection type [default: no-tls] [possible values: no-tls, tls, ra-tls]
      --token <token.bin>      CCA token file in binary CBOR format (used with ra-tls) [default: ./res/token.bin]
      --root-ca <root-ca.crt>  Root certificate file in PEM format (used with tls and ra-tls) [default: ./res/root-ca.crt]
  -n, --app-name <APP_NAME>    Repository namespace (application name)
  -r, --reference <REFERENCE>  Reference of image manifest [digest or tag]
  -d, --dest <DEST>            directory to unpack the image
```
