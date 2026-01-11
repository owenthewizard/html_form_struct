# 📝 html_form_struct

Generate a struct from an HTML form.

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

#[form_struct("index.html", "form#register")]
// important: derive must come after form_struct
#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Registration;
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
