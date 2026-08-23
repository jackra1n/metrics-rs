# Changelog

## [0.3.0](https://github.com/jackra1n/metrics-rs/compare/0.2.0...0.3.0) (2026-08-23)


### Features

* add fact icons rest licenses and analysis meta line ([1881567](https://github.com/jackra1n/metrics-rs/commit/1881567e8e90dff9ba92f325f1c2ffca351d613a))
* add GitHub Action metadata and workflow documentation ([c628789](https://github.com/jackra1n/metrics-rs/commit/c6287892cb191675219135a67c2bb468f62ff547))
* add in-depth authored language analysis ([6dfb707](https://github.com/jackra1n/metrics-rs/commit/6dfb707a2e9935c817dfaa2b4e9db92880820a4d))
* compute language stats with linguist colors ([5d852eb](https://github.com/jackra1n/metrics-rs/commit/5d852eb69ad8787dd38fdeb39bf907251c914e17))
* display lifetime contributed repos count matching og metrics ([8e83b6a](https://github.com/jackra1n/metrics-rs/commit/8e83b6a763cc38bc5096d5607388a551f5c19d29))
* fetch profile and repositories via graphql ([96d5862](https://github.com/jackra1n/metrics-rs/commit/96d58628887381f6bb581fc56fde9e249f1955ea))
* fetch weekly contributor line stats ([c9bc221](https://github.com/jackra1n/metrics-rs/commit/c9bc221d620b76a4f95ad0d6a6111e3d4ecf8ced))
* parse cli arguments ([f1a5f3b](https://github.com/jackra1n/metrics-rs/commit/f1a5f3b83dbf485374d459c69ca2419f95a2532d))
* print detailed stats, activity repos, and language impact to stdout ([7573c72](https://github.com/jackra1n/metrics-rs/commit/7573c721dacde38508226b7b4f7164544343d4c5))
* redesign card with fact grid and compact header ([225d207](https://github.com/jackra1n/metrics-rs/commit/225d207fcbd6c8878c5fad310a0a01f781c017be))
* render dark svg ([c3b45cb](https://github.com/jackra1n/metrics-rs/commit/c3b45cbadaec9bd7a66104ac0c85d708b840dc25))


### Bug Fixes

* deserialize graphql totalCount fields ([8335c51](https://github.com/jackra1n/metrics-rs/commit/8335c51919e145301c2bfdcc23a18afdce6ec8fa))
* normalize chart amplitude to card bounds ([9300d57](https://github.com/jackra1n/metrics-rs/commit/9300d57648fed9236958a01c9e748c0210d2188a))


### Styles

* fix clippy warnings and format ([9ef29e1](https://github.com/jackra1n/metrics-rs/commit/9ef29e10ac8302b3a8c3c369c016cc4254fd2708))


### Documentation

* add README and AGPL-3.0 license ([a90a764](https://github.com/jackra1n/metrics-rs/commit/a90a764402b8f7877008f62aacda1790ac2c882c))
* clean up headings and formatting in README ([9cddfb2](https://github.com/jackra1n/metrics-rs/commit/9cddfb2c37b3ef07f55d1ca88de66a07491304d0))
* move preview metrics.svg to docs/images/ ([4ed815d](https://github.com/jackra1n/metrics-rs/commit/4ed815dac392e24afaa1420c250e14139c756cd3))
* remove terminal summary bullet from features in README ([bb5a7d3](https://github.com/jackra1n/metrics-rs/commit/bb5a7d37f108c1dfbad2f90c08d848bb62e4e1eb))
* remove test and license footer sections from README ([c57b39c](https://github.com/jackra1n/metrics-rs/commit/c57b39c17a6517bfd8cbc5b925f6210b0c7a9199))
* simplify CLI usage examples in README ([3995bfe](https://github.com/jackra1n/metrics-rs/commit/3995bfed08d1d466fdb9b6509dddbd12f5a05037))


### Tests

* cover formatting dates aggregation and render ([84d5f2f](https://github.com/jackra1n/metrics-rs/commit/84d5f2facbbd6b3d5a1707d2c4b966bab4feb924))


### Build

* add dependencies ([bc97981](https://github.com/jackra1n/metrics-rs/commit/bc97981c6b715165fa90c31b191e4776e0cff177))


### CI/CD

* add code quality and test workflow with clippy and rustfmt ([7947959](https://github.com/jackra1n/metrics-rs/commit/79479596aa867c8cf5abec7a73f4759f64de08be))
* add release-please workflow and prebuilt binary download in action ([7c019d7](https://github.com/jackra1n/metrics-rs/commit/7c019d766425f38259e700f9d90d0195f0dd0dd5))
* align release-please configuration structure with czkawka-web ([e67115c](https://github.com/jackra1n/metrics-rs/commit/e67115cb1acc8565166d522c3a73d2cf771127f5))
* configure release-please to use raw version tags without v prefix ([1d03fa8](https://github.com/jackra1n/metrics-rs/commit/1d03fa861df082338446078206ec23c37634b10b))
* disable component prefix in release-please tags, release names, and PR titles ([5075fbe](https://github.com/jackra1n/metrics-rs/commit/5075fbee8ee2da6e271e9e8cc55c5de9efb24510))
* remove legacy x86_64-apple-darwin from release matrix ([a9b8eb8](https://github.com/jackra1n/metrics-rs/commit/a9b8eb881568a5252dffae2f32ff600bafed64de))
* upgrade actions to Node 24 targeting versions ([3433dfa](https://github.com/jackra1n/metrics-rs/commit/3433dfa361383838cca8dd436393c917b6f1ffb7))

## [0.2.0](https://github.com/jackra1n/metrics-rs/compare/metrics-rs-0.1.0...metrics-rs-0.2.0) (2026-08-23)


### Features

* add fact icons rest licenses and analysis meta line ([1881567](https://github.com/jackra1n/metrics-rs/commit/1881567e8e90dff9ba92f325f1c2ffca351d613a))
* add GitHub Action metadata and workflow documentation ([c628789](https://github.com/jackra1n/metrics-rs/commit/c6287892cb191675219135a67c2bb468f62ff547))
* add in-depth authored language analysis ([6dfb707](https://github.com/jackra1n/metrics-rs/commit/6dfb707a2e9935c817dfaa2b4e9db92880820a4d))
* compute language stats with linguist colors ([5d852eb](https://github.com/jackra1n/metrics-rs/commit/5d852eb69ad8787dd38fdeb39bf907251c914e17))
* display lifetime contributed repos count matching og metrics ([8e83b6a](https://github.com/jackra1n/metrics-rs/commit/8e83b6a763cc38bc5096d5607388a551f5c19d29))
* fetch profile and repositories via graphql ([96d5862](https://github.com/jackra1n/metrics-rs/commit/96d58628887381f6bb581fc56fde9e249f1955ea))
* fetch weekly contributor line stats ([c9bc221](https://github.com/jackra1n/metrics-rs/commit/c9bc221d620b76a4f95ad0d6a6111e3d4ecf8ced))
* parse cli arguments ([f1a5f3b](https://github.com/jackra1n/metrics-rs/commit/f1a5f3b83dbf485374d459c69ca2419f95a2532d))
* print detailed stats, activity repos, and language impact to stdout ([7573c72](https://github.com/jackra1n/metrics-rs/commit/7573c721dacde38508226b7b4f7164544343d4c5))
* redesign card with fact grid and compact header ([225d207](https://github.com/jackra1n/metrics-rs/commit/225d207fcbd6c8878c5fad310a0a01f781c017be))
* render dark svg ([c3b45cb](https://github.com/jackra1n/metrics-rs/commit/c3b45cbadaec9bd7a66104ac0c85d708b840dc25))


### Bug Fixes

* deserialize graphql totalCount fields ([8335c51](https://github.com/jackra1n/metrics-rs/commit/8335c51919e145301c2bfdcc23a18afdce6ec8fa))
* normalize chart amplitude to card bounds ([9300d57](https://github.com/jackra1n/metrics-rs/commit/9300d57648fed9236958a01c9e748c0210d2188a))


### Styles

* fix clippy warnings and format ([9ef29e1](https://github.com/jackra1n/metrics-rs/commit/9ef29e10ac8302b3a8c3c369c016cc4254fd2708))


### Documentation

* add README and AGPL-3.0 license ([a90a764](https://github.com/jackra1n/metrics-rs/commit/a90a764402b8f7877008f62aacda1790ac2c882c))
* clean up headings and formatting in README ([9cddfb2](https://github.com/jackra1n/metrics-rs/commit/9cddfb2c37b3ef07f55d1ca88de66a07491304d0))
* move preview metrics.svg to docs/images/ ([4ed815d](https://github.com/jackra1n/metrics-rs/commit/4ed815dac392e24afaa1420c250e14139c756cd3))
* remove terminal summary bullet from features in README ([bb5a7d3](https://github.com/jackra1n/metrics-rs/commit/bb5a7d37f108c1dfbad2f90c08d848bb62e4e1eb))
* remove test and license footer sections from README ([c57b39c](https://github.com/jackra1n/metrics-rs/commit/c57b39c17a6517bfd8cbc5b925f6210b0c7a9199))
* simplify CLI usage examples in README ([3995bfe](https://github.com/jackra1n/metrics-rs/commit/3995bfed08d1d466fdb9b6509dddbd12f5a05037))


### Tests

* cover formatting dates aggregation and render ([84d5f2f](https://github.com/jackra1n/metrics-rs/commit/84d5f2facbbd6b3d5a1707d2c4b966bab4feb924))


### Build

* add dependencies ([bc97981](https://github.com/jackra1n/metrics-rs/commit/bc97981c6b715165fa90c31b191e4776e0cff177))


### CI/CD

* add release-please workflow and prebuilt binary download in action ([7c019d7](https://github.com/jackra1n/metrics-rs/commit/7c019d766425f38259e700f9d90d0195f0dd0dd5))
* align release-please configuration structure with czkawka-web ([e67115c](https://github.com/jackra1n/metrics-rs/commit/e67115cb1acc8565166d522c3a73d2cf771127f5))
* configure release-please to use raw version tags without v prefix ([1d03fa8](https://github.com/jackra1n/metrics-rs/commit/1d03fa861df082338446078206ec23c37634b10b))
