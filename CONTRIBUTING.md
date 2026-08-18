# How to Contribute

Contributions are always welcome! There is a multitude of ways in which you can
help depending on what you like to do, or are good at. Documentation, code,
issues, new features are all ways of contributing and greatly appreciated!

## Install Rust

If you want to contribute code then `Rust` (stable) should be installed along
with `rustfmt` and `clippy`.

`Rust` can be downloaded from here: [https://rust-lang.org/](https://rust-lang.org/)

`rustfmt` and `clippy` can be installed with `rustup` like so:

```sh
rustup component add clippy
rustup component add rustfmt
```

## Basic workflow

- Fork this repository from `main` branch
- `git clone <your_forked_repo>`
- `git checkout -b my_fixes`
- Make your changes
- Commit your changes. Please use [Conventional
  Commits](https://www.conventionalcommits.org/) specification for your commit
  messages.

If your contribution is code related, make sure to run these commands before
pushing:

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --all-targets`
- `cargo test --all-targets`

Fix any warnings or errors and then:

- `git push` your changes
- Create a pull request
- Await the approval from project owner/maintainers. Discuss possible changes
  and update your pull request if necessary.
- If all goes well the changes will be merged

# License

By contributing, you agree that your contributions will be licensed under [Zlib
License](./LICENSE).
