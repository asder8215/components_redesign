#![allow(unused)]
use std::{cmp, hint::black_box, path::Path};

use criterion::{Criterion, criterion_group, criterion_main};
use rand::random_range;

fn components_iter(path: &Path) {
    for _ in 0..1000 {
        let comps = path.components();
        for comp in comps {}
    }
}

fn components_next_iter(path: &Path) {
    for _ in 0..1000 {
        let mut comps = path.components();
        while let Some(comp) = comps.next() {}
    }
}

fn components_next_back_iter(path: &Path) {
    for _ in 0..1000 {
        let mut comps = path.components();
        while let Some(comp) = comps.next_back() {}
    }
}

fn path_iter(path: &Path) {
    for _ in 0..1000 {
        let comps = path.iter();
        for comp in comps {}
    }
}

fn as_path_iter(path: &Path) {
    for _ in 0..1000 {
        let mut comps = path.iter();
        while let Some(comp) = comps.next() {
            let path = comps.as_path();
        }
    }
}

fn eq_comps(path: &Path, other_path: &Path) {
    for _ in 0..1000 {
        path.components() == other_path.components();
    }
}

fn compare_comps(path: &Path, other_path: &Path) {
    for _ in 0..1000 {
        let comp = path.components();
        let other_comp = other_path.components();
        comp > other_comp;
    }
}

