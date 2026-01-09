# 📝 html_form_struct

Generate a struct from an HTML form.

[![Rust](https://img.shields.io/badge/Language-Rust-blue?style=for-the-badge)](https://www.rust-lang.org)
[![Build Status](https://github.com/owenthewizard/html_form_struct/actions/workflows/ci.yml/badge.svg?style=for-the-badge)](https://github.com/owenthewizard/html_form_struct/actions/workflows/rust_ci.yml)
[![Crates.io](https://img.shields.io/crates/v/html_form_struct?style=for-the-badge&color=fc8d62)](https://crates.io/crates/html_form_struct)
[![Documentation](https://img.shields.io/docsrs/html_form_struct/latest.svg?style=for-the-badge&color=66c2a5)](https://docs.rs/html_form_struct/latest/html_form_struct/)
[![Crate Downloads](https://img.shields.io/crates/dv/html_form_struct.svg?style=for-the-badge)](https://crates.io/crates/html_form_struct)
[![License (MIT)](https://img.shields.io/badge/License-MIT-f46623?style=for-the-badge)](LICENSE-MIT.md)
[![License (Apache 2.0)](https://img.shields.io/badge/License-Apache-f46623?style=for-the-badge)](LICENSE-APACHE.md)
[![Contributors](https://img.shields.io/github/contributors/owenthewizard/html_form_struct?style=for-the-badge&color=8da0cb)](https://github.com/owenthewizard/html_form_struct/graphs/contributors)
[![GitHub forks](https://img.shields.io/github/forks/owenthewizard/html_form_struct?style=for-the-badge&color=8da0cb)](https://github.com/owenthewizard/html_form_struct/network/members)
[![Stars](https://img.shields.io/github/stars/owenthewizard/html_form_struct?style=for-the-badge&color=8da0cb)](https://github.com/owenthewizard/html_form_struct/stargazers)
[![Issues](https://img.shields.io/github/issues/owenthewizard/html_form_struct?style=for-the-badge)](https://github.com/owenthewizard/html_form_struct/issues)
[![Funding](https://img.shields.io/badge/Funding-Sponsor%20me%21-yellow?style=for-the-badge)](https://github.com/sponsors/owenthewizard)

## ▶️ Quick Start

```html
<form id="register">
  <input type="text" name="username" required />
  <input type="password" name="password" required />
  <input type="email" name="email" />
  <!--
      form_struct: type=u32
  -->
  <input type="number" name="guess" />
</form>
```

```rust
use html_form_struct::form_struct;

form_struct!("index.html", "form#register", Registration);
```

Output:

```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Registration {
    pub email: Option<String>,
    pub guess: Option<u32>,
    pub password: String,
    pub username: String,
}
```

<!-- TODO: add more direct link -->

A production example can be found in [esp32-wifi-bridge](https://github.com/owenthewizard/esp32-wifi-bridge).

## ℹ️ Note

This project was created for [esp32-wifi-bridge](https://github.com/owenthewizard/esp32-wifi-bridge), but may be useful for others.

## 👷 Code Style

Obey `rustfmt`, Rust 2024, and `clippy`.

## 🤝 Cotributions

Pull requests are always welcome.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed under the terms of both the MIT License and the Apache License (Version 2.0).

## 🔢 Version Scheme

- This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
- This project uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
- Changes are documented in the [Changelog](CHANGELOG.md).

## 👪 Authors

See [the list of contributors](https://github.com/owenthewizard/i3lockr/contributors).

## ⚖️ License

See [LICENSE-APACHE](LICENSE-APACHE.md) and [LICENSE-MIT](LICENSE-MIT.md) for details.
