# Text Functions

Text analysis and processing functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`bigrams`](#bigrams) | `string -> array` | Generate word bigrams (2-grams) |
| [`char_count`](#char-count) | `string -> number` | Count characters in text |
| [`char_frequencies`](#char-frequencies) | `string -> object` | Count character frequencies |
| [`ngrams`](#ngrams) | `string, number, string? -> array` | Generate n-grams from text (word or character) |
| [`paragraph_count`](#paragraph-count) | `string -> number` | Count paragraphs in text |
| [`reading_time`](#reading-time) | `string -> string` | Estimate reading time |
| [`reading_time_seconds`](#reading-time-seconds) | `string -> number` | Estimate reading time in seconds |
| [`sentence_count`](#sentence-count) | `string -> number` | Count sentences in text |
| [`trigrams`](#trigrams) | `string -> array` | Generate word trigrams (3-grams) |
| [`word_count`](#word-count) | `string -> number` | Count words in text |
| [`word_frequencies`](#word-frequencies) | `string -> object` | Count word frequencies |

## Functions

### bigrams

Generate word bigrams (2-grams)

**Signature:** `string -> array`

**Examples:**

```text
# Basic bigrams
bigrams('a b c') -> \[\['a', 'b'\], \['b', 'c'\]\]
# Sentence bigrams
bigrams('the quick brown fox') -> \[\['the', 'quick'\], \['quick', 'brown'\], \['brown', 'fox'\]\]
# Single word
bigrams('single') -> \[\]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bigrams('a b c')'
```

### char_count

Count characters in text

**Signature:** `string -> number`

**Examples:**

```text
# Simple word
char_count('hello') -> 5
# With space
char_count('hello world') -> 11
# Empty string
char_count('') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'char_count('hello')'
```

### char_frequencies

Count character frequencies

**Signature:** `string -> object`

**Examples:**

```text
# Count repeated chars
char_frequencies('aab') -> {a: 2, b: 1}
# Word frequencies
char_frequencies('hello') -> {e: 1, h: 1, l: 2, o: 1}
# Empty string
char_frequencies('') -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'char_frequencies('aab')'
```

### ngrams

Generate n-grams from text (word or character)

**Signature:** `string, number, string? -> array`

**Examples:**

```text
# Character trigrams
ngrams('hello', `3`, 'char') -> \['hel', 'ell', 'llo'\]
# Word bigrams
ngrams('a b c d', `2`, 'word') -> \[\['a', 'b'\], \['b', 'c'\], \['c', 'd'\]\]
# Text shorter than n
ngrams('ab', `3`, 'char') -> \[\]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ngrams('hello', `3`, 'char')'
```

### paragraph_count

Count paragraphs in text

**Signature:** `string -> number`

**Examples:**

```text
# Two paragraphs
paragraph_count('A\\n\\nB') -> 2
# Single paragraph
paragraph_count('Single paragraph') -> 1
# Three paragraphs
paragraph_count('A\\n\\nB\\n\\nC') -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'paragraph_count('A\\n\\nB')'
```

### reading_time

Estimate reading time

**Signature:** `string -> string`

**Examples:**

```text
# Short text
reading_time('The quick brown fox') -> \"1 min read\"
# Empty text minimum
reading_time('') -> \"1 min read\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'reading_time('The quick brown fox')'
```

### reading_time_seconds

Estimate reading time in seconds

**Signature:** `string -> number`

**Examples:**

```text
# Short sentence
reading_time_seconds('The quick brown fox jumps over the lazy dog') -> 2
# Empty text
reading_time_seconds('') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'reading_time_seconds('The quick brown fox jumps over the lazy dog')'
```

### sentence_count

Count sentences in text

**Signature:** `string -> number`

**Examples:**

```text
# Two sentences
sentence_count('Hello. World!') -> 2
# Single sentence
sentence_count('One sentence') -> 1
# Different terminators
sentence_count('What? Yes! No.') -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sentence_count('Hello. World!')'
```

### trigrams

Generate word trigrams (3-grams)

**Signature:** `string -> array`

**Examples:**

```text
# Basic trigrams
trigrams('a b c d') -> \[\['a', 'b', 'c'\], \['b', 'c', 'd'\]\]
# Sentence trigrams
trigrams('the quick brown fox jumps') -> \[\['the', 'quick', 'brown'\], \['quick', 'brown', 'fox'\], \['brown', 'fox', 'jumps'\]\]
# Too few words
trigrams('a b') -> \[\]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'trigrams('a b c d')'
```

### word_count

Count words in text

**Signature:** `string -> number`

**Examples:**

```text
# Two words
word_count('hello world') -> 2
# Single word
word_count('one') -> 1
# Empty string
word_count('') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'word_count('hello world')'
```

### word_frequencies

Count word frequencies

**Signature:** `string -> object`

**Examples:**

```text
# Count repeated words
word_frequencies('a a b') -> {a: 2, b: 1}
# Unique words
word_frequencies('the quick brown fox') -> {brown: 1, fox: 1, quick: 1, the: 1}
# Empty string
word_frequencies('') -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'word_frequencies('a a b')'
```

