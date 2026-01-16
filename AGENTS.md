# Agent Guidelines for jpx

This document provides guidance for AI agents using jpx as an MCP server.

## Overview

jpx is a JMESPath processor with 400+ extension functions. As an MCP server, it provides tools for:

- Querying and transforming JSON data
- Discovering JMESPath functions
- Semantic search across multiple MCP servers

## Quick Start

### JSON Queries

Use `evaluate` for JMESPath queries:

```
mcp__jpx__evaluate({
  expression: "users[?active].name",
  json: {"users": [...]}
})
```

### Finding Functions

Use `search` to find relevant functions:

```
mcp__jpx__search({ query: "string padding" })
```

Use `describe` for detailed documentation:

```
mcp__jpx__describe({ function: "pad_left" })
```

## Multi-Server Tool Discovery

When connected to jpx alongside other MCP servers (e.g., redisctl, github-mcp), you can enable semantic search across all tools.

### Registration Pattern

At session start, register each server's tools with jpx:

```
// For each non-jpx MCP server:
mcp__jpx__register_discovery({
  spec: {
    server: { name: "server-name", version: "1.0.0" },
    tools: [
      { 
        name: "tool_name", 
        description: "What this tool does",
        category: "optional-category",
        tags: ["optional", "tags"]
      }
      // ... more tools
    ]
  }
})
```

### Searching

After registration, use semantic search:

```
// Natural language search
mcp__jpx__query_tools({ query: "backup database", top_k: 5 })

// Find similar tools
mcp__jpx__similar_tools({ tool_id: "server:tool_name", top_k: 5 })

// Browse by category
mcp__jpx__list_discovery_categories()
```

### Recommended System Prompt Addition

Add this to your agent configuration for automatic discovery:

```markdown
## MCP Tool Discovery

At session start, if jpx and other MCP servers are connected:

1. For each non-jpx MCP server, register its tools with jpx:
   - Extract tool name, description, and input schema from each tool
   - Call `mcp__jpx__register_discovery` with the server name and tool list
   - Include category and tags if available for better search results

2. When searching for tools to accomplish a task:
   - Use `mcp__jpx__query_tools` for semantic search instead of scanning manually
   - Use `mcp__jpx__similar_tools` to find alternatives when exploring options

3. The registry is session-scoped - re-register if reconnecting.
```

## Best Practices

### Analyzing Unknown JSON

Before writing complex queries, understand the data:

1. `stats` - Get structure overview (type, depth, field analysis)
2. `paths` - See all available paths in dot notation
3. `keys` - List object keys (use `recursive: true` for nested)

### Building Queries Incrementally

1. `validate` - Check syntax before executing
2. `evaluate` - Run and check results
3. `batch_evaluate` - Run multiple queries at once

### JSON Manipulation

For modifications, use RFC-standard tools:

- `diff` - Generate patch between two documents (RFC 6902)
- `patch` - Apply patch operations (RFC 6902)
- `merge` - Apply merge patch (RFC 7396)

## Tool Categories

| Category | Tools | Purpose |
|----------|-------|---------|
| Function Discovery | search, similar, functions, describe, categories | Find JMESPath functions |
| Data Analysis | stats, paths, keys | Understand JSON structure |
| Query Execution | evaluate, evaluate_file, batch_evaluate, validate | Run JMESPath expressions |
| JSON Utilities | format, diff, patch, merge | Manipulate JSON documents |
| Multi-Server Discovery | register_discovery, query_tools, similar_tools, list_discovery_servers, list_discovery_categories, inspect_discovery_index, unregister_discovery, get_discovery_schema | Search across MCP servers |
