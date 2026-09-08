# Criterion Benchmark Results — Component (Debug) vs Release Compiled Std Components (Debug)

Times shown are the Criterion point estimate (middle of the confidence interval). Component was built in debug mode; Std Components was compiled in release mode but benchmarked under the debug harness/profile.

### Rel Short Path with 1 byte comps

| Rel Short Path with 1 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 6.4312 µs | 1.3832 µs |
| Components Next Back | 8.2556 µs | 1.5810 µs |
| As Path Iter | 8.1168 µs | 2.6898 µs |
| Components Equality Succeed | 1.1031 µs | 1.8857 µs |
| Components Equality Fail from Start | 34.129 µs | 4.1185 µs |
| Components Equality Fail from Mid | 34.205 µs | 4.1405 µs |
| Components Equality Fail from End | 10.156 µs | 2.2407 µs |
| Components Comparison Succeed | 2.8118 µs | 2.1071 µs |
| Components Comparison Fail from Start | 28.810 µs | 3.3287 µs |
| Components Comparison Fail from Mid | 28.959 µs | 3.3154 µs |
| Components Comparison Fail from End | 20.040 µs | 3.0959 µs |

### Abs Short Path with 1 byte comps

| Abs Short Path with 1 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 9.6039 µs | 1.7211 µs |
| Components Next Back | 10.976 µs | 1.8755 µs |
| As Path Iter | 14.005 µs | 4.1632 µs |
| Components Equality Succeed | 1.0956 µs | 1.8874 µs |
| Components Equality Fail from Start | 27.995 µs | 3.8081 µs |
| Components Equality Fail from Mid | 28.264 µs | 3.8369 µs |
| Components Equality Fail from End | 10.023 µs | 2.2176 µs |
| Components Comparison Succeed | 2.7624 µs | 2.1181 µs |
| Components Comparison Fail from Start | 9.2362 µs | 2.1395 µs |
| Components Comparison Fail from Mid | 9.1597 µs | 2.1536 µs |
| Components Comparison Fail from End | 21.411 µs | 3.1147 µs |

### Rel Short Path with 3 byte comps

| Rel Short Path with 3 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 6.8295 µs | 1.3907 µs |
| Components Next Back | 9.3180 µs | 1.6435 µs |
| As Path Iter | 8.5926 µs | 2.6790 µs |
| Components Equality Succeed | 1.1082 µs | 1.8854 µs |
| Components Equality Fail from Start | 40.314 µs | 4.1895 µs |
| Components Equality Fail from Mid | 36.968 µs | 4.1539 µs |
| Components Equality Fail from End | 10.117 µs | 2.2419 µs |
| Components Comparison Succeed | 2.8111 µs | 2.1063 µs |
| Components Comparison Fail from Start | 9.1603 µs | 2.1521 µs |
| Components Comparison Fail from Mid | 10.645 µs | 2.1721 µs |
| Components Comparison Fail from End | 22.583 µs | 3.1766 µs |

### Abs Short Path with 3 byte comps

| Abs Short Path with 3 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 10.175 µs | 1.7324 µs |
| Components Next Back | 11.821 µs | 2.2510 µs |
| As Path Iter | 14.524 µs | 4.1381 µs |
| Components Equality Succeed | 1.1005 µs | 1.9048 µs |
| Components Equality Fail from Start | 32.822 µs | 4.0028 µs |
| Components Equality Fail from Mid | 40.958 µs | 4.2814 µs |
| Components Equality Fail from End | 10.022 µs | 2.2230 µs |
| Components Comparison Succeed | 2.7677 µs | 2.1185 µs |
| Components Comparison Fail from Start | 9.2037 µs | 2.1486 µs |
| Components Comparison Fail from Mid | 10.681 µs | 2.1926 µs |
| Components Comparison Fail from End | 24.686 µs | 3.1787 µs |

### Rel Short Path with 7 byte comps

| Rel Short Path with 7 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 7.9510 µs | 1.4267 µs |
| Components Next Back | 11.506 µs | 2.1043 µs |
| As Path Iter | 9.7936 µs | 2.7209 µs |
| Components Equality Succeed | 1.1159 µs | 1.8856 µs |
| Components Equality Fail from Start | 53.924 µs | 4.4860 µs |
| Components Equality Fail from Mid | 46.053 µs | 4.3328 µs |
| Components Equality Fail from End | 10.166 µs | 2.2337 µs |
| Components Comparison Succeed | 2.8198 µs | 2.1048 µs |
| Components Comparison Fail from Start | 9.2094 µs | 2.1277 µs |
| Components Comparison Fail from Mid | 13.630 µs | 2.2182 µs |
| Components Comparison Fail from End | 28.688 µs | 3.2150 µs |

