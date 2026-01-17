use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;
use jmespath_extensions::registry::FunctionRegistry;

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

    // Trim operations
    let padded = Variable::String("   hello world   ".to_string());
    let expr = runtime.compile("trim(@)").unwrap();
    group.bench_function("trim", |b| b.iter(|| expr.search(black_box(&padded))));

    // Pad operations
    let short = Variable::String("hello".to_string());
    let expr = runtime.compile("pad_left(@, `20`, ' ')").unwrap();
    group.bench_function("pad_left", |b| b.iter(|| expr.search(black_box(&short))));

    let expr = runtime.compile("pad_right(@, `20`, ' ')").unwrap();
    group.bench_function("pad_right", |b| b.iter(|| expr.search(black_box(&short))));

    group.finish();
}

#[cfg(feature = "regex")]
fn bench_regex_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("regex");

    let email = Variable::String("user@example.com".to_string());

    let expr = runtime
        .compile(r#"regex_match(@, `"^[a-z]+@[a-z]+\\.[a-z]+$"`)"#)
        .unwrap();
    group.bench_function("regex_match/simple", |b| {
        b.iter(|| expr.search(black_box(&email)))
    });

    let text = Variable::String("The quick brown fox jumps over the lazy dog".to_string());
    let expr = runtime
        .compile(r#"regex_replace(@, `"[aeiou]"`, `"*"`)"#)
        .unwrap();
    group.bench_function("regex_replace", |b| {
        b.iter(|| expr.search(black_box(&text)))
    });

    // Larger text
    let large_text = Variable::String("Hello world! ".repeat(100));
    let expr = runtime.compile(r#"regex_match(@, `"world"`)"#).unwrap();
    group.bench_with_input(
        BenchmarkId::new("regex_match", "large"),
        &large_text,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

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

    // Clojure-inspired functions
    // dedupe - array with consecutive duplicates
    let with_dupes = Variable::from_json("[1, 1, 2, 2, 2, 3, 3, 1, 1, 4, 4, 4, 4, 5]").unwrap();
    let expr = runtime.compile("dedupe(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("dedupe", "14"), &with_dupes, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // dedupe on larger array with pattern
    let large_dupes: Vec<i32> = (0..100).flat_map(|x| vec![x, x, x]).collect();
    let large_dupes = Variable::from_json(&serde_json::to_string(&large_dupes).unwrap()).unwrap();
    let expr = runtime.compile("dedupe(@)").unwrap();
    group.bench_with_input(
        BenchmarkId::new("dedupe", "300"),
        &large_dupes,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    // interpose
    let expr = runtime.compile("interpose(@, `0`)").unwrap();
    group.bench_with_input(BenchmarkId::new("interpose", "100"), &medium, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // butlast
    let expr = runtime.compile("butlast(@)").unwrap();
    group.bench_with_input(BenchmarkId::new("butlast", "100"), &medium, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // zipmap
    let keys: Vec<String> = (0..50).map(|i| format!("key{}", i)).collect();
    let values: Vec<i32> = (0..50).collect();
    let zipmap_data = Variable::from_json(&format!(
        r#"{{"keys": {}, "values": {}}}"#,
        serde_json::to_string(&keys).unwrap(),
        serde_json::to_string(&values).unwrap()
    ))
    .unwrap();
    let expr = runtime.compile("zipmap(keys, values)").unwrap();
    group.bench_with_input(BenchmarkId::new("zipmap", "50"), &zipmap_data, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // partition_by - array of objects
    let objects: Vec<serde_json::Value> = (0..100)
        .map(|i| serde_json::json!({"type": format!("t{}", i / 10), "value": i}))
        .collect();
    let objects = Variable::from_json(&serde_json::to_string(&objects).unwrap()).unwrap();
    let expr = runtime.compile(r#"partition_by(@, `"type"`)"#).unwrap();
    group.bench_with_input(
        BenchmarkId::new("partition_by", "100"),
        &objects,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    // flatten_deep - nested arrays
    let nested: Vec<serde_json::Value> = (0..20)
        .map(|i| serde_json::json!([[i, i + 1], [i + 2, [i + 3, i + 4]]]))
        .collect();
    let nested = Variable::from_json(&serde_json::to_string(&nested).unwrap()).unwrap();
    let expr = runtime.compile("flatten_deep(@)").unwrap();
    group.bench_with_input(
        BenchmarkId::new("flatten_deep", "nested"),
        &nested,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    // group_by - array of objects
    let users: Vec<serde_json::Value> = (0..100)
        .map(
            |i| serde_json::json!({"role": format!("role{}", i % 5), "name": format!("user{}", i)}),
        )
        .collect();
    let users = Variable::from_json(&serde_json::to_string(&users).unwrap()).unwrap();
    let expr = runtime.compile(r#"group_by(@, `"role"`)"#).unwrap();
    group.bench_with_input(BenchmarkId::new("group_by", "100"), &users, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // Set operations - intersection, difference, union
    let arr1: Vec<i32> = (0..100).collect();
    let arr2: Vec<i32> = (50..150).collect();
    let sets = Variable::from_json(&format!(
        r#"{{"a": {}, "b": {}}}"#,
        serde_json::to_string(&arr1).unwrap(),
        serde_json::to_string(&arr2).unwrap()
    ))
    .unwrap();

    let expr = runtime.compile("intersection(a, b)").unwrap();
    group.bench_with_input(BenchmarkId::new("intersection", "100"), &sets, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    let expr = runtime.compile("difference(a, b)").unwrap();
    group.bench_with_input(BenchmarkId::new("difference", "100"), &sets, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    let expr = runtime.compile("union(a, b)").unwrap();
    group.bench_with_input(BenchmarkId::new("union", "100"), &sets, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // frequencies
    let with_repeats: Vec<i32> = (0..100).flat_map(|x| vec![x % 10; 3]).collect();
    let with_repeats = Variable::from_json(&serde_json::to_string(&with_repeats).unwrap()).unwrap();
    let expr = runtime.compile("frequencies(@)").unwrap();
    group.bench_with_input(
        BenchmarkId::new("frequencies", "300"),
        &with_repeats,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    // repeat_array and cycle (Phase 3 functions)
    let expr = runtime.compile("repeat_array(`1`, `100`)").unwrap();
    let null_data = Variable::Null;
    group.bench_with_input(
        BenchmarkId::new("repeat_array", "100"),
        &null_data,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    let small_arr = Variable::from_json("[1, 2, 3]").unwrap();
    let expr = runtime.compile("cycle(@, `50`)").unwrap();
    group.bench_with_input(BenchmarkId::new("cycle", "150"), &small_arr, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    group.finish();
}

fn bench_object_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("object");

    // deep_merge - nested objects
    let obj = Variable::from_json(r#"{"a": {"b": {"c": 1}}, "d": {"e": 2}, "f": 3}"#).unwrap();
    let expr = runtime
        .compile(r#"deep_merge(@, `{"a": {"b": {"x": 9}}, "d": {"y": 8}}`)"#)
        .unwrap();
    group.bench_function("deep_merge", |b| b.iter(|| expr.search(black_box(&obj))));

    // flatten_keys - nested object
    let nested_obj =
        Variable::from_json(r#"{"a": {"b": {"c": 1, "d": 2}, "e": 3}, "f": {"g": 4}}"#).unwrap();
    let expr = runtime.compile("flatten_keys(@)").unwrap();
    group.bench_function("flatten_keys", |b| {
        b.iter(|| expr.search(black_box(&nested_obj)))
    });

    // unflatten_keys
    let flat_obj = Variable::from_json(r#"{"a.b.c": 1, "a.b.d": 2, "a.e": 3, "f.g": 4}"#).unwrap();
    let expr = runtime.compile("unflatten_keys(@)").unwrap();
    group.bench_function("unflatten_keys", |b| {
        b.iter(|| expr.search(black_box(&flat_obj)))
    });

    // pick - select specific keys
    let large_obj: serde_json::Map<String, serde_json::Value> = (0..50)
        .map(|i| (format!("key{}", i), serde_json::json!(i)))
        .collect();
    let large_obj = Variable::from_json(&serde_json::to_string(&large_obj).unwrap()).unwrap();
    let expr = runtime
        .compile(r#"pick(@, ['key1', 'key5', 'key10', 'key20', 'key30'])"#)
        .unwrap();
    group.bench_with_input(BenchmarkId::new("pick", "50"), &large_obj, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    // omit - exclude specific keys
    let expr = runtime
        .compile(r#"omit(@, ['key1', 'key5', 'key10', 'key20', 'key30'])"#)
        .unwrap();
    group.bench_with_input(BenchmarkId::new("omit", "50"), &large_obj, |b, data| {
        b.iter(|| expr.search(black_box(data)))
    });

    group.finish();
}

#[cfg(feature = "datetime")]
fn bench_datetime_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("datetime");

    // parse_date
    let date_str = Variable::String("2024-01-15".to_string());
    let expr = runtime.compile("parse_date(@, '%Y-%m-%d')").unwrap();
    group.bench_function("parse_date", |b| {
        b.iter(|| expr.search(black_box(&date_str)))
    });

    // format_date
    let timestamp = Variable::from_json("1705276800").unwrap();
    let expr = runtime.compile("format_date(@, '%Y-%m-%d')").unwrap();
    group.bench_function("format_date", |b| {
        b.iter(|| expr.search(black_box(&timestamp)))
    });

    // date_diff
    let dates = Variable::from_json(r#"[1705276800, 1704067200]"#).unwrap();
    let expr = runtime.compile("date_diff(@[0], @[1], 'days')").unwrap();
    group.bench_function("date_diff", |b| b.iter(|| expr.search(black_box(&dates))));

    // relative_time
    let expr = runtime.compile("relative_time(@)").unwrap();
    group.bench_function("relative_time", |b| {
        b.iter(|| expr.search(black_box(&timestamp)))
    });

    group.finish();
}

#[cfg(feature = "path")]
fn bench_path_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("path");

    let nested = Variable::from_json(r#"{"a": {"b": {"c": {"d": {"e": 1}}}}}"#).unwrap();

    // get_path - deep access
    let expr = runtime.compile(r#"get_path(@, `"a.b.c.d.e"`)"#).unwrap();
    group.bench_function("get_path/deep", |b| {
        b.iter(|| expr.search(black_box(&nested)))
    });

    // set_path
    let expr = runtime
        .compile(r#"set_path(@, `"a.b.c.d.e"`, `999`)"#)
        .unwrap();
    group.bench_function("set_path/deep", |b| {
        b.iter(|| expr.search(black_box(&nested)))
    });

    // has_path
    let expr = runtime.compile(r#"has_path(@, `"a.b.c.d.e"`)"#).unwrap();
    group.bench_function("has_path/deep", |b| {
        b.iter(|| expr.search(black_box(&nested)))
    });

    group.finish();
}

#[cfg(feature = "encoding")]
fn bench_encoding_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("encoding");

    let data = Variable::String("Hello, World! This is a test string.".to_string());

    let expr = runtime.compile("base64_encode(@)").unwrap();
    group.bench_function("base64_encode", |b| {
        b.iter(|| expr.search(black_box(&data)))
    });

    let encoded = Variable::String("SGVsbG8sIFdvcmxkISBUaGlzIGlzIGEgdGVzdCBzdHJpbmcu".to_string());
    let expr = runtime.compile("base64_decode(@)").unwrap();
    group.bench_function("base64_decode", |b| {
        b.iter(|| expr.search(black_box(&encoded)))
    });

    // JWT decode (without verification)
    let jwt = Variable::String(
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string(),
    );
    let expr = runtime.compile("jwt_decode(@)").unwrap();
    group.bench_function("jwt_decode", |b| b.iter(|| expr.search(black_box(&jwt))));

    group.finish();
}

#[cfg(feature = "validation")]
fn bench_validation_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("validation");

    let email = Variable::String("user@example.com".to_string());
    let expr = runtime.compile("is_email(@)").unwrap();
    group.bench_function("is_email", |b| b.iter(|| expr.search(black_box(&email))));

    let url = Variable::String("https://example.com/path?query=value".to_string());
    let expr = runtime.compile("is_url(@)").unwrap();
    group.bench_function("is_url", |b| b.iter(|| expr.search(black_box(&url))));

    let uuid = Variable::String("550e8400-e29b-41d4-a716-446655440000".to_string());
    let expr = runtime.compile("is_uuid(@)").unwrap();
    group.bench_function("is_uuid", |b| b.iter(|| expr.search(black_box(&uuid))));

    let ip = Variable::String("192.168.1.1".to_string());
    let expr = runtime.compile("is_ipv4(@)").unwrap();
    group.bench_function("is_ipv4", |b| b.iter(|| expr.search(black_box(&ip))));

    group.finish();
}

fn bench_type_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("type");

    let string = Variable::String("42".to_string());
    let expr = runtime.compile("to_number(@)").unwrap();
    group.bench_function("to_number", |b| b.iter(|| expr.search(black_box(&string))));

    let number = Variable::from_json("42").unwrap();
    let expr = runtime.compile("to_string(@)").unwrap();
    group.bench_function("to_string", |b| b.iter(|| expr.search(black_box(&number))));

    let expr = runtime.compile("type_of(@)").unwrap();
    group.bench_function("type_of/number", |b| {
        b.iter(|| expr.search(black_box(&number)))
    });

    let arr = Variable::from_json("[1, 2, 3]").unwrap();
    group.bench_function("type_of/array", |b| b.iter(|| expr.search(black_box(&arr))));

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

#[cfg(feature = "multi-match")]
fn bench_multi_match_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("multi-match");

    let text = Variable::String("The quick brown fox jumps over the lazy dog".to_string());

    let expr = runtime
        .compile("match_any(@, ['fox', 'cat', 'dog'])")
        .unwrap();
    group.bench_function("match_any", |b| b.iter(|| expr.search(black_box(&text))));

    let expr = runtime
        .compile("match_all(@, ['quick', 'fox', 'dog'])")
        .unwrap();
    group.bench_function("match_all", |b| b.iter(|| expr.search(black_box(&text))));

    let expr = runtime
        .compile("match_which(@, ['quick', 'slow', 'fox', 'cat'])")
        .unwrap();
    group.bench_function("match_which", |b| b.iter(|| expr.search(black_box(&text))));

    let expr = runtime.compile("match_count(@, ['o', 'e', 'a'])").unwrap();
    group.bench_function("match_count", |b| b.iter(|| expr.search(black_box(&text))));

    // Larger text
    let large_text = Variable::String("The quick brown fox. ".repeat(100));
    let expr = runtime
        .compile("match_count(@, ['quick', 'fox', 'the'])")
        .unwrap();
    group.bench_with_input(
        BenchmarkId::new("match_count", "large"),
        &large_text,
        |b, data| b.iter(|| expr.search(black_box(data))),
    );

    group.finish();
}

#[cfg(feature = "jsonpatch")]
fn bench_jsonpatch_functions(c: &mut Criterion) {
    let runtime = create_runtime();
    let mut group = c.benchmark_group("jsonpatch");

    let data = Variable::from_json(r#"{"a": 1, "b": {"c": 2}}"#).unwrap();

    let expr = runtime
        .compile("json_patch(@, `[{\"op\": \"add\", \"path\": \"/d\", \"value\": 3}]`)")
        .unwrap();
    group.bench_function("json_patch/add", |b| {
        b.iter(|| expr.search(black_box(&data)))
    });

    let expr = runtime
        .compile("json_merge_patch(@, `{\"b\": {\"d\": 4}}`)")
        .unwrap();
    group.bench_function("json_merge_patch", |b| {
        b.iter(|| expr.search(black_box(&data)))
    });

    // Diff two objects
    let pair = Variable::from_json(r#"[{"a": 1, "b": 2}, {"a": 1, "b": 3, "c": 4}]"#).unwrap();
    let expr = runtime.compile("json_diff(@[0], @[1])").unwrap();
    group.bench_function("json_diff", |b| b.iter(|| expr.search(black_box(&pair))));

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

    // Registry-based registration
    c.bench_function("registry_register_all", |b| {
        b.iter(|| {
            let mut registry = FunctionRegistry::new();
            registry.register_all();
            let mut runtime = Runtime::new();
            runtime.register_builtin_functions();
            registry.apply(&mut runtime);
            black_box(runtime)
        })
    });

    // Registry creation and introspection
    c.bench_function("registry_create_all", |b| {
        b.iter(|| {
            let mut registry = FunctionRegistry::new();
            registry.register_all();
            black_box(registry)
        })
    });

    // Registry introspection
    c.bench_function("registry_introspection", |b| {
        let mut registry = FunctionRegistry::new();
        registry.register_all();
        b.iter(|| {
            let count: usize = registry.functions().count();
            black_box(count)
        })
    });
}

// Core benchmark groups (always available)
criterion_group!(
    core_benches,
    bench_string_functions,
    bench_array_functions,
    bench_object_functions,
    bench_type_functions,
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

#[cfg(feature = "multi-match")]
criterion_group!(multi_match_benches, bench_multi_match_functions);

#[cfg(feature = "jsonpatch")]
criterion_group!(jsonpatch_benches, bench_jsonpatch_functions);

#[cfg(feature = "regex")]
criterion_group!(regex_benches, bench_regex_functions);

#[cfg(feature = "datetime")]
criterion_group!(datetime_benches, bench_datetime_functions);

#[cfg(feature = "path")]
criterion_group!(path_benches, bench_path_functions);

#[cfg(feature = "encoding")]
criterion_group!(encoding_benches, bench_encoding_functions);

#[cfg(feature = "validation")]
criterion_group!(validation_benches, bench_validation_functions);

// Full feature set
#[cfg(all(
    feature = "hash",
    feature = "fuzzy",
    feature = "phonetic",
    feature = "geo",
    feature = "expression",
    feature = "text",
    feature = "multi-match",
    feature = "jsonpatch",
    feature = "regex",
    feature = "datetime",
    feature = "path",
    feature = "encoding",
    feature = "validation"
))]
criterion_main!(
    core_benches,
    hash_benches,
    fuzzy_benches,
    phonetic_benches,
    geo_benches,
    expression_benches,
    text_benches,
    multi_match_benches,
    jsonpatch_benches,
    regex_benches,
    datetime_benches,
    path_benches,
    encoding_benches,
    validation_benches
);

// Fallback for minimal feature sets
#[cfg(not(all(
    feature = "hash",
    feature = "fuzzy",
    feature = "phonetic",
    feature = "geo",
    feature = "expression",
    feature = "text",
    feature = "multi-match",
    feature = "jsonpatch",
    feature = "regex",
    feature = "datetime",
    feature = "path",
    feature = "encoding",
    feature = "validation"
)))]
criterion_main!(core_benches);
