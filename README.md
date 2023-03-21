<img width="100%" src="logo/background.png">

[![build](https://github.com/refcell/varro/actions/workflows/test.yml/badge.svg)](https://github.com/refcell/varro/actions/workflows/test.yml) [![license: MIT](https://img.shields.io/badge/license-MIT-brightgreen.svg)](https://opensource.org/licenses/MIT) [![varro](https://img.shields.io/crates/v/varro.svg)](https://crates.io/crates/varro)

`varro` is a persistent, robust, and composable proposal service for rollup stacks written in pure rust.

> **Note**
>
> Varro is primarily tested against the [op-stack](https://stack.optimism.io/).

## Quickstart

Varro provides an easy to use installer called `varrup`.

To install `varro`, run: `curl https://raw.githubusercontent.com/refcell/varro/main/varrup/install | bash`.

Additionally, you can run varro for [ethereum-optimism](https://github.com/ethereum-optimism) using the provided scripts in the `scripts` directory.

## Architecture

At it's core, `varro` is a client. In [./src/client.rs](./src/client.rs), there is an `Varro` struct that is the core client/driver of the batcher.

The `Varro` struct first _builds_ "stages" and then executes them as asynchronous threads. These stages split up the transformation of data types and handles channel transitions and metrics.

The primary entrypoint for `varro` is [./src/client/rs](./src/client.rs) which is the `Varro` struct. `Varro` exposes methods for constructing proposals.

## Configuration

The `varro` cli maintains a verbose menu for running a proposal service. To see a list of all available commands, run `varro --help`. This will print output similar to the following:

```bash

```

### Environment Variables

The following environment variables are the default values for `varro`'s configuration.

They can be overridden by setting the environment variable in the shell before running `varro`, or setting the associated flags when running the `varro` cli.

```env

```

## Specifications

// TODO:

## Why "Varro"?

The term "varro" comes from [Marcus Terentius Varro](https://en.wikipedia.org/wiki/Marcus_Terentius_Varro), an ancient Roman polymath and prolific author. Referred to the "great third light of Rome", Varro mirrors the role of the `varro` service in the rollup stack, handling the "inscription" of outputs to the settlement layer (eg, posting L2 output proposals to Ethereum).

## FAQ

// TODO:

## Contributing

All contributions are welcome. Before opening a PR, please submit an issue detailing the bug or feature. When opening a PR, please ensure that your contribution builds on the nightly rust toolchain, has been linted with `cargo fmt`, and contains tests when applicable.

## Disclaimer

_This code is being provided as is. No guarantee, representation or warranty is being made, express or implied, as to the safety or correctness of the code. It has not been audited and as such there can be no assurance it will work as intended, and users may experience delays, failures, errors, omissions or loss of transmitted information. Nothing in this repo should be construed as investment advice or legal advice for any particular facts or circumstances and is not meant to replace competent counsel. It is strongly advised for you to contact a reputable attorney in your jurisdiction for any questions or concerns with respect thereto. The authors is not liable for any use of the foregoing, and users should proceed with caution and use at their own risk._