### Abs Short Path with 7 byte comps

| Abs Short Path with 7 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 11.097 µs | 1.7817 µs |
| Components Next Back | 13.865 µs | 2.8028 µs |
| As Path Iter | 15.642 µs | 4.1931 µs |
| Components Equality Succeed | 1.1102 µs | 1.9067 µs |
| Components Equality Fail from Start | 41.034 µs | 4.0472 µs |
| Components Equality Fail from Mid | 48.925 µs | 4.4167 µs |
| Components Equality Fail from End | 10.040 µs | 2.3223 µs |
| Components Comparison Succeed | 2.7328 µs | 2.2083 µs |
| Components Comparison Fail from Start | 9.2857 µs | 2.1817 µs |
| Components Comparison Fail from Mid | 13.727 µs | 2.2310 µs |
| Components Comparison Fail from End | 29.755 µs | 3.3410 µs |

### Rel Short Path with 15 byte comps

| Rel Short Path with 15 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 11.115 µs | 1.5874 µs |
| Components Next Back | 16.621 µs | 2.4514 µs |
| As Path Iter | 12.019 µs | 2.8428 µs |
| Components Equality Succeed | 1.1163 µs | 1.9086 µs |
| Components Equality Fail from Start | 83.590 µs | 5.4708 µs |
| Components Equality Fail from Mid | 62.990 µs | 4.9191 µs |
| Components Equality Fail from End | 10.316 µs | 2.2485 µs |
| Components Comparison Succeed | 2.8545 µs | 2.1086 µs |
| Components Comparison Fail from Start | 9.6687 µs | 2.1540 µs |
| Components Comparison Fail from Mid | 19.255 µs | 2.2778 µs |
| Components Comparison Fail from End | 41.613 µs | 3.3207 µs |

### Abs Short Path with 15 byte comps

| Abs Short Path with 15 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 13.529 µs | 2.0069 µs |
| Components Next Back | 18.713 µs | 2.7643 µs |
| As Path Iter | 18.213 µs | 4.3217 µs |
| Components Equality Succeed | 1.1167 µs | 1.9318 µs |
| Components Equality Fail from Start | 59.069 µs | 4.1703 µs |
| Components Equality Fail from Mid | 65.417 µs | 4.9442 µs |
| Components Equality Fail from End | 10.141 µs | 2.2184 µs |
| Components Comparison Succeed | 2.7356 µs | 2.1036 µs |
| Components Comparison Fail from Start | 9.2567 µs | 2.1493 µs |
| Components Comparison Fail from Mid | 18.959 µs | 2.2776 µs |
| Components Comparison Fail from End | 42.448 µs | 3.3289 µs |

### Rel Short Path with 31 byte comps

| Rel Short Path with 31 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 15.446 µs | 1.9932 µs |
| Components Next Back | 26.167 µs | 3.2269 µs |
| As Path Iter | 16.560 µs | 3.0788 µs |
| Components Equality Succeed | 1.1297 µs | 1.9134 µs |
| Components Equality Fail from Start | 143.12 µs | 8.5507 µs |
| Components Equality Fail from Mid | 96.029 µs | 6.2338 µs |
| Components Equality Fail from End | 10.226 µs | 2.2370 µs |
| Components Comparison Succeed | 2.8952 µs | 2.1134 µs |
| Components Comparison Fail from Start | 9.2302 µs | 2.1684 µs |
| Components Comparison Fail from Mid | 30.734 µs | 2.4174 µs |
| Components Comparison Fail from End | 64.418 µs | 3.5681 µs |

### Abs Short Path with 31 byte comps

