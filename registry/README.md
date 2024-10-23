# Introduction

This is an example registry used by [ir-server](../ir-server). It contains two
applications:

## com.samsung.example.app

This is a ready to be served example OCI layout. It contains few mostly empty
layers. There are no binaries, it's only used as a reference and an example.

## light_app

This is a working "hello world" application for AARCH64. It needs to be compiled
and packaged before it can be served/used. It's specifically made for the binary
to be as small as possible (around 1kB).

To compile do `make` inside the directory. It requires docker and cross AARCH64
compilator. It will compile the binary and package it as OCI layout using
docker.
