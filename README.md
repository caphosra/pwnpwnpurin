# purin

Automatically build `libc.so.6` and its corresponding `ld.so` from source.

This tool is mainly designed to patch an executable when glibc on a server is older or newer than you have. It is useful when `libc.so.6` is not distributed on a CTF pwn challenge or when you want to test an exploiting code relying on a vulnerability of specific glibc.

## Requirement

This tool depends on `Docker`. Please make sure it is installed.

## Installation

```
git clone https://github.com/caphosra/pwnpwnpurin.git
cd pwnpwnpurin
cargo install --path .
```

## Usage

Build specific `libc.so` and `ld.so` and place them into the current directory. (ex. `purin install 2.33`)
```
purin install [VERSION]
```

It can take a long time for the first time. Once installed, it would be much faster, thanks to caches.