| Abs Short Path with 31 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 18.310 µs | 2.3563 µs |
| Components Next Back | 28.696 µs | 3.4758 µs |
| As Path Iter | 22.836 µs | 4.4182 µs |
| Components Equality Succeed | 1.1325 µs | 1.9222 µs |
| Components Equality Fail from Start | 93.973 µs | 4.5847 µs |
| Components Equality Fail from Mid | 99.047 µs | 6.6018 µs |
| Components Equality Fail from End | 10.154 µs | 2.2453 µs |
| Components Comparison Succeed | 2.8486 µs | 2.1013 µs |
| Components Comparison Fail from Start | 9.2550 µs | 2.1535 µs |
| Components Comparison Fail from Mid | 30.507 µs | 2.3924 µs |
| Components Comparison Fail from End | 65.627 µs | 3.5942 µs |

### Rel Short Path with 63 byte comps

| Rel Short Path with 63 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 25.075 µs | 3.1238 µs |
| Components Next Back | 45.342 µs | 4.5110 µs |
| As Path Iter | 25.918 µs | 4.0738 µs |
| Components Equality Succeed | 1.1306 µs | 1.8983 µs |
| Components Equality Fail from Start | 265.94 µs | 14.292 µs |
| Components Equality Fail from Mid | 166.82 µs | 10.424 µs |
| Components Equality Fail from End | 10.176 µs | 2.2336 µs |
| Components Comparison Succeed | 2.8696 µs | 2.1083 µs |
| Components Comparison Fail from Start | 9.2810 µs | 2.1654 µs |
| Components Comparison Fail from Mid | 54.478 µs | 2.7346 µs |
| Components Comparison Fail from End | 109.86 µs | 4.3586 µs |

### Abs Short Path with 63 byte comps

| Abs Short Path with 63 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 28.158 µs | 3.4502 µs |
| Components Next Back | 48.358 µs | 5.2335 µs |
| As Path Iter | 32.894 µs | 5.4845 µs |
| Components Equality Succeed | 1.1476 µs | 1.9025 µs |
| Components Equality Fail from Start | 169.17 µs | 6.1577 µs |
| Components Equality Fail from Mid | 169.04 µs | 10.589 µs |
| Components Equality Fail from End | 10.221 µs | 2.2189 µs |
| Components Comparison Succeed | 2.8445 µs | 2.1402 µs |
| Components Comparison Fail from Start | 9.2819 µs | 2.1598 µs |
| Components Comparison Fail from Mid | 55.161 µs | 2.7371 µs |
| Components Comparison Fail from End | 113.19 µs | 4.3921 µs |

### Rel Short Path with 127 byte comps

| Rel Short Path with 127 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 44.229 µs | 4.4855 µs |
| Components Next Back | 84.237 µs | 7.5454 µs |
| As Path Iter | 44.218 µs | 5.3062 µs |
| Components Equality Succeed | 1.1602 µs | 1.8953 µs |
| Components Equality Fail from Start | 510.07 µs | 25.154 µs |
| Components Equality Fail from Mid | 306.53 µs | 17.556 µs |
| Components Equality Fail from End | 10.120 µs | 2.2324 µs |
| Components Comparison Succeed | 2.8732 µs | 2.1240 µs |
| Components Comparison Fail from Start | 9.2563 µs | 2.1303 µs |
| Components Comparison Fail from Mid | 101.19 µs | 4.0032 µs |
| Components Comparison Fail from End | 201.72 µs | 6.1748 µs |

### Abs Short Path with 127 byte comps

| Abs Short Path with 127 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 46.958 µs | 4.7426 µs |
| Components Next Back | 87.602 µs | 7.9309 µs |
| As Path Iter | 51.863 µs | 6.9481 µs |
| Components Equality Succeed | 1.2046 µs | 1.8199 µs |
| Components Equality Fail from Start | 315.68 µs | 8.9081 µs |
| Components Equality Fail from Mid | 309.19 µs | 18.075 µs |
| Components Equality Fail from End | 10.085 µs | 2.2141 µs |
| Components Comparison Succeed | 2.8220 µs | 2.1651 µs |
| Components Comparison Fail from Start | 9.1819 µs | 2.1591 µs |
| Components Comparison Fail from Mid | 101.19 µs | 4.0523 µs |
| Components Comparison Fail from End | 202.51 µs | 6.2185 µs |

### Rel Short Path with 255 byte comps

