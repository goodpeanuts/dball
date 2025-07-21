use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use dball::bench::{has_duplicates_hashset, has_duplicates_sort_in_place};
use rand::{Rng, SeedableRng, rngs::StdRng};

/// 生成完全唯一的数据：0..n-1 的顺序，打乱 (shuffle)
fn gen_unique_vec(n: usize, rng: &mut StdRng) -> Vec<u32> {
    let mut v: Vec<u32> = (0..n as u32).collect();
    // 简单 Fisher-Yates 洗牌
    for i in (1..v.len()).rev() {
        let j = rng.gen_range(0..=i);
        v.swap(i, j);
    }
    v
}

/// 生成带重复的数据：先生成唯一，再克隆一部分元素追加形成重复
fn gen_with_dups_vec(n: usize, dup_fraction: f64, rng: &mut StdRng) -> Vec<u32> {
    assert!((0.0..=1.0).contains(&dup_fraction));
    let mut base = gen_unique_vec(n, rng);
    let dup_count = (n as f64 * dup_fraction).round() as usize;
    if dup_count > 0 {
        // 从 base 中随机抽取 dup_count 个元素再 push
        for _ in 0..dup_count {
            let idx = rng.gen_range(0..base.len());
            let val = base[idx];
            base.push(val);
        }
    }
    base
}

fn bench_duplicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("duplicate_detection");

    // 你可以调整这些规模
    let sizes = [1_000, 10_000, 100_000];
    // 重复比例（相对前 n 个唯一元素数）：例如 5% 的重复量
    let dup_fraction = 0.05;

    for &n in &sizes {
        // 为了结果稳定：每个规模重新建一个 RNG（固定种子 + n）
        let mut rng = StdRng::seed_from_u64(n as u64);

        // 数据：全唯一
        let data_unique = gen_unique_vec(n, &mut rng);

        // 数据：含重复
        let mut rng2 = StdRng::seed_from_u64((n as u64) ^ 0xDEADBEEF);
        let data_dups = gen_with_dups_vec(n, dup_fraction, &mut rng2);

        // --------- HashSet: unique ----------
        group.bench_with_input(
            BenchmarkId::new("hashset/all_unique", n),
            &data_unique,
            |b, data| {
                b.iter(|| {
                    let has_dup = has_duplicates_hashset(black_box(data));
                    black_box(has_dup);
                })
            },
        );

        // --------- HashSet: with_dups ----------
        group.bench_with_input(
            BenchmarkId::new("hashset/with_dups", n),
            &data_dups,
            |b, data| {
                b.iter(|| {
                    let has_dup = has_duplicates_hashset(black_box(data));
                    black_box(has_dup);
                })
            },
        );

        // --------- sort_in_place: all_unique ----------
        // 使用 batched 方式，每次基准迭代都 clone 原始数据（因为 sort 会修改）
        group.bench_with_input(
            BenchmarkId::new("sort/all_unique", n),
            &data_unique,
            |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut v| {
                        let has_dup = has_duplicates_sort_in_place(black_box(&mut v));
                        black_box(has_dup);
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        // --------- sort_in_place: with_dups ----------
        group.bench_with_input(
            BenchmarkId::new("sort/with_dups", n),
            &data_dups,
            |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut v| {
                        let has_dup = has_duplicates_sort_in_place(black_box(&mut v));
                        black_box(has_dup);
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_duplicates);
criterion_main!(benches);
