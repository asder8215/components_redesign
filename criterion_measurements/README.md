The release compiled std is based on the rustc build commit 6e44b7b25aec040ef7b39b2e27988aae1a380b89.

You can run
```
rustup-toolchain-install-master 6e44b7b25aec040ef7b39b2e27988aae1a380b89
```

to install a rustc build with PGO + release mode optimizations with the Components optimized code for non-prefixed platforms.

Disclaimer: I used Claude Sonnet 5 to parse through the raw Criterion measurement results I got from benchmarking Components iterator vs std Components iterator into well formatted md code tables (there's like 374 rows, so it would've taken me way longer to do this by hand or take some time to write a python script for this). I have cross-checked it with the raw Criterion results and it should be reflect the avg measurement results from what I got from Criterion. All benchmarking code is written myself, and I will explain the process of what I did to benchmark this below. You can find the benchmarking code in the [`components_subslice`](https://github.com/asder8215/components_redesign/tree/components_subslice) branch of my `components_redesign` repo (specifically in [`benches/components_slice.rs`](https://github.com/asder8215/components_redesign/blob/components_subslice/benches/components_slice.rs)). The raw criterion results is also in the folder [criterion_measurements/criterion_raw_report](https://github.com/asder8215/components_redesign/tree/components_subslice/criterion_measurements).

Note, I have only done the release optimizations benchmarking for `Components` (based around the code that is currently within [`nonprefixed_components.rs`](https://github.com/rust-lang/rust/pull/156496/changes#diff-6fcfc9fc9e6bb1d8ca7bf41677e82b46f9acfbc1b2564ef1a2ab30c8b499cab2) in this commit). I haven't done debug benchmarking yet because it takes a while for this to benchmarked completely (like ~1 hour and half), and I kept making tweaks into benchmarking the two iterators.

Right now my methodology is as follows:
* Benchmarking this revolves around two different macros defined in Linux: `PATH_MAX` and `NAME_MAX`. `PATH_MAX`, which is 4096, is the number of bytes maximally allowed as a passed in path. `NAME_MAX`, which is 255, is the number of bytes maximally allowed as a file name. I use these two values to determine my stop points for experimenting a path and how large path component should be.
* I experimented the following types:
    * Absolute (Abs) vs Relative (Rel) paths:
        * This subdivides down to long paths (`PATH_MAX`/(sizeof(path component) number of components) vs short paths (1 path component):
            * The size of each path components are further divided into the following numbers: 1, 3, 7, 15, 31, 63, 127, 255 (meant to do powers of 2s, like 1, 2, 4, 8, 16, 32, 64, 128, and then last one would be 255 instead of 256, but eh I did minus one from 2 onward).
            * We use the byte "a" in all the components (maybe might've been useful to use a random character? But I just chose to make this simpler.)
         * I also experimented on inconsistently sized components (paths with randomly sized components from 1-255 bytes), but it's only a long path right now (goes up to `PATH_MAX`, maybe a slightly a bit more due to a separator byte at the end). I randomly generated one such path and then chose that to be a fixed value, which you can see in the benchmark code (both as a relative path, and then made it an absolute path with a `/` byte at the start). 
* What I benchmarked were the following (both this Components and Std Components version):
    * Using `Components::next` to iterate through a given path
    * Using `Components::next_back` to iterate through a given path
    * Iterating through a given path with `Components::next` and using `Components::as_path` at every iteration
    * Components Equality
        * Equality on the path itself
        * Equality on the path vs same path with "b/" inserted at index 1
        * Equality on the path vs same path with "b/" inserted at path length / 2
        * Equality on the path vs same path with "b/" inserted at the end
     * Components Comparison:
        * `>` on the path itself
        * `>` on the path vs same path with "b/" inserted at index 1
        * `>` on the path vs same path with "b/" inserted at path length / 2
        * `>` on the path vs same path with "b/" inserted at the end
     * Internally, all of these functions run the same iteration/equality/comparison 100 times, to reduce noise from one iteration. 

Looking back, there's a lot more I could've benchmarked and experimented with. For example, I could've varied the number of path components we had in total instead of doing minimum number of component (short path, 1 comp) and maximum number of path components (long path, `PATH_MAX`/sizeof(path components). I could've also done inconsistent path component size with varying number of path components instead of making it a long path. But benchmarking all those different types of combos/permutations likely would've taken a quarter/a third of my day.