| Rel Short Path with 255 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 83.421 µs | 7.2487 µs |
| Components Next Back | 162.59 µs | 12.880 µs |
| As Path Iter | 82.254 µs | 8.1003 µs |
| Components Equality Succeed | 1.1475 µs | 1.7991 µs |
| Components Equality Fail from Start | 1.0103 ms | 44.790 µs |
| Components Equality Fail from Mid | 585.69 µs | 30.463 µs |
| Components Equality Fail from End | 10.278 µs | 2.2355 µs |
| Components Comparison Succeed | 2.8928 µs | 2.1693 µs |
| Components Comparison Fail from Start | 9.2628 µs | 2.1453 µs |
| Components Comparison Fail from Mid | 193.97 µs | 5.4546 µs |
| Components Comparison Fail from End | 391.37 µs | 8.9605 µs |

### Abs Short Path with 255 byte comps

| Abs Short Path with 255 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 84.789 µs | 7.4019 µs |
| Components Next Back | 163.00 µs | 13.423 µs |
| As Path Iter | 89.809 µs | 9.5925 µs |
| Components Equality Succeed | 1.2040 µs | 1.9196 µs |
| Components Equality Fail from Start | 631.77 µs | 14.237 µs |
| Components Equality Fail from Mid | 589.77 µs | 31.047 µs |
| Components Equality Fail from End | 10.119 µs | 2.2228 µs |
| Components Comparison Succeed | 2.8605 µs | 2.1977 µs |
| Components Comparison Fail from Start | 9.2625 µs | 2.1529 µs |
| Components Comparison Fail from Mid | 192.78 µs | 5.5045 µs |
| Components Comparison Fail from End | 388.23 µs | 8.8948 µs |

### Rel Long Path with 1 byte comps

| Rel Long Path with 1 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 12.630 ms | 920.94 µs |
| Components Next Back | 14.639 ms | 907.81 µs |
| As Path Iter | 18.703 ms | 2.7865 ms |
| Components Equality Succeed | 3.7073 µs | 4.2708 µs |
| Components Equality Fail from Start | 10.028 ms | 176.24 µs |
| Components Equality Fail from Mid | 5.0002 ms | 88.073 µs |
| Components Equality Fail from End | 10.180 µs | 2.2354 µs |
| Components Comparison Succeed | 5.2331 µs | 4.5242 µs |
| Components Comparison Fail from Start | 30.068 µs | 3.3374 µs |
| Components Comparison Fail from Mid | 2.9804 ms | 47.761 µs |
| Components Comparison Fail from End | 6.0421 ms | 89.231 µs |

### Abs Long Path with 1 byte comps

| Abs Long Path with 1 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 12.722 ms | 995.99 µs |
| Components Next Back | 14.683 ms | 907.51 µs |
| As Path Iter | 18.753 ms | 2.8004 ms |
| Components Equality Succeed | 3.7552 µs | 4.3131 µs |
| Components Equality Fail from Start | 9.9088 ms | 177.19 µs |
| Components Equality Fail from Mid | 5.0605 ms | 90.852 µs |
| Components Equality Fail from End | 10.140 µs | 2.2044 µs |
| Components Comparison Succeed | 5.6001 µs | 4.5705 µs |
| Components Comparison Fail from Start | 9.1891 µs | 2.1323 µs |
| Components Comparison Fail from Mid | 2.9866 ms | 49.404 µs |
| Components Comparison Fail from End | 5.9963 ms | 88.339 µs |

### Rel Long Path with 3 byte comps

| Rel Long Path with 3 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 6.7311 ms | 472.73 µs |
| Components Next Back | 8.3256 ms | 540.10 µs |
| As Path Iter | 9.6730 ms | 1.3750 ms |
| Components Equality Succeed | 3.7388 µs | 4.2611 µs |
| Components Equality Fail from Start | 9.9941 ms | 177.55 µs |
| Components Equality Fail from Mid | 4.9950 ms | 88.586 µs |
| Components Equality Fail from End | 10.163 µs | 2.2355 µs |
| Components Comparison Succeed | 5.2868 µs | 4.5703 µs |
| Components Comparison Fail from Start | 9.2633 µs | 2.1391 µs |
| Components Comparison Fail from Mid | 2.9777 ms | 46.011 µs |
| Components Comparison Fail from End | 5.9894 ms | 89.159 µs |

### Abs Long Path with 3 byte comps