fn bench_components(c: &mut Criterion) {
    // maximum bytes for a file name on Linux,
    // we'll use this as an ideal limit on what a long
    // path component looks like
    const NAME_MAX: usize = 255;
    // path max on Linux, we'll use this as an ideal
    // limit on what a long path should be
    const PATH_MAX: usize = 4096;

    let mut path_strings = vec![];
    let short_comp = vec!["a/"].join("");
    let mut long_comp = vec!["a"; NAME_MAX].join("");
    long_comp.push('/');

    // Short Paths: 1 path component
    let mut relative_short_path_short_comps = String::new();
    let mut absolute_short_path_short_comps = String::from("/");
    relative_short_path_short_comps.push_str(&short_comp);
    absolute_short_path_short_comps.push_str(&short_comp);
    path_strings.push(("Rel Short Path Short Comp", relative_short_path_short_comps));
    path_strings.push(("Abs Short Path Short Comp", absolute_short_path_short_comps));

    let mut relative_short_path_long_comps = String::new();
    let mut absolute_short_path_long_comps = String::from("/");
    relative_short_path_long_comps.push_str(&long_comp);
    absolute_short_path_long_comps.push_str(&long_comp);
    path_strings.push(("Rel Short Path Long Comp", relative_short_path_long_comps));
    path_strings.push(("Abs Short Path Long Comp", absolute_short_path_long_comps));

    // Long Paths: PATH_MAX/sizeof(comp bytes)
    let mut relative_long_path_short_comps = String::new();
    let mut absolute_long_path_short_comps = String::from("/");

    for _ in 0..PATH_MAX / 2 {
        relative_long_path_short_comps.push_str(&short_comp);
        absolute_long_path_short_comps.push_str(&short_comp);
    }

    path_strings.push(("Rel Long Path Short Comp", relative_long_path_short_comps));
    path_strings.push(("Abs Long Path Short Comp", absolute_long_path_short_comps));

    let mut relative_long_path_long_comps = String::new();
    let mut absolute_long_path_long_comps = String::from("/");

    // +1 for separator byte
    for _ in 0..PATH_MAX / (NAME_MAX + 1) {
        relative_long_path_long_comps.push_str(&long_comp);
        absolute_long_path_long_comps.push_str(&long_comp);
    }

    path_strings.push(("Rel Long Path Long Comp", relative_long_path_long_comps));
    path_strings.push(("Abs Long Path Long Comp", absolute_long_path_long_comps));

    // Inconsistent sized paths: Similar as long path, but randomly
    // sized components
    let mut relative_long_path_inconsistent_comps = String::new();
    let mut absolute_long_path_inconsistent_comps = String::from("/");

    let mut counter = PATH_MAX;
    while counter > 1 {
        let rand = random_range(1..=cmp::min(NAME_MAX, counter));
        let mut a_string = String::new();

        for _ in 0..rand {
            a_string.push('a');
        }

        relative_long_path_inconsistent_comps.push_str(&a_string);
        absolute_long_path_inconsistent_comps.push_str(&a_string);

        counter -= rand;

        if counter > 1 {
            relative_long_path_inconsistent_comps.push('/');
            absolute_long_path_inconsistent_comps.push('/');
            counter -= 1;
        }
    }

    path_strings.push((
        "Rel Long Path Inconsistent Comp",
        relative_long_path_inconsistent_comps,
    ));
    path_strings.push((
        "Abs Long Path Inconsistent Comp",
        absolute_long_path_inconsistent_comps,
    ));

    for (case, path) in path_strings {
        let mut start_path_fail = path.clone();
        let mut mid_path_fail = path.clone();
        let mut end_path_fail = path.clone();
        start_path_fail.insert_str(1, "b/");
        mid_path_fail.insert_str(mid_path_fail.len() / 2, "b/");
        end_path_fail.push_str("b/");

        c.bench_function(&format!("{:?}, Std Components Next", case), |b| {
            b.iter(|| black_box(components_next_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Std Components Next Back", case), |b| {
            b.iter(|| black_box(components_next_back_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Std As Path Iter", case), |b| {
            b.iter(|| black_box(as_path_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Std Components Equality", case), |b| {
            b.iter(|| black_box(eq_comps(black_box(path.as_ref()), black_box(path.as_ref()))))
        });

        c.bench_function(
            &format!("{:?}, Std Components Equality Fail from Start", case),
            |b| {
                b.iter(|| {
                    black_box(eq_comps(
                        black_box(path.as_ref()),
                        black_box(start_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Equality Fail from Mid", case),
            |b| {
                b.iter(|| {
                    black_box(eq_comps(
                        black_box(path.as_ref()),
                        black_box(mid_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Equality Fail from End", case),
            |b| {
                b.iter(|| {
                    black_box(eq_comps(
                        black_box(path.as_ref()),
                        black_box(end_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Comparison Succeed", case),
            |b| {
                b.iter(|| {
                    black_box(compare_comps(
                        black_box(path.as_ref()),
                        black_box(path.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Comparison Fail from Start", case),
            |b| {
                b.iter(|| {
                    black_box(compare_comps(
                        black_box(path.as_ref()),
                        black_box(start_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Comparison Fail from Mid", case),
            |b| {
                b.iter(|| {
                    black_box(compare_comps(
                        black_box(path.as_ref()),
                        black_box(mid_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Comparison Fail from End", case),
            |b| {
                b.iter(|| {
                    black_box(compare_comps(
                        black_box(path.as_ref()),
                        black_box(end_path_fail.as_ref()),
                    ))
                })
            },
        );
    }
    // let mut path = String::from("/");
    // let chars = vec!["a"; 64];
    // let mut str = chars.join("");
    // str.push('/');

    // for i in 0..1000 {
    //     path.push_str(&str);
    // }

    // // "/a0..a64/a0..a64/a0..a64/.../b/"
    // let path_b = format!("{path}/b/");

    // // "/b/a0..a64/a0..a64/.../a0..a64/"
    // let path_c = format!("/b/{path}");

    // // c.bench_function("Std Components", |b| {
    // //     b.iter(|| black_box(components_iter(black_box(path.as_ref()))))
    // // });

    // c.bench_function("Std Components Next", |b| {
    //     b.iter(|| black_box(components_next_iter(black_box(path.as_ref()))))
    // });

    // c.bench_function("Std Components Next Back", |b| {
    //     b.iter(|| black_box(components_next_back_iter(black_box(path.as_ref()))))
    // });

    // // c.bench_function("Std Path Iter", |b| {
    // //     b.iter(|| black_box(path_iter(black_box(path.as_ref()))))
    // // });

    // c.bench_function("Std As Path Iter", |b| {
    //     b.iter(|| black_box(as_path_iter(black_box(path.as_ref()))))
    // });

    // c.bench_function("Std Eq Comps", |b| {
    //     b.iter(|| black_box(eq_comps(black_box(path.as_ref()), black_box(path.as_ref()))))
    // });

    // c.bench_function("Std Uneq Comps", |b| {
    //     b.iter(|| {
    //         black_box(eq_comps(
    //             black_box(path.as_ref()),
    //             black_box(path_b.as_ref()),
    //         ))
    //     })
    // });

    // c.bench_function("Std Uneq 2 Comps", |b| {
    //     b.iter(|| {
    //         black_box(eq_comps(
    //             black_box(path.as_ref()),
    //             black_box(path_c.as_ref()),
    //         ))
    //     })
    // });

    // c.bench_function("Std Compare Comps", |b| {
    //     b.iter(|| {
    //         black_box(compare_comps(
    //             black_box(path.as_ref()),
    //             black_box(path.as_ref()),
    //         ))
    //     })
    // });

    // c.bench_function("Std Compare Uneq Comps", |b| {
    //     b.iter(|| {
    //         black_box(compare_comps(
    //             black_box(path.as_ref()),
    //             black_box(path_b.as_ref()),
    //         ))
    //     })
    // });

    // c.bench_function("Std Compare Uneq 2 Comps", |b| {
    //     b.iter(|| {
    //         black_box(compare_comps(
    //             black_box(path.as_ref()),
    //             black_box(path_c.as_ref()),
    //         ))
    //     })
    // });

    // ----------- WITHOUT BLACK BOX ---------------------

    // c.bench_function("Std Components (No BB)", |b| {
    //     b.iter(|| {
    //         components_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Std Components Next (No BB)", |b| {
    //     b.iter(|| {
    //         components_next_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Std Components Next Back (No BB)", |b| {
    //     b.iter(|| {
    //         components_next_back_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Std Path Iter (No BB)", |b| {
    //     b.iter(|| {
    //         path_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Std As Path Iter (No BB)", |b| {
    //     b.iter(|| {
    //         as_path_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Std Eq Comps (No BB)", |b| {
    //     b.iter(|| {
    //         eq_comps(path.as_ref(), path.as_ref())
    //     })
    // });

    // c.bench_function("Std Uneq Comps (No BB)", |b| {
    //     b.iter(|| {
    //         eq_comps(path.as_ref(), path_b.as_ref())
    //     })
    // });

    // c.bench_function("Std Uneq 2 Comps (No BB)", |b| {
    //     b.iter(|| {
    //         eq_comps(path.as_ref(), path_c.as_ref())
    //     })
    // });

    // c.bench_function("Std Compare Comps (No BB)", |b| {
    //     b.iter(|| compare_comps(path.as_ref(), path.as_ref()))
    // });

    // c.bench_function("Std Compare Uneq Comps (No BB)", |b| {
    //     b.iter(|| compare_comps(path.as_ref(), path_b.as_ref()))
    // });

    // c.bench_function("Std Compare Uneq 2 Comps (No BB)", |b| {
    //     b.iter(|| compare_comps(path.as_ref(), path_c.as_ref()))
    // });
}

criterion_group!(benches, bench_components);
criterion_main!(benches);
