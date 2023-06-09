use std::thread::{self, JoinHandle};

use easybench::bench;
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPool, ThreadPoolBuilder,
};
use sysinfo::{CpuRefreshKind, RefreshKind, System, SystemExt};
use tokio::{runtime::Runtime, task::JoinHandle as TokioJoinHandle};

fn main() {
    // setup variables
    let sys = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    let threads = sys.cpus().len();

    let datasets: Vec<(i32, i64)> = vec![
        (1_000, 233168),
        (10_000, 23331668),
        (100_000, 2333316668),
        (1_000_000, 233333166668),
        (10_000_000, 23333331666668),
        (100_000_000, 2333333316666668),
        // (1_000_000_000, 233333333166666668),
    ];

    // run benchmarks
    for (input, res) in datasets {
        println!("input: {}, res: {}", input, res);

        assert_eq!(res, euler1_unpar(input)); // ensure function return correct result
        mark4("euler1_unpar", euler1_unpar, input); // manual benchmark
                                                    // println!("euler1_unpar: {}", bench(|| euler1_unpar(input))); // easybench benchmark

        // assert_eq!(res, euler1_par(input));
        // println!("euler1_par: {}", bench(|| euler1_par(input)));

        for i in 1..=threads {
            assert_eq!(res, euler1_par_chunk(input, i as i32));
            mark4_chunk(
                format!("euler1_par_chunk{i}"),
                euler1_par_chunk,
                input,
                i as i32,
            ); // manual benchmark
               // println!(
               //     "euler1_par_chunk{i}: {}",
               //     bench(|| euler1_par_chunk(input, i as i32))
               // );
        }

        // assert_eq!(res, euler1_rayon_native(input));
        // println!(
        //     "euler1_rayon_native: {}",
        //     bench(|| euler1_rayon_native(input))
        // );

        for i in 1..=threads {
            let pool = ThreadPoolBuilder::new().num_threads(i).build().unwrap();
            assert_eq!(res, euler1_rayon(input, &pool));
            mark4_rayon(format!("euler1_rayon{i}"), euler1_rayon, input, &pool);
            // manual benchmark
            // println!("euler1_rayon{i}: {}", bench(|| euler1_rayon(input, &pool)));
        }

        for i in 1..=threads {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                //.worker_threads(i)
                .build()
                .unwrap();
            assert_eq!(res, euler1_tokio(input, &runtime, i as i32));
            mark4_tokio(
                format!("euler1_tokio{i}"),
                euler1_tokio,
                input,
                &runtime,
                i as i32,
            );
            // println!(
            //     "euler1_tokio{i}: {}",
            //     bench(|| euler1_tokio(input, &runtime, i as i32))
            // );
        }
    }
}

fn mark4(name: &str, func: fn(i32) -> i64, input: i32) -> i64 {
    let count = 100;
    let n = 30;
    let mut st = 0;
    let mut sst = 0;
    let mut res: i64 = 0;
    for _ in 0..n {
        let start = std::time::Instant::now();
        for _ in 0..count {
            res += func(input);
        }
        let end = std::time::Instant::now();
        let time = end.duration_since(start).as_nanos() / count;
        st += time;
        sst += time * time;
    }
    let mean = st / n;
    let sdev = (((sst - mean * mean * n) / (n - 1)) as f64).sqrt();
    println!("{name}: {mean}ns +/- {sdev:.2} -> {res}");

    res
}

fn mark4_chunk(name: String, func: fn(i32, i32) -> i64, input: i32, thread_count: i32) -> i64 {
    let count = 100;
    let n = 30;
    let mut st = 0;
    let mut sst = 0;
    let mut res: i64 = 0;
    for _ in 0..n {
        let start = std::time::Instant::now();
        for _ in 0..count {
            res += func(input, thread_count);
        }
        let end = std::time::Instant::now();
        let time = end.duration_since(start).as_nanos() / count;
        st += time;
        sst += time * time;
    }
    let mean = st / n;
    let sdev = (((sst - mean * mean * n) / (n - 1)) as f64).sqrt();
    println!("{name}: {mean}ns +/- {sdev:.2}");

    res
}

