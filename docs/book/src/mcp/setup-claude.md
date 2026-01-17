# Setup with Claude Desktop

Configure jpx as an MCP server for Claude Desktop.

## Prerequisites

1. Install jpx (MCP support is included by default):
   ```bash
   brew install joshrotenberg/brew/jpx
   # or
   cargo install jpx
   ```

2. Have Claude Desktop installed

## Configuration

### macOS

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx",
      "args": ["mcp"]
    }
  }
}
```

If jpx is not in your PATH, use the full path:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "/opt/homebrew/bin/jpx",
      "args": ["mcp"]
    }
  }
}
```

### Windows

Edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "C:\\path\\to\\jpx.exe",
      "args": ["mcp"]
    }
  }
}
```

## Verify Setup

1. Restart Claude Desktop
2. Look for the jpx tools in Claude's tool list
3. Try a simple query:

```
User: I have this JSON: {"users": [{"name": "alice"}, {"name": "bob"}]}
      Get all the names.

Claude: [Uses jpx.evaluate]
        Result: ["alice", "bob"]
```

## Strict Mode

To use only standard JMESPath functions (no extensions):

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx",
      "args": ["mcp", "--strict"]
    }
  }
}
```

## Troubleshooting

### jpx not found

Make sure jpx is in your PATH, or use the full path in the config.

### Tools not appearing

1. Check the config file syntax (must be valid JSON)
2. Restart Claude Desktop completely
3. Check Claude Desktop logs for errors

### Permission errors on file access

The `evaluate_file` tool has security restrictions. It only allows access to files in safe directories (not system paths).
