# `kmeans-colors` changelog

## Version 0.7.0 - 2023-07

Version bump for updating `rand` to `0.9`.

Optimized color conversion performance in the binary application.  
Switched the color parsing logic for the binary application to allow for 3 or 6
digit hexadecimal colors.  
See [#55][55] and [#57][57] for more details.

[#62][62] - Update rand crate and usage  
[#57][57] - Add fxhash for caching, relocate find_colors to find.rs  
[#55][55] - Speed up Lab conversion with Srgb/LinSrgb fast path, use palette FromStr impl

## Version 0.6.0 - 2023-07

Version bump for updating `palette` to `0.7`.

Users will need to change calls using
`palette::Pixel::{from_raw_slice, into_raw_slice}`
to `palette::cast::{from_component_slice, into_component_slice}` for preparing
the input image buffer. See the [documentation] or `lib.rs` file for examples.

[#52][52] - Upgrade palette to 0.7, bump crate version to 0.6, update CI/CD workflow

## Version 0.5.0 - 2022-03-17

Version bump for updating `palette` to `0.6`.

No changes to library code.

[#49][49] - Update metadata for 0.5 release; CI/CD, clippy, and `image` fixups  
[#44][44] - Upgrade `palette` to 0.6, fix clippy lints, `image` function fixups

## Version 0.4.0 - 2021-03-13

Version bump for updating the `rand` dependency to 0.8. No major API changes.

Minor change to `Sort::sort_indexed_colors` which now takes `&[Self]` instead of
`&Vec<Self>` in the trait definition.

[#41][41] - Prepare metadata for 0.4.0 release, small fixups  
[#40][40] - Update crate version to 0.4.0, rand to 0.8, rand_chacha to 0.3

## Version 0.3.4 - 2020-11-16

An upstream package was changed which prevented the crate from building when
installing with cargo from crates.io.

[#36][36] - Upgrade image to 0.23.11, bump to crate version 0.3.4  
[#32][32] - Move color impls to their own module, add lints  

## Version 0.3.3 - 2020-06-17

Added transparency support to the `find` sub-command. This will now work like
the main command where transparent pixels are disregarded for the k-means
calculation. Upstream dependencies have been updated, notably reading in PNG
images should have improved performance for the binary.

[#27][27] - Add transparency support for find/replace  
[#26][26] - Update dependencies  
[#25][25] - Refactor out raw array indexing in favor of iterators  

## Version 0.3.2 - 2020-06-03

Bug fix for k-means++ to avoid divide by zero errors and panics with rand.

[#23][23] - Fix bugs introduced by switch to kmeans++  

## Version 0.3.1 - 2020-06-03

Major performance improvements in the form of algorithmic
optimization. Hamerly's algorithm was added which allows for skipping the
calculation of many distance checks. kmeans++ initialization was also added
which provides better initialization for centroids and generally higher quality
results. Because of kmeans++, the results produced by this version will not
match the previous implementation exactly but performance will be drastically
improved.

[#20][20] - Implement kmeans++ for better centroid initialization  
[#18][18] - Implement Hamerly's algoirthm for Lab and Srgb  
[#16][16] - Add metadata for docs.rs, allow other Lab white points  

## Version 0.3.0 - 2020-05-27

This update completes the refactor into a more generic, reusable library crate,
and marks the first "stable" unstable release. The next breaking change release
will occur after the color dependency has been updated, which will bring better
performance to color and format conversions.
* Heavy dependencies have been made optional.
* Changes in the API should be much smaller.
* Binary performance has been improved due to the refactor.

[#13][13] - Rethink MapColor trait, add transparency support  
[#12][12] - Deduplicate code in palette file saving [BIN]  
[#11][11] - Move `map_indices_to_centroids` to its own trait  
[#10][10] - Make `palette` an optional feature, update docs  
[#09][9] - Reimplement kmeans with generics  
[#08][8] - Fix indexing error for proportional palettes [BIN]  

## Version 0.2.0 - 2020-05-22

[#06][6] - Bump to version 0.2.0  
[#05][5] - Output color palette as image [BIN]  
[#03][3] - Refactor crate into library with bin folder  

## Version 0.1.0 - 2020-04
* Initial Commit

[62]: https://github.com/okaneco/kmeans-colors/pull/62
[57]: https://github.com/okaneco/kmeans-colors/pull/57
[55]: https://github.com/okaneco/kmeans-colors/pull/55
[52]: https://github.com/okaneco/kmeans-colors/pull/52
[49]: https://github.com/okaneco/kmeans-colors/pull/49
[44]: https://github.com/okaneco/kmeans-colors/pull/44
[41]: https://github.com/okaneco/kmeans-colors/pull/41
[40]: https://github.com/okaneco/kmeans-colors/pull/40
[36]: https://github.com/okaneco/kmeans-colors/pull/36
[32]: https://github.com/okaneco/kmeans-colors/pull/32
[27]: https://github.com/okaneco/kmeans-colors/pull/27
[26]: https://github.com/okaneco/kmeans-colors/pull/26
[25]: https://github.com/okaneco/kmeans-colors/pull/25
[23]: https://github.com/okaneco/kmeans-colors/pull/23
[20]: https://github.com/okaneco/kmeans-colors/pull/20
[18]: https://github.com/okaneco/kmeans-colors/pull/18
[16]: https://github.com/okaneco/kmeans-colors/pull/16
[13]: https://github.com/okaneco/kmeans-colors/pull/13
[12]: https://github.com/okaneco/kmeans-colors/pull/12
[11]: https://github.com/okaneco/kmeans-colors/pull/11
[10]: https://github.com/okaneco/kmeans-colors/pull/10
[9]: https://github.com/okaneco/kmeans-colors/pull/9
[8]: https://github.com/okaneco/kmeans-colors/pull/8
[6]: https://github.com/okaneco/kmeans-colors/pull/6
[5]: https://github.com/okaneco/kmeans-colors/pull/5
[3]: https://github.com/okaneco/kmeans-colors/pull/3
[documentation]: https://docs.rs/kmeans_colors
