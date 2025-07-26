# Changelog
All notable changes to this project will be documented in this file.


## [0.3.1-rc.2] - 2025-07-26


### Bug Fixes

- (**ci**) Added missing write permission in the github release job ([e98b802](https://github.com/PyDataBlog/simstring_rs/commit/e98b8021403add8a6427526f66a3d1441c4192f7))


### Documentation

- (**benchmarks**) Update benchmark results ([74e435e](https://github.com/PyDataBlog/simstring_rs/commit/74e435e322bd65f001885746fd12581b0cca2bd1))



## [0.3.1-rc.1] - 2025-07-26


### Documentation

- (**benchmarks**) Update benchmark results ([7c3a453](https://github.com/PyDataBlog/simstring_rs/commit/7c3a4539282e70afc64fe63a395607afb3ec1f47))


### Miscellaneous Tasks

- (**ci**) Added missing hrefs in changelogs & added specs to benchmarks [#37](https://github.com/PyDataBlog/simstring_rs/issues/37) ([b4357e0](https://github.com/PyDataBlog/simstring_rs/commit/b4357e0ef5af512bdef6b458047c03a50f4e2a66))



## [0.3.1-beta.3] - 2025-07-18


### Bug Fixes

- (**ci**) Attempt to fix broken auto commit of benchmark results on a tagged release [#32](https://github.com/PyDataBlog/simstring_rs/issues/32) ([71d5a5e](https://github.com/PyDataBlog/simstring_rs/commit/71d5a5e9646a7881a02e50473efaf273e324deb6))



## [0.3.1-beta.2] - 2025-07-17


### Bug Fixes

- (**ci**) Remove old wheels from current jobs [#28](https://github.com/PyDataBlog/simstring_rs/issues/28) ([0507233](https://github.com/PyDataBlog/simstring_rs/commit/0507233320db06f57466c5c9602769fe9789bb15))

- (**ci**) Maturin can't publish generated bindings as it can't find the correct path [#29](https://github.com/PyDataBlog/simstring_rs/issues/29) ([2579a2b](https://github.com/PyDataBlog/simstring_rs/commit/2579a2b77df54e107741e6dae522541772520339))

- (**ci**) Moved repo urls for maturin to env vars [#30](https://github.com/PyDataBlog/simstring_rs/issues/30) ([d81deef](https://github.com/PyDataBlog/simstring_rs/commit/d81deeffc42cdfebd54b8e87df7274ceb8175791))

- (**ci**) Added missing dist file path for python wheels [#31](https://github.com/PyDataBlog/simstring_rs/issues/31) ([74f259a](https://github.com/PyDataBlog/simstring_rs/commit/74f259a5b3a5a64161c3e1bff2f9b6850235e409))


### Documentation

- (**benchmarks**) Update benchmark results ([f34d96f](https://github.com/PyDataBlog/simstring_rs/commit/f34d96f33a807d6cd92b1b3dab17f8006db9bdc9))

- (**benchmarks**) Update benchmark results ([0766907](https://github.com/PyDataBlog/simstring_rs/commit/07669076026eb8f26973999fdb5b8af3829ee4ee))

- (**benchmarks**) Update benchmark results ([1129c90](https://github.com/PyDataBlog/simstring_rs/commit/1129c9080853e4aeeb4bd62e044f280437370b52))


### Features

- (**ci**) Added pypi and test pypi publishing jobs [#24](https://github.com/PyDataBlog/simstring_rs/issues/24) ([b59cbc6](https://github.com/PyDataBlog/simstring_rs/commit/b59cbc6ca54cf43954142dab85d5597e1a1fc0ba))



## [0.3.1-alpha.1] - 2025-07-14


### Bug Fixes

- (**release**) Get rid of git-cliff as a dev dependencies [#26](https://github.com/PyDataBlog/simstring_rs/issues/26) ([0af2a25](https://github.com/PyDataBlog/simstring_rs/commit/0af2a25d4a14d812927bc0414442be403ff4a3bd))

- (**release**) Fixed hyperlink bug in git-cliff config [#27](https://github.com/PyDataBlog/simstring_rs/issues/27) ([6564d87](https://github.com/PyDataBlog/simstring_rs/commit/6564d8770715ecd6d0d82d4acdd4344ed30b3170))


### Documentation

- (**benchmarks**) Update benchmark results ([f08bd21](https://github.com/PyDataBlog/simstring_rs/commit/f08bd2143df00eb2db7e50aa85e446113780f1ea))

- (**benchmarks**) Update benchmark results ([4f62c62](https://github.com/PyDataBlog/simstring_rs/commit/4f62c621f961b1bcb5ce0a8a7becb9db20b55671))

- (**benchmarks**) Update benchmark results ([b77b264](https://github.com/PyDataBlog/simstring_rs/commit/b77b264a5eea0fa97cb6c47565d43cdab77340b7))

- (**benchmarks**) Update benchmark results ([d99bdcb](https://github.com/PyDataBlog/simstring_rs/commit/d99bdcb291bb006f270d476f0c2ab34bcc252816))


### Features

- (**release**) Initial exploration of an automated release management system [#25](https://github.com/PyDataBlog/simstring_rs/issues/25) ([7c28e01](https://github.com/PyDataBlog/simstring_rs/commit/7c28e0141b2208d9c695fab2fbc0f3a76b547892))



## [0.3.0] - 2025-07-04


### Features

- (**python**) Add python bindings to the project [#21](https://github.com/PyDataBlog/simstring_rs/issues/21) ([b206537](https://github.com/PyDataBlog/simstring_rs/commit/b2065379f1700ad480a123bd6fc4ea4d1c0ae6df))



## [0.2.0] - 2025-06-30


### Refactor

- (**api**) Optimize data structures to avoid unnecessary allocations and enable parallel searches [#18](https://github.com/PyDataBlog/simstring_rs/issues/18) ([f10972f](https://github.com/PyDataBlog/simstring_rs/commit/f10972fd76a944384757f4283c61d804436cf83b))



## [0.1.3] - 2025-06-29


### Features

- PR and Issues template [#17](https://github.com/PyDataBlog/simstring_rs/issues/17) ([77c74fc](https://github.com/PyDataBlog/simstring_rs/commit/77c74fc6156377e254f838a0ac475a896f85c210))



## [0.1.2] - 2025-01-07


### Feat

- Initial  benchmark suite [#11](https://github.com/PyDataBlog/simstring_rs/issues/11) ([f14274a](https://github.com/PyDataBlog/simstring_rs/commit/f14274a99ad13d9c9a914cb195ca9c8833317987))


### Release

- Initial benchmarks compared to other implementations in other languages [#13](https://github.com/PyDataBlog/simstring_rs/issues/13) ([9fca265](https://github.com/PyDataBlog/simstring_rs/commit/9fca26563cbba35bd65215b6d2482bf932150866))



## [0.1.1] - 2024-12-15


### WIP

- Initial proposed search method for hashdb [#7](https://github.com/PyDataBlog/simstring_rs/issues/7) ([19b6068](https://github.com/PyDataBlog/simstring_rs/commit/19b6068d2272af4d84ff4aad2da8f1c29827906d))



## [0.1.0] - 2024-12-06


### Fix

- Added tag trigger to CI workflow [#2](https://github.com/PyDataBlog/simstring_rs/issues/2) ([386c0f1](https://github.com/PyDataBlog/simstring_rs/commit/386c0f1b2bd153eafadb0fa64e590ae56cac1bbd))

- Keywords not exceeding 5 items [#3](https://github.com/PyDataBlog/simstring_rs/issues/3) ([3a57b84](https://github.com/PyDataBlog/simstring_rs/commit/3a57b840c15b66da4813f3d18a8b8ca8d57f9bd3))


### Init

- Initial commit ([32d6a3b](https://github.com/PyDataBlog/simstring_rs/commit/32d6a3b9ecb2775e03ce68279291647ca18689d2))


<!-- generated by git-cliff -->
