# Serde YAML (NU)

Rust library for using the [Serde] serialization framework with data in [YAML]
file format, with No Unsafe (NU) code.

This is a fork of [serde_yaml], using [libyaml-safer] so it has no unsafe code,
while maintaining the same API as `serde_yaml`.

[Serde]: https://github.com/serde-rs/serde
[YAML]: https://yaml.org/
[serde_yaml]: https://github.com/dtolnay/serde-yaml
[libyaml-safer]: https://github.com/simonask/libyaml-safer

## Dependency

This entry in your. `Cargo.toml` will migrate to `serde_yaml_nu` without needing to change your code.

```toml
[dependencies]
...
serde_yaml = { package = "serde_yaml_nu", version = "0.9.100" }
```

Release notes are available under [GitHub releases].

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
