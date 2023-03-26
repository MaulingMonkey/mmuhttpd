# mmuhttpd

| 🦀    | Meanings |
| ------| ---------|
| MM    | [MaulingMonkey](https://github.com/MaulingMonkey)'s
| U     | [Micro (µ)](https://en.wikipedia.org/wiki/Micro-)
| HTTP  | [HyperText Transfer Protocol](https://en.wikipedia.org/wiki/HTTP)
| D     | [Daemon](https://en.wikipedia.org/wiki/Daemon_(computing))

## Raison d'être

*   0 dependencies
*   100% monkey
*   don't actually use this

## Usage

```sh
cargo install --git https://github.com/MaulingMonkey/mmuhttpd
mmuhttpd                        # use CWD as your webroot
mmuhttpd --open some/other/dir  # use another dir as your webroot + open your browser
mmuhttpd --allow-all-ipv4       # allow non-localhost traffic (bind to any/all IPv4 addresses)
mmuhttpd --allow-all-ipv6       # allow non-localhost traffic (bind to any/all IPv6 addresses)
```



<h2 name="license">License</h2>

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.



<h2 name="contribution">Contribution</h2>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
