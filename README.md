# DBall

[![dependency status](https://deps.rs/repo/github/goodpeanuts/dball/status.svg)](https://deps.rs/repo/github/goodpeanuts/dball)
[![Build Status](https://github.com/goodpeanuts/dball/workflows/CI/badge.svg)](https://github.com/goodpeanuts/dball/actions?workflow=CI)

# Dev prepare

## Install tools

### [Cargo deny](https://embarkstudios.github.io/cargo-deny/index.html)

```bash
cargo install --locked cargo-deny --version 0.18.3 && cargo deny init && cargo deny check
```

**todo:**

If you already have cargo-deny installed, update it to the latest version so it can parse
CVSS 4.0 advisories from the RustSec database.

### [Pre-commit](https://pre-commit.com/#usage)

It is a multi-language package manager for pre-commit hooks. You specify a list of hooks you want and pre-commit manages the installation and execution of any hook written in any language before every commit. pre-commit is specifically designed to not require root access. If one of your developers doesn’t have node installed but modifies a JavaScript file, pre-commit automatically handles downloading and building node to run eslint without root.

```bash
brew install pre-commit
```

setup pre-commit hooks:

```bash
pre-commit install
```

### typos

Typos is a spell checker for source code.

```bash
cargo install --locked typos-cli --version 1.39.0
```

### [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

cargo-llvm-cov is a code coverage tool for Rust using LLVM.

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --all-targets --workspace --html
cargo llvm-cov --open
```

### Git cliff

Git cliff is used to generate changelogs from commit messages.

```bash
cargo install --locked git-cliff
```


### cargo nextest

Nextest is a boosted test runner for Rust.

```bash
cargo install --locked cargo-nextest --version 0.9.100
```


# eframe

This is a template repo for [eframe](https://github.com/emilk/egui/tree/master/crates/eframe), a framework for writing apps using [egui](https://github.com/emilk/egui/).

The goal is for this to be the simplest way to get started writing a GUI app in Rust.

You can compile your app natively.

## Getting started

Start by clicking "Use this template" at https://github.com/goodpeanuts/dball/ or follow [these instructions](https://docs.github.com/en/free-pro-team@latest/github/creating-cloning-and-archiving-repositories/creating-a-repository-from-a-template).

Change the name of the crate: Choose a good name for your project, and change the name to it in:
* `Cargo.toml`
    * Change the `package.name` from `eframe_template` to `your_crate`.
    * Change the `package.authors`
* `main.rs`
    * Change `eframe_template::TemplateApp` to `your_crate::TemplateApp`

Alternatively, you can run `fill_template.sh` which will ask for the needed names and email and perform the above patches for you. This is particularly useful if you clone this repository outside GitHub and hence cannot make use of its
templating function.

### Learning about egui

`src/app.rs` contains a simple example app. This is just to give some inspiration - most of it can be removed if you like.

The official egui docs are at <https://docs.rs/egui>. If you prefer watching a video introduction, check out <https://www.youtube.com/watch?v=NtUkr_z7l84>. For inspiration, check out the [the egui web demo](https://emilk.github.io/egui/index.html) and follow the links in it to its source code.

### Testing locally

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

You can test the template app at <https://emilk.github.io/eframe_template/>.

## Updating egui

As of 2023, egui is in active development with frequent releases with breaking changes. [eframe_template](https://github.com/goodpeanuts/dball/) will be updated in lock-step to always use the latest version of egui.

When updating `egui` and `eframe` it is recommended you do so one version at the time, and read about the changes in [the egui changelog](https://github.com/emilk/egui/blob/master/CHANGELOG.md) and [eframe changelog](https://github.com/emilk/egui/blob/master/crates/eframe/CHANGELOG.md).