fn mark4_rayon(
    name: String,
    func: fn(i32, &ThreadPool) -> i64,
    input: i32,
    pool: &ThreadPool,
) -> i64 {
    let count = 100;
    let n = 30;
    let mut st = 0;
    let mut sst = 0;
    let mut res: i64 = 0;
    for _ in 0..n {
        let start = std::time::Instant::now();
        for _ in 0..count {
            res += func(input, pool);
        }
        let end = std::time::Instant::now();
        let time = end.duration_since(start).as_nanos() / count;
        st += time;
        sst += time * time;
    }
    let mean = st / n;
    let sdev = (((sst - mean * mean * n) / (n - 1)) as f64).sqrt();
    println!("{name}: {mean}ns +/- {sdev:.2}");

    res
}

fn mark4_tokio(
    name: String,
    func: fn(i32, &Runtime, i32) -> i64,
    input: i32,
    runtime: &Runtime,
    thread_count: i32,
) -> i64 {
    let count = 100;
    let n = 30;
    let mut st = 0;
    let mut sst = 0;
    let mut res: i64 = 0;
    for _ in 0..n {
        let start = std::time::Instant::now();
        for _ in 0..count {
            res += func(input, runtime, thread_count);
        }
        let end = std::time::Instant::now();
        let time = end.duration_since(start).as_nanos() / count;
        st += time;
        sst += time * time;
    }
    let mean = st / n;
    let sdev = (((sst - mean * mean * n) / (n - 1)) as f64).sqrt();
    println!("{name}: {mean}ns +/- {sdev:.2}");

    res
}

fn euler1_unpar(input: i32) -> i64 {
    (1..input)
        .filter(|i| i % 3 == 0 || i % 5 == 0)
        .map(|num| num as i64)
        .sum()
}

fn euler1_par(input: i32) -> i64 {
    let handle1 = thread::spawn(move || {
        (1..input / 2)
            .filter(|i| i % 3 == 0 || i % 5 == 0)
            .map(|num| num as i64)
            .sum::<i64>()
    });

    let handle2 = thread::spawn(move || {
        ((input / 2)..input)
            .filter(|i| i % 3 == 0 || i % 5 == 0)
            .map(|num| num as i64)
            .sum::<i64>()
    });

    handle1.join().unwrap() + handle2.join().unwrap()
}

fn euler1_par_chunk(input: i32, thread_count: i32) -> i64 {
    let chunk_size = input / thread_count;

    (0..thread_count)
        .map(|i| {
            let chunk_start = 1 + i * chunk_size;
            let chunk_end = if i == thread_count - 1 {
                input
            } else {
                chunk_start + chunk_size
            };
            thread::spawn(move || {
                (chunk_start..chunk_end)
                    .filter(|i| i % 3 == 0 || i % 5 == 0)
                    .map(|num| num as i64)
                    .sum::<i64>()
            })
        })
        .collect::<Vec<JoinHandle<i64>>>()
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .sum()
}

fn euler1_rayon_native(input: i32) -> i64 {
    (1..input)
        .into_par_iter()
        .filter(|i| i % 3 == 0 || i % 5 == 0)
        .map(|num| num as i64)
        .sum()
}

fn euler1_rayon(input: i32, pool: &ThreadPool) -> i64 {
    pool.install(|| {
        (1..input)
            .into_par_iter()
            .filter(|i| i % 3 == 0 || i % 5 == 0)
            .map(|num| num as i64)
            .sum()
    })
}

fn euler1_tokio(input: i32, runtime: &tokio::runtime::Runtime, thread_count: i32) -> i64 {
    runtime.block_on(async {
        let chunk_size = input / thread_count;

        let handles = (0..thread_count)
            .map(|i| {
                let chunk_start = 1 + i * chunk_size;
                let chunk_end = if i == thread_count - 1 {
                    input
                } else {
                    chunk_start + chunk_size
                };
                runtime.spawn(async move {
                    (chunk_start..chunk_end)
                        .filter(|i| i % 3 == 0 || i % 5 == 0)
                        .map(|num| num as i64)
                        .sum::<i64>()
                })
            })
            .collect::<Vec<TokioJoinHandle<i64>>>();

        let mut sum = 0;
        for handle in handles {
            sum += handle.await.unwrap();
        }

        sum
    })
}
