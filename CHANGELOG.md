# `kmeans-colors` changelog

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

[13]: https://github.com/okaneco/kmeans-colors/pull/13
[12]: https://github.com/okaneco/kmeans-colors/pull/12
[11]: https://github.com/okaneco/kmeans-colors/pull/11
[10]: https://github.com/okaneco/kmeans-colors/pull/10
[9]: https://github.com/okaneco/kmeans-colors/pull/9
[8]: https://github.com/okaneco/kmeans-colors/pull/8
[6]: https://github.com/okaneco/kmeans-colors/pull/6
[5]: https://github.com/okaneco/kmeans-colors/pull/5
[3]: https://github.com/okaneco/kmeans-colors/pull/3