| Abs Long Path with 3 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 6.7163 ms | 515.02 µs |
| Components Next Back | 8.3336 ms | 536.68 µs |
| As Path Iter | 9.6184 ms | 1.3936 ms |
| Components Equality Succeed | 3.8675 µs | 4.3358 µs |
| Components Equality Fail from Start | 10.010 ms | 175.75 µs |
| Components Equality Fail from Mid | 5.0754 ms | 90.985 µs |
| Components Equality Fail from End | 10.056 µs | 2.2184 µs |
| Components Comparison Succeed | 5.6315 µs | 4.6088 µs |
| Components Comparison Fail from Start | 9.1700 µs | 2.1529 µs |
| Components Comparison Fail from Mid | 2.9996 ms | 47.258 µs |
| Components Comparison Fail from End | 5.9613 ms | 88.860 µs |

### Rel Long Path with 7 byte comps

| Rel Long Path with 7 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 3.9718 ms | 297.09 µs |
| Components Next Back | 5.2910 ms | 359.90 µs |
| As Path Iter | 5.4358 ms | 727.96 µs |
| Components Equality Succeed | 3.8444 µs | 4.3612 µs |
| Components Equality Fail from Start | 9.9751 ms | 178.46 µs |
| Components Equality Fail from Mid | 5.0386 ms | 88.761 µs |
| Components Equality Fail from End | 10.171 µs | 2.2339 µs |
| Components Comparison Succeed | 5.6553 µs | 4.6522 µs |
| Components Comparison Fail from Start | 9.2167 µs | 2.1286 µs |
| Components Comparison Fail from Mid | 3.0008 ms | 45.840 µs |
| Components Comparison Fail from End | 6.0109 ms | 88.938 µs |

### Abs Long Path with 7 byte comps

| Abs Long Path with 7 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 3.9635 ms | 321.14 µs |
| Components Next Back | 5.2971 ms | 354.33 µs |
| As Path Iter | 5.5001 ms | 745.69 µs |
| Components Equality Succeed | 3.8645 µs | 4.2570 µs |
| Components Equality Fail from Start | 9.9022 ms | 176.10 µs |
| Components Equality Fail from Mid | 5.0731 ms | 90.219 µs |
| Components Equality Fail from End | 10.102 µs | 2.2211 µs |
| Components Comparison Succeed | 5.6511 µs | 4.5663 µs |
| Components Comparison Fail from Start | 9.2520 µs | 2.1299 µs |
| Components Comparison Fail from Mid | 3.0073 ms | 48.400 µs |
| Components Comparison Fail from End | 6.0405 ms | 89.558 µs |

### Rel Long Path with 15 byte comps

| Rel Long Path with 15 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 2.5719 ms | 220.04 µs |
| Components Next Back | 3.7755 ms | 281.82 µs |
| As Path Iter | 3.3208 ms | 415.47 µs |
| Components Equality Succeed | 3.8639 µs | 4.2920 µs |
| Components Equality Fail from Start | 10.032 ms | 176.73 µs |
| Components Equality Fail from Mid | 5.0592 ms | 88.528 µs |
| Components Equality Fail from End | 10.065 µs | 2.2303 µs |
| Components Comparison Succeed | 5.6979 µs | 4.6407 µs |
| Components Comparison Fail from Start | 9.2944 µs | 2.1424 µs |
| Components Comparison Fail from Mid | 2.9626 ms | 46.729 µs |
| Components Comparison Fail from End | 5.9428 ms | 92.886 µs |

### Abs Long Path with 15 byte comps

| Abs Long Path with 15 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 2.5674 ms | 223.51 µs |
| Components Next Back | 3.7915 ms | 280.06 µs |
| As Path Iter | 3.3063 ms | 434.67 µs |
| Components Equality Succeed | 3.8955 µs | 4.2873 µs |
| Components Equality Fail from Start | 10.133 ms | 176.59 µs |
| Components Equality Fail from Mid | 5.0302 ms | 92.120 µs |
| Components Equality Fail from End | 10.077 µs | 2.2030 µs |
| Components Comparison Succeed | 5.6598 µs | 4.5778 µs |
| Components Comparison Fail from Start | 9.2331 µs | 2.1574 µs |
| Components Comparison Fail from Mid | 2.9835 ms | 48.977 µs |
| Components Comparison Fail from End | 5.9868 ms | 89.479 µs |

### Rel Long Path with 31 byte comps

