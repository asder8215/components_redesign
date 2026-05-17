#![feature(try_blocks)]
#![allow(dead_code)]
#![allow(unused)]
use std::{fs::ReadDir, hint::black_box, path::Path};

use criterion::{Criterion, criterion_group, criterion_main};

fn components_iter(path: &Path) {
    let comps = path.components();
    for comp in comps {}
}

fn components_next_iter(path: &Path) {
    let mut comps = path.components();
    while let Some(comp) = comps.next() {}
}

fn components_next_back_iter(path: &Path) {
    let mut comps = path.components();
    while let Some(comp) = comps.next_back() {}
}

fn path_iter(path: &Path) {
    let comps = path.iter();
    for comp in comps {}
}

fn as_path_iter(path: &Path) {
    let mut comps = path.iter();
    while let Some(comp) = comps.next() {
        let path = comps.as_path();
    }
}

fn eq_comps(path: &Path, other_path: &Path) {
    path.components() == other_path.components();
}

fn compare_comps(path: &Path, other_path: &Path) {
    let comp = path.components();
    let other_comp = other_path.components();
    comp.cmp(other_comp);
}

fn bench_components(c: &mut Criterion) {
    let mut path = String::from("/");
    let chars = vec!["a"; 64];
    let mut str = chars.join("");
    str.push('/');

    for i in 0..1000 {
        path.push_str(&str);
    }

    // "/a/a/a/.../b/"
    let path_b = format!("{path}/b/");
    
    // "/b/a/a/.../a/"
    let path_c = format!("/b/{path}");

    // c.bench_function("Std Components", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(components_iter(black_box(path.as_ref())))
    //     })
    // });

    // c.bench_function("Std Components Next", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(components_next_iter(black_box(path.as_ref())))
    //     })
    // });

    // c.bench_function("Std Components Next Back", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(components_next_back_iter(black_box(path.as_ref())))
    //     })
    // });

    // c.bench_function("Std Path Iter", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(path_iter(black_box(path.as_ref())))
    //     })
    // });

    // c.bench_function("Std As Path Iter", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(as_path_iter(black_box(path.as_ref())))
    //     })
    // });

    // c.bench_function("Std Eq Comps", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(eq_comps(black_box(path.as_ref()), black_box(path.as_ref())))
    //     })
    // });

    // c.bench_function("Std Uneq Comps", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(eq_comps(black_box(path.as_ref()), black_box(path_b.as_ref())))
    //     })
    // });

    // c.bench_function("Std Uneq 2 Comps", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(eq_comps(black_box(path.as_ref()), black_box(path_c.as_ref())))
    //     })
    // });

    // c.bench_function("Std Compare Comps", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(compare_comps(black_box(path.as_ref()), black_box(path.as_ref())))
    //     })
    // });

    // c.bench_function("Std Compare Uneq Comps", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(compare_comps(black_box(path.as_ref()), black_box(path_b.as_ref())))
    //     })
    // });

    // c.bench_function("Std Compare Uneq 2 Comps", |b| {
    //     b.iter(|| {
    //         // Use black_box to prevent compiler optimizations from 
    //         // skipping the code you want to measure
    //         black_box(compare_comps(black_box(path.as_ref()), black_box(path_c.as_ref())))
    //     })
    // });

    // ----------- WITHOUT BLACK BOX ---------------------

    c.bench_function("Std Components (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            components_iter(path.as_ref())
        })
    });

    c.bench_function("Std Components Next (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            components_next_iter(path.as_ref())
        })
    });

    c.bench_function("Std Components Next Back (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            components_next_back_iter(path.as_ref())
        })
    });

    c.bench_function("Std Path Iter (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            path_iter(path.as_ref())
        })
    });

    c.bench_function("Std As Path Iter (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            as_path_iter(path.as_ref())
        })
    });

    c.bench_function("Std Eq Comps (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            eq_comps(path.as_ref(), path.as_ref())
        })
    });

    c.bench_function("Std Uneq Comps (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            eq_comps(path.as_ref(), path_b.as_ref())
        })
    });

    c.bench_function("Std Uneq 2 Comps (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            eq_comps(path.as_ref(), path_c.as_ref())
        })
    });

    c.bench_function("Std Compare Comps (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            compare_comps(path.as_ref(), path.as_ref())
        })
    });

    c.bench_function("Std Compare Uneq Comps (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            compare_comps(path.as_ref(), path_b.as_ref())
        })
    });

    c.bench_function("Std Compare Uneq 2 Comps (No BB)", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from 
            // skipping the code you want to measure
            compare_comps(path.as_ref(), path_c.as_ref())
        })
    });
}

criterion_group!(benches, bench_components);
criterion_main!(benches);