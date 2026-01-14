# MCP Server Overview

jpx can run as an [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server, allowing AI assistants like Claude to use JMESPath for JSON querying and transformation.

## What is MCP?

MCP is an open protocol that enables AI assistants to interact with external tools and data sources. By running jpx as an MCP server, you give Claude (and other MCP-compatible assistants) the ability to:

- Query JSON data using JMESPath expressions
- Transform and manipulate JSON structures
- Use all 320+ extension functions
- Explore available functions and their documentation

## Why Use jpx with Claude?

When working with JSON data in Claude, jpx provides:

- **Precise queries**: Extract exactly the data you need
- **Complex transformations**: Reshape data structures on the fly
- **Powerful functions**: String manipulation, math, dates, hashing, and more
- **Consistent results**: Deterministic query execution

## Available Tools

The MCP server exposes 12 tools:

| Tool | Description |
|------|-------------|
| `evaluate` | Run JMESPath expressions against JSON input |
| `evaluate_file` | Query JSON files directly from disk |
| `batch_evaluate` | Run multiple expressions against the same input |
| `format` | Pretty-print JSON with configurable indentation |
| `diff` | Generate RFC 6902 JSON Patch between documents |
| `patch` | Apply RFC 6902 JSON Patch operations |
| `merge` | Apply RFC 7396 JSON Merge Patch |
| `keys` | Extract object keys (optionally recursive) |
| `functions` | List available functions |
| `describe` | Get detailed info for a specific function |
| `categories` | List all function categories |
| `validate` | Check expression syntax without executing |

## Getting Started

See [Setup with Claude](./setup-claude.md) to configure jpx as an MCP server for Claude Desktop.