| Rel Long Path with 31 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.9130 ms | 152.83 µs |
| Components Next Back | 3.2133 ms | 227.54 µs |
| As Path Iter | 2.2590 ms | 258.85 µs |
| Components Equality Succeed | 3.9356 µs | 4.3610 µs |
| Components Equality Fail from Start | 9.9336 ms | 178.92 µs |
| Components Equality Fail from Mid | 4.9537 ms | 88.871 µs |
| Components Equality Fail from End | 10.189 µs | 2.2417 µs |
| Components Comparison Succeed | 5.7491 µs | 4.6540 µs |
| Components Comparison Fail from Start | 9.1659 µs | 2.1448 µs |
| Components Comparison Fail from Mid | 2.9625 ms | 46.521 µs |
| Components Comparison Fail from End | 5.9493 ms | 90.517 µs |

### Abs Long Path with 31 byte comps

| Abs Long Path with 31 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.8993 ms | 155.83 µs |
| Components Next Back | 3.1726 ms | 226.89 µs |
| As Path Iter | 2.2641 ms | 264.38 µs |
| Components Equality Succeed | 3.9151 µs | 4.2626 µs |
| Components Equality Fail from Start | 9.9147 ms | 175.99 µs |
| Components Equality Fail from Mid | 4.9688 ms | 92.702 µs |
| Components Equality Fail from End | 10.023 µs | 2.2073 µs |
| Components Comparison Succeed | 5.5788 µs | 4.6256 µs |
| Components Comparison Fail from Start | 9.2572 µs | 2.1445 µs |
| Components Comparison Fail from Mid | 3.0511 ms | 49.002 µs |
| Components Comparison Fail from End | 5.9731 ms | 89.439 µs |

### Rel Long Path with 63 byte comps

| Rel Long Path with 63 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.5919 ms | 143.54 µs |
| Components Next Back | 2.8035 ms | 224.15 µs |
| As Path Iter | 1.7604 ms | 193.41 µs |
| Components Equality Succeed | 3.8186 µs | 4.3330 µs |
| Components Equality Fail from Start | 10.113 ms | 184.60 µs |
| Components Equality Fail from Mid | 5.0366 ms | 87.861 µs |
| Components Equality Fail from End | 10.168 µs | 2.2385 µs |
| Components Comparison Succeed | 5.6501 µs | 4.6421 µs |
| Components Comparison Fail from Start | 9.2882 µs | 2.1242 µs |
| Components Comparison Fail from Mid | 2.9938 ms | 45.545 µs |
| Components Comparison Fail from End | 5.9417 ms | 91.003 µs |

### Abs Long Path with 63 byte comps

| Abs Long Path with 63 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.5820 ms | 142.64 µs |
| Components Next Back | 2.8086 ms | 223.24 µs |
| As Path Iter | 1.7607 ms | 198.64 µs |
| Components Equality Succeed | 3.9671 µs | 4.2870 µs |
| Components Equality Fail from Start | 9.9558 ms | 176.16 µs |
| Components Equality Fail from Mid | 5.1023 ms | 97.211 µs |
| Components Equality Fail from End | 10.027 µs | 2.2350 µs |
| Components Comparison Succeed | 5.6485 µs | 4.6732 µs |
| Components Comparison Fail from Start | 9.1661 µs | 2.1424 µs |
| Components Comparison Fail from Mid | 3.0612 ms | 53.388 µs |
| Components Comparison Fail from End | 6.0034 ms | 89.369 µs |

### Rel Long Path with 127 byte comps

| Rel Long Path with 127 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.4174 ms | 116.39 µs |
| Components Next Back | 2.6107 ms | 199.27 µs |
| As Path Iter | 1.4950 ms | 141.21 µs |
| Components Equality Succeed | 4.1709 µs | 4.9598 µs |
| Components Equality Fail from Start | 10.202 ms | 193.41 µs |
| Components Equality Fail from Mid | 5.0444 ms | 88.640 µs |
| Components Equality Fail from End | 10.058 µs | 2.2293 µs |
| Components Comparison Succeed | 5.7048 µs | 5.1965 µs |
| Components Comparison Fail from Start | 9.2470 µs | 2.1403 µs |
| Components Comparison Fail from Mid | 2.9906 ms | 46.242 µs |
| Components Comparison Fail from End | 5.9377 ms | 90.853 µs |

### Abs Long Path with 127 byte comps

