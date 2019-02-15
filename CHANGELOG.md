# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- `GlyphBrush::draw` now takes an iterator of `Section`s instead of just one, allowing a single batched draw call.

[Unreleased]: https://github.com/andrewhickman/vulkano-glyph/compare/v0.2.0...HEAD