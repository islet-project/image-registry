# Introduction

This is a repository for Image Registry application and surrounding tools and
libraries. Currently it consists of the following components:

## IR Server

Standalone application serving OCI compatible images over the network. See the
detailed readme [here](ir-server/).

## IR Client

A library and a set of examples for implementing OCI compatible client
applications. See the detailed readme [here](ir-client/).

## IR Signing tool

A tool for signing OCI compatible images (using annotations) for use with
[application
provisioning](https://github.com/islet-project/islet/tree/app-provisioning/examples/app-provisioning). See
the detailed readme [here](ir-sign/).

## Registry

Very small example registry for the server to have something to run on by
default. It also contains a simple docker built AARCH64 "hello world"
application. See the detailed readme [here](registry/).

# Simple test

To perform the simplest possible test of client/server communication with ratls
on one machine (using localhost) one can do:

## Server

```
cd ir-server
cargo run --features disable-challenge-veraison -- -t ra-tls
```

## Client

```
cd ir-client
cargo run --bin client -- -t ra-tls get-manifest -a com.samsung.example.app -r stable
```
