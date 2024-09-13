# Introduction

This is a tool to sign images served by the Image Registry server (`ir-server`).

It doesn't use X.509 certificates yet but for now for simplicity's sake it
creates a simple ECDSA (secp384r1) based chain of trust.

Everything it generates is kept in various DER based formats (SEC1, SPKI, etc)
and has been verified to be binary compatible with `openssl` cmd line tool.

# Workflow

Below is written a simple workflow to sign the images. Some basic commands
related to keys have two variants (`ir-sign` and `openssl`) and it doesn't matter
which is used, they are interechangable and binary compatible.

Sections are marked with "CA:" and "Vendor:" to indicate which parties should be
doing what.

## Keygen

- CA: Generate ROOT-CA (root certificate authority) private key:

```
openssl ecparam -name secp384r1 -genkey -noout -out root-ca.prv -outform DER
```

```
cargo run -- gen-key -o root-ca.prv
```

- CA: Extract public portion of ROOT-CA key:

```
openssl ec -in root-ca.prv -inform DER -pubout -out root-ca.pub -outform DER
```

```
cargo run -- extract-public -i root-ca.prv -o root-ca.pub
```

This public key (`root-ca.pub`) should be sent to Vendor.

- Vendor: Generate Vendor private key

```
openssl ecparam -name secp384r1 -genkey -noout -out vendor.prv -outform DER
```

```
cargo run -- gen-key -o vendor.prv
```

## Image signing

Two separate workflows are possible here. One realistic mimicking how that would
really look like in a real world scenario and simplified for quicker testing.

### Realistic scenario

- Vendor: Extract Vendor public key

```
openssl ec -in vendor.prv -inform DER -pubout -out vendor.pub -outform DER
```

```
cargo run -- extract-public -i vendor.prv -o vendor.pub
```

This public key (`vendor.pub`) should be sent to ROOT-CA.

- CA: Sign the Vendor public key with ROOT-CA private key

```
openssl dgst -sha384 -sign root-ca.prv -keyform DER -out vendor.pub.sig vendor.pub
```

```
cargo run -- sign -k root-ca.prv -f vendor.pub -s vendor.pub.sig
```

This signature (`vendor.pub.sig`) should be sent to Vendor.

- Vendor: Sign the image with all the collected data

```
cargo run -- sign-image -r REGISTRY_DIR -a APP_NAME -d MANIFEST_DIGEST -v vendor.prv -s vendor.pub.sig -c root-ca.pub
```

The tool will verify the given signature for Vendor public key first to make
sure it's correct. Only then it will sign the config for a given manifest and
update the manifest with annotations containing the following data:

  * signature of the config file signed with Vendor key
  * Vendor public key
  * signature of the Vendor public key signed with ROOT-CA key

The latter two entries comprise something akin to certificate.

The image is then rehashed so the hashes are correct (only given manifest and
layout index for now).

### Simplified scenario

- CA/Vendor: Sign the Vendor public key and then sign the image

```
cargo run -- sign-image -r REGISTRY_DIR -a APP_NAME -d MANIFEST_DIGEST -v vendor.prv -x root-ca.prv
```

This is an unrealistic scenario as it assumes access to ROOT-CA private key
(hence designation CA/Vendor). It simply allows for faster testing/signing
locally. This scenario instead of verifying the signature (as in realistic
scenario) creates one (signs the Vendor public key with given ROOT-CA private
key). The output of the command is the same as in the realistic scenario. The
given manifest will contain all the same annotations.

## Image extract and signing directly from OCI image archive

Images can be extracted and then signed directly from OCI image archives
(e.g. produced by docker export/save commands).

A command to do this is `extract-sign-image`. It's very similar to
`sign-image`. The only difference is that it will first extract the TAR archive
into registry (only uncompressed TAR files are supported now).

```
cargo run -- extract-sign-image -r REGISTRY_DIR -f OCI_IMAGE_TAR -a OPTIONAL_APP_NAME -d MANIFEST_DIGEST -v vendor.prv [OTHER_CRYPTO_PARAMS]
```

`OPTIONAL_APP_NAME` is an app name that will be created in the registry. If
ommited a filename will be used (e.g. for `nginx.tar` an app name `nginx` will
be used). The rest is the same as for `sign-image` with `OTHER_CRYPTO_PARAMS`
dependent on realistic or simplified scenario.

## Image verification

A signed image can be verified with a simple command that only requires ROOT-CA
public key. All the other data are taken from the manifest annotations and from
files themselves.

```
cargo run -- verify-image -r REGISTRY_DIR -a APP_NAME -d MANIFEST_DIGEST -c root-ca.pub
```

This will verify the following:

  * Hash of the config file (saved in manifest vs real file)
  * Vendor public key signature (from annotations using `root-ca.pub`)
  * Config signature (from annotations using Vendor public key)