| Abs Long Path with 127 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.3966 ms | 119.75 µs |
| Components Next Back | 2.6310 ms | 202.22 µs |
| As Path Iter | 1.4929 ms | 146.00 µs |
| Components Equality Succeed | 3.8439 µs | 4.3905 µs |
| Components Equality Fail from Start | 10.049 ms | 175.31 µs |
| Components Equality Fail from Mid | 5.2424 ms | 104.20 µs |
| Components Equality Fail from End | 10.255 µs | 2.2356 µs |
| Components Comparison Succeed | 5.6187 µs | 4.6577 µs |
| Components Comparison Fail from Start | 9.2705 µs | 2.1251 µs |
| Components Comparison Fail from Mid | 3.1995 ms | 56.690 µs |
| Components Comparison Fail from End | 6.3176 ms | 89.491 µs |

### Rel Long Path with 255 byte comps

| Rel Long Path with 255 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.2963 ms | 101.32 µs |
| Components Next Back | 2.5419 ms | 186.07 µs |
| As Path Iter | 1.3535 ms | 114.69 µs |
| Components Equality Succeed | 3.8252 µs | 4.3357 µs |
| Components Equality Fail from Start | 10.419 ms | 206.85 µs |
| Components Equality Fail from Mid | 5.0610 ms | 87.963 µs |
| Components Equality Fail from End | 10.289 µs | 2.2205 µs |
| Components Comparison Succeed | 5.6942 µs | 4.6441 µs |
| Components Comparison Fail from Start | 9.2716 µs | 2.1615 µs |
| Components Comparison Fail from Mid | 2.9625 ms | 48.451 µs |
| Components Comparison Fail from End | 6.0154 ms | 89.488 µs |

### Abs Long Path with 255 byte comps

| Abs Long Path with 255 byte comps | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.3054 ms | 102.20 µs |
| Components Next Back | 2.5408 ms | 185.46 µs |
| As Path Iter | 1.3650 ms | 117.24 µs |
| Components Equality Succeed | 3.8231 µs | 4.3220 µs |
| Components Equality Fail from Start | 10.002 ms | 176.57 µs |
| Components Equality Fail from Mid | 5.3206 ms | 112.76 µs |
| Components Equality Fail from End | 10.038 µs | 2.2200 µs |
| Components Comparison Succeed | 5.5582 µs | 4.5844 µs |
| Components Comparison Fail from Start | 9.2978 µs | 2.1460 µs |
| Components Comparison Fail from Mid | 3.3385 ms | 65.404 µs |
| Components Comparison Fail from End | 5.9985 ms | 89.367 µs |

### Rel Long Path Inconsistent Comp

| Rel Long Path Inconsistent Comp | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.4331 ms | 120.62 µs |
| Components Next Back | 2.6417 ms | 201.44 µs |
| As Path Iter | 1.5060 ms | 147.89 µs |
| Components Equality Succeed | 4.0057 µs | 4.3882 µs |
| Components Equality Fail from Start | 10.249 ms | 195.41 µs |
| Components Equality Fail from Mid | 5.0793 ms | 95.280 µs |
| Components Equality Fail from End | 33.342 µs | 3.9342 µs |
| Components Comparison Succeed | 5.5377 µs | 4.7277 µs |
| Components Comparison Fail from Start | 9.2995 µs | 2.1205 µs |
| Components Comparison Fail from Mid | 2.9469 ms | 47.419 µs |
| Components Comparison Fail from End | 6.0816 ms | 91.525 µs |

### Abs Long Path Inconsistent Comp

| Abs Long Path Inconsistent Comp | Component (Debug Mode) | Release Compiled Std Components (Debug) |
|---|---|---|
| Components Next | 1.4355 ms | 121.55 µs |
| Components Next Back | 2.6499 ms | 202.75 µs |
| As Path Iter | 1.5214 ms | 151.29 µs |
| Components Equality Succeed | 3.8225 µs | 4.2626 µs |
| Components Equality Fail from Start | 10.142 ms | 177.93 µs |
| Components Equality Fail from Mid | 5.1277 ms | 95.859 µs |
| Components Equality Fail from End | 33.489 µs | 3.9511 µs |
| Components Comparison Succeed | 5.5358 µs | 4.6313 µs |
| Components Comparison Fail from Start | 9.1755 µs | 2.1344 µs |
| Components Comparison Fail from Mid | 2.9439 ms | 45.828 µs |
| Components Comparison Fail from End | 5.9969 ms | 89.451 µs |
