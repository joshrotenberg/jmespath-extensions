use anyhow::{Context, Result};
use clap::Parser;
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;
use std::io::{self, Read};

/// JMESPath CLI with extended functions
///
/// A command-line tool for querying JSON data using JMESPath expressions
/// with 150+ additional functions beyond the standard specification.
#[derive(Parser, Debug)]
#[command(name = "jpx", version, about, long_about = None)]
struct Args {
    /// JMESPath expression to evaluate
    expression: Option<String>,

    /// Input file (reads from stdin if not provided)
    #[arg(short, long)]
    file: Option<String>,

    /// Output raw strings without quotes
    #[arg(short = 'r', long)]
    raw: bool,

    /// Compact output (no pretty printing)
    #[arg(short, long)]
    compact: bool,

    /// List available extension functions
    #[arg(long)]
    list_functions: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.list_functions {
        print_functions();
        return Ok(());
    }

    let expression = args
        .expression
        .ok_or_else(|| anyhow::anyhow!("Expression required. Use --help for usage."))?;

    // Read input JSON
    let input = match &args.file {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?,
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            buf
        }
    };

    // Parse JSON
    let data = Variable::from_json(&input)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON input: {}", e))?;

    // Create runtime with extensions
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    register_all(&mut runtime);

    // Compile and execute expression
    let expr = runtime
        .compile(&expression)
        .with_context(|| format!("Failed to compile expression: {}", expression))?;

    let result = expr
        .search(&data)
        .context("Failed to evaluate expression")?;

    // Output result
    if result.is_null() {
        // Don't print anything for null results (like jq)
        return Ok(());
    }

    if args.raw {
        if let Some(s) = result.as_string() {
            println!("{}", s);
            return Ok(());
        }
    }

    let output = if args.compact {
        serde_json::to_string(&*result)?
    } else {
        serde_json::to_string_pretty(&*result)?
    };

    println!("{}", output);
    Ok(())
}

fn print_functions() {
    println!("jpx - JMESPath with Extended Functions\n");
    println!("Standard JMESPath functions (26):");
    println!("  abs, avg, ceil, contains, ends_with, floor, join, keys, length,");
    println!("  map, max, max_by, merge, min, min_by, not_null, reverse, sort,");
    println!("  sort_by, starts_with, sum, to_array, to_number, to_string, type, values\n");

    println!("Extension functions by category:\n");

    println!("STRING: upper, lower, trim, trim_left, trim_right, capitalize, title,");
    println!("        split, replace, repeat, pad_left, pad_right, substr, slice,");
    println!("        find_first, find_last, concat, camel_case, snake_case, kebab_case, wrap\n");

    println!("ARRAY: first, last, unique, take, drop, chunk, zip, flatten_deep,");
    println!("       compact, range, index_at, includes, find_index, difference,");
    println!("       intersection, union, group_by, frequencies\n");

    println!("OBJECT: items, from_items, pick, omit, deep_merge, invert,");
    println!("        rename_keys, flatten_keys\n");

    println!("MATH: round, floor_fn, ceil_fn, abs_fn, mod_fn, pow, sqrt, log,");
    println!("      clamp, median, percentile, variance, stddev, sin, cos, tan\n");

    println!("TYPE: to_string, to_number, to_boolean, type_of, is_string,");
    println!("      is_number, is_boolean, is_array, is_object, is_null\n");

    println!("UTILITY: now, now_ms, default, if, coalesce\n");

    println!("DATETIME: now, now_millis, parse_date, format_date, date_add, date_diff\n");

    println!("EXPRESSION: map_expr, filter_expr, any_expr, all_expr, find_expr,");
    println!("            find_index_expr, count_expr, sort_by_expr, group_by_expr,");
    println!(
        "            partition_expr, min_by_expr, max_by_expr, unique_by_expr, flat_map_expr\n"
    );

    println!("HASH: md5, sha1, sha256, crc32\n");

    println!("ENCODING: base64_encode, base64_decode, hex_encode, hex_decode\n");

    println!("REGEX: regex_match, regex_extract, regex_replace\n");

    println!("URL: url_encode, url_decode, url_parse\n");

    println!("VALIDATION: is_email, is_url, is_uuid, is_ipv4, is_ipv6\n");

    println!("PATH: path_basename, path_dirname, path_ext, path_join\n");

    println!("RANDOM: random, uuid, shuffle, sample\n");

    println!("FUZZY: levenshtein, normalized_levenshtein, damerau_levenshtein,");
    println!("       jaro, jaro_winkler, sorensen_dice\n");

    println!("PHONETIC: soundex, metaphone, double_metaphone, nysiis,");
    println!(
        "          match_rating_codex, caverphone, caverphone2, sounds_like, phonetic_match\n"
    );

    println!("GEO: haversine, haversine_km, haversine_mi, bearing\n");

    println!("SEMVER: semver_parse, semver_major, semver_minor, semver_patch,");
    println!("        semver_compare, semver_matches, is_semver\n");

    println!("NETWORK: ip_to_int, int_to_ip, cidr_contains, cidr_network,");
    println!("         cidr_broadcast, cidr_prefix, is_private_ip\n");

    println!("IDS: nanoid, ulid, ulid_timestamp\n");

    println!("TEXT: word_count, char_count, sentence_count, paragraph_count,");
    println!("      reading_time, reading_time_seconds, char_frequencies, word_frequencies\n");

    println!("DURATION: parse_duration, format_duration, duration_hours,");
    println!("          duration_minutes, duration_seconds\n");

    println!("COLOR: hex_to_rgb, rgb_to_hex, lighten, darken, color_mix,");
    println!("       color_invert, color_grayscale, color_complement\n");

    println!("COMPUTING: parse_bytes, format_bytes, format_bytes_binary,");
    println!("           bit_and, bit_or, bit_xor, bit_not, bit_shift_left, bit_shift_right\n");

    println!("For full documentation: https://docs.rs/jmespath_extensions");
}
