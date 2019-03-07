# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## [Unreleased] -->

## [0.4.0] - 2019-03-07

### Changed

- `GlyphBrush::draw` now takes an iterator of `Section`s instead of just one, allowing a single batched draw call.

### Fixed

- Fixed text rendering not using alpha blending. Thanks to [@knappador] for [#4].

[@knappador]: https://github.com/knappador
[#4]: https://github.com/andrewhickman/vulkano-glyph/pull/4

[0.4.0]: https://github.com/andrewhickman/vulkano-glyph/compare/v0.3.0...v0.4.0
[Unreleased]: https://github.com/andrewhickman/vulkano-glyph/compare/v0.4.0...HEAD