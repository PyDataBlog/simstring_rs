## What's Changed in 0.3.4
* feat(bindings): Expose extractor interface via CustomExtractor for Python bindings by @PyDataBlog in [#50](https://github.com/PyDataBlog/simstring_rs/pull/50)
* feat(ci): Initial benchmark support for original C++ implementation by @PyDataBlog in [#54](https://github.com/PyDataBlog/simstring_rs/pull/54)
* fix(ci): github release notes were skipping not rc tags by @PyDataBlog in [#49](https://github.com/PyDataBlog/simstring_rs/pull/49)
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.3...0.3.4

## What's Changed in 0.3.3
* chore(release): 0.3.3 by @PyDataBlog
* feat(bindings): expose extractor for direct usage in Python by @PyDataBlog in [#47](https://github.com/PyDataBlog/simstring_rs/pull/47)
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.2...v0.3.3

## What's Changed in 0.3.2
* chore(release): 0.3.2 by @PyDataBlog
* feat(ci): Exploring support for wheels for old linux distros by @PyDataBlog in [#45](https://github.com/PyDataBlog/simstring_rs/pull/45)
* perf(search): latest tweaks to boost search performance by @PyDataBlog in [#42](https://github.com/PyDataBlog/simstring_rs/pull/42)
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.1...v0.3.2

## What's Changed in 0.3.1
* chore(release): 0.3.1 by @PyDataBlog
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.1-rc.3...v0.3.1

## What's Changed in 0.3.1-rc.3
* chore(release): 0.3.1-rc.3 by @PyDataBlog
* feat(perf): better search performance by avoiding allocations by @PyDataBlog in [#40](https://github.com/PyDataBlog/simstring_rs/pull/40)
* feat(ci): Added test coverage by @PyDataBlog in [#39](https://github.com/PyDataBlog/simstring_rs/pull/39)
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.1-rc.2...v0.3.1-rc.3

## What's Changed in 0.3.1-rc.2
* chore(release): 0.3.1-rc.2 by @PyDataBlog
* fix(ci): added missing write permission in the github release job by @PyDataBlog
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.1-rc.1...v0.3.1-rc.2

## What's Changed in 0.3.1-rc.1
* chore(release): 0.3.1-rc.1 by @PyDataBlog
* chore(ci): added missing hrefs in changelogs & added specs to benchmarks by @PyDataBlog in [#37](https://github.com/PyDataBlog/simstring_rs/pull/37)
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.1-beta.3...v0.3.1-rc.1

## What's Changed in 0.3.1-beta.3
* chore(release): 0.3.1-beta.3 by @PyDataBlog
* fix(ci): Attempt to fix broken auto commit of benchmark results by @PyDataBlog in [#32](https://github.com/PyDataBlog/simstring_rs/pull/32)

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.1-beta.2...v0.3.1-beta.3

## What's Changed in 0.3.1-beta.2
* chore(release): 0.3.1-beta.2 by @PyDataBlog
* fix(ci): Added missing dist file path for python wheels by @PyDataBlog in [#31](https://github.com/PyDataBlog/simstring_rs/pull/31)
* fix(ci): Moved repo urls for maturin to env vars by @PyDataBlog in [#30](https://github.com/PyDataBlog/simstring_rs/pull/30)
* fix(ci): maturin can't publish generated bindings by @PyDataBlog in [#29](https://github.com/PyDataBlog/simstring_rs/pull/29)
* feat(ci): Added pypi and test pypi publishing jobs by @PyDataBlog in [#24](https://github.com/PyDataBlog/simstring_rs/pull/24)
* docs(benchmarks): update benchmark results by @PyDataBlog
* chore(release): 0.3.1-beta.1 by @PyDataBlog
* docs(benchmarks): update benchmark results by @PyDataBlog
* fix(ci): remove old wheels from current jobs by @PyDataBlog in [#28](https://github.com/PyDataBlog/simstring_rs/pull/28)
* docs(benchmarks): update benchmark results by @PyDataBlog

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.1-alpha.1...v0.3.1-beta.2

## What's Changed in 0.3.1-alpha.1
* chore(release): 0.3.1-alpha.1 by @PyDataBlog
* docs(benchmarks): update benchmark results by @PyDataBlog
* fix(release): fixed hyperlink bug in git-cliff config by @PyDataBlog in [#27](https://github.com/PyDataBlog/simstring_rs/pull/27)
* docs(benchmarks): update benchmark results by @PyDataBlog
* fix(release): Get rid of git-cliff as a dev dependencies by @PyDataBlog in [#26](https://github.com/PyDataBlog/simstring_rs/pull/26)
* docs(benchmarks): update benchmark results by @PyDataBlog
* feat(release): Initial exploration of an automated release management system by @PyDataBlog in [#25](https://github.com/PyDataBlog/simstring_rs/pull/25)
* docs(benchmarks): update benchmark results by @PyDataBlog
* feat!(benchmarks): Refactored benchmarks to produce structure outputs by @PyDataBlog in [#23](https://github.com/PyDataBlog/simstring_rs/pull/23)

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.3.0...v0.3.1-alpha.1

## What's Changed in 0.3.0
* feat(python): Add python bindings to the project by @PyDataBlog in [#21](https://github.com/PyDataBlog/simstring_rs/pull/21)

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.2.0...v0.3.0

## What's Changed in 0.2.0
* refactor(api): Optimize data structures to avoid unnecessary allocations and enable parallel searches by @PyDataBlog in [#18](https://github.com/PyDataBlog/simstring_rs/pull/18)

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.1.3...v0.2.0

## What's Changed in 0.1.3
* feat: PR and Issues template by @PyDataBlog in [#17](https://github.com/PyDataBlog/simstring_rs/pull/17)
* added order indipendence test by @icfly2 in [#12](https://github.com/PyDataBlog/simstring_rs/pull/12)

### New Contributors
* @icfly2 made their first contribution in [#12](https://github.com/PyDataBlog/simstring_rs/pull/12)

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.1.2...v0.1.3

## What's Changed in 0.1.2
* Release: Initial benchmarks compared to other implementations by @PyDataBlog in [#13](https://github.com/PyDataBlog/simstring_rs/pull/13)
* Feat: Initial  benchmark suite by @PyDataBlog in [#11](https://github.com/PyDataBlog/simstring_rs/pull/11)

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.1.1...v0.1.2

## What's Changed in 0.1.1
* WIP: Initial proposed search method for hashdb by @PyDataBlog in [#7](https://github.com/PyDataBlog/simstring_rs/pull/7)

**Full Changelog**: https://github.com/PyDataBlog/simstring_rs/compare/v0.1.0...v0.1.1

## What's Changed in 0.1.0
* initial WIP release of simsstring_rust by @PyDataBlog
* Set version to 0.1.0 for initial release of simsstring_rust by @PyDataBlog
* Set version to 0.1.0 for initial release by @PyDataBlog
* Bumped version to v0.1.3 by @PyDataBlog
* Fixed imports based on new name by @PyDataBlog
* Rename crate to simstring_rust and update metadata by @PyDataBlog
* Fix: keywords not exceeding 5 items by @PyDataBlog in [#3](https://github.com/PyDataBlog/simstring_rs/pull/3)
* Fix: Added tag trigger to CI workflow by @PyDataBlog in [#2](https://github.com/PyDataBlog/simstring_rs/pull/2)
* Initial API structure by @PyDataBlog in [#1](https://github.com/PyDataBlog/simstring_rs/pull/1)
* Init: Initial commit by @PyDataBlog

### New Contributors
* @PyDataBlog made their first contribution

<!-- generated by git-cliff -->
