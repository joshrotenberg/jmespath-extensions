use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;

fn create_runtime() -> Runtime {
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    register_all(&mut runtime);
    runtime
}

fn bench_string_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("string");

    // Simple string operations
    let data = Variable::String("hello world".to_string());

    let expr = runtime.compile("upper(@)").unwrap();
    group.bench_function("upper", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("lower(@)").unwrap();
    group.bench_function("lower", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("split(@, ' ')").unwrap();
    group.bench_function("split", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("replace(@, 'o', 'a')").unwrap();
    group.bench_function("replace", |b| b.iter(|| expr.search(black_box(&data))));

    // Case conversion
    let data = Variable::String("hello_world_test".to_string());
    let expr = runtime.compile("camel_case(@)").unwrap();
    group.bench_function("camel_case", |b| b.iter(|| expr.search(black_box(&data))));

    group.finish();
}

fn bench_array_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("array");

    // Small array
    let small = Variable::from_json("[1, 2, 3, 4, 5]").unwrap();

    let expr = runtime.compile("unique(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("unique", "5"), &small, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    let expr = runtime.compile("first(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("first", "5"), &small, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // Medium array
    let medium: Vec<i32> = (0..100).collect();
    let medium = Variable::from_json(&serde_json::to_string(&medium).unwrap()).unwrap();

    let expr = runtime.compile("unique(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("unique", "100"), &medium, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    let expr = runtime.compile("chunk(@, `10`)").unwrap();
    group.bench_with_input(BenchmarkId::new("chunk", "100"), &medium, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // Large array
    let large: Vec<i32> = (0..1000).collect();
    let large = Variable::from_json(&serde_json::to_string(&large).unwrap()).unwrap();

    let expr = runtime.compile("unique(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("unique", "1000"), &large, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    group.finish();
}

fn bench_math_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("math");

    let data = Variable::from_json("3.14159").unwrap();

    let expr = runtime.compile("round(@, `2`)").unwrap();
    group.bench_function("round", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("sqrt(@)").unwrap();
    group.bench_function("sqrt", |b| b.iter(|| expr.search(black_box(&data))));

    // Statistics on array
    let numbers: Vec<f64> = (0..100).map(|x| x as f64).collect();
    let arr = Variable::from_json(&serde_json::to_string(&numbers).unwrap()).unwrap();

    let expr = runtime.compile("median(@)").unwrap();
    group.bench_function("median/100", |b| b.iter(|| expr.search(black_box(&arr))));

    let expr = runtime.compile("stddev(@)").unwrap();
    group.bench_function("stddev/100", |b| b.iter(|| expr.search(black_box(&arr))));

    group.finish();
}

#[cfg(feature = "hash")]
fn bench_hash_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("hash");

    let data = Variable::String("hello world".to_string());

    let expr = runtime.compile("md5(@)").unwrap();
    group.bench_function("md5", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("sha256(@)").unwrap();
    group.bench_function("sha256", |b| b.iter(|| expr.search(black_box(&data))));

    // Larger input
    let large = Variable::String("x".repeat(10000));

    let expr = runtime.compile("sha256(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("sha256", "10KB"), &large, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    group.finish();
}

#[cfg(feature = "fuzzy")]
fn bench_fuzzy_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("fuzzy");

    let data = Variable::from_json(r#"["kitten", "sitting"]"#).unwrap();

    let expr = runtime.compile("levenshtein(@[0], @[1])").unwrap();
    group.bench_function("levenshtein", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("jaro_winkler(@[0], @[1])").unwrap();
    group.bench_function("jaro_winkler", |b| b.iter(|| expr.search(black_box(&data))));

    group.finish();
}

#[cfg(feature = "phonetic")]
fn bench_phonetic_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("phonetic");

    let data = Variable::String("Robert".to_string());

    let expr = runtime.compile("soundex(@)").unwrap();
    group.bench_function("soundex", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("metaphone(@)").unwrap();
    group.bench_function("metaphone", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("double_metaphone(@)").unwrap();
    group.bench_function("double_metaphone", |b| {
        b.iter(|| expr.search(black_box(&data)))
    });

    group.finish();
}

#[cfg(feature = "geo")]
fn bench_geo_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("geo");

    // NYC to London coordinates
    let data = Variable::from_json(r#"[40.7128, -74.0060, 51.5074, -0.1278]"#).unwrap();

    let expr = runtime
        .compile("haversine(@[0], @[1], @[2], @[3])")
        .unwrap();
    group.bench_function("haversine", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("bearing(@[0], @[1], @[2], @[3])").unwrap();
    group.bench_function("bearing", |b| b.iter(|| expr.search(black_box(&data))));

    group.finish();
}

#[cfg(feature = "expression")]
fn bench_expression_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("expression");

    // Array of objects
    let data = Variable::from_json(
        r#"[
        {"name": "alice", "age": 30},
        {"name": "bob", "age": 25},
        {"name": "carol", "age": 35}
    ]"#,
    )
    .unwrap();

    let expr = runtime.compile("map_expr('name', @)").unwrap();
    group.bench_function("map_expr", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("filter_expr('age > `28`', @)").unwrap();
    group.bench_function("filter_expr", |b| b.iter(|| expr.search(black_box(&data))));

    let expr = runtime.compile("sort_by_expr('age', @)").unwrap();
    group.bench_function("sort_by_expr", |b| b.iter(|| expr.search(black_box(&data))));

    // Larger dataset
    let large_data: Vec<serde_json::Value> = (0..100)
        .map(|i| serde_json::json!({"name": format!("user{}", i), "age": i % 50 + 20}))
        .collect();
    let large = Variable::from_json(&serde_json::to_string(&large_data).unwrap()).unwrap();

    let expr = runtime.compile("filter_expr('age > `40`', @)").unwrap();
    group.bench_with_input(BenchmarkId::new("filter_expr", "100"), &large, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    group.finish();
}

#[cfg(feature = "text")]
fn bench_text_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("text");

    let short = Variable::String("Hello world, this is a test.".to_string());
    let long = Variable::String("Lorem ipsum dolor sit amet. ".repeat(100));

    let expr = runtime.compile("word_count(@)").unwrap();
    group.bench_with_input(
        BenchmarkId::new("word_count", "short"),
        &short,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    let expr = runtime.compile("word_count(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("word_count", "long"), &long, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    let expr = runtime.compile("word_frequencies(@)").unwrap();
    group.bench_with_input(
        BenchmarkId::new("word_frequencies", "short"),
        &short,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    group.finish();
}

// Compile-time registration benchmark
fn bench_registration(c: &mut Criterion) {
    c.bench_function("register_all", |b| {
        b.iter(|| {
            let mut runtime = Runtime::new();
            runtime.register_builtin_functions();
            register_all(&mut runtime);
            black_box(runtime)
        })
    });
}

// Core benchmark groups (always available)
criterion_group!(
    core_benches,
    bench_string_functions,
    bench_array_functions,
    bench_math_functions,
    bench_registration
);

#[cfg(feature = "hash")]
criterion_group!(hash_benches, bench_hash_functions);

#[cfg(feature = "fuzzy")]
criterion_group!(fuzzy_benches, bench_fuzzy_functions);

#[cfg(feature = "phonetic")]
criterion_group!(phonetic_benches, bench_phonetic_functions);

#[cfg(feature = "geo")]
criterion_group!(geo_benches, bench_geo_functions);

#[cfg(feature = "expression")]
criterion_group!(expression_benches, bench_expression_functions);

#[cfg(feature = "text")]
criterion_group!(text_benches, bench_text_functions);

// Full feature set
#[cfg(all(
    feature = "hash",
    feature = "fuzzy",
    feature = "phonetic",
    feature = "geo",
    feature = "expression",
    feature = "text"
))]
criterion_main!(
    core_benches,
    hash_benches,
    fuzzy_benches,
    phonetic_benches,
    geo_benches,
    expression_benches,
    text_benches
);

// Fallback for minimal feature sets
#[cfg(not(all(
    feature = "hash",
    feature = "fuzzy",
    feature = "phonetic",
    feature = "geo",
    feature = "expression",
    feature = "text"
)))]
criterion_main!(core_benches);
