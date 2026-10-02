# Setup Examples for AI Coding Tools

This document provides detailed setup examples for integrating the Cortex MCP server with various AI coding tools.

## Prerequisites

1. **Node.js 18+** installed
2. **Cortex API server** running (with access to API key)
3. **MCP server built**:

```bash
npm install
npm run build
```

4. **Keep the full path** to the built server: `/path/to/cortex-mcp/dist/index.js`

## Claude Code

### Configuration Method 1: Direct File Path

Edit `~/.config/Claude Code/mcp.json` (or `~/Library/Application Support/Claude Code/mcp.json` on macOS):

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["/absolute/path/to/cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_KEY": "sk_live_your_actual_api_key_here",
        "CORTEX_API_URL": "https://api.cortex.doaide.com",
        "KNOL_USER_ID": "your-user-id"
      }
    }
  }
}
```

### Configuration Method 2: Using npm (if published)

```json
{
  "mcpServers": {
    "cortex": {
      "command": "npx",
      "args": ["@doaide/cortex-mcp-server"],
      "env": {
        "CORTEX_API_KEY": "sk_live_your_actual_api_key_here",
        "CORTEX_API_URL": "https://api.cortex.doaide.com",
        "KNOL_USER_ID": "your-user-id"
      }
    }
  }
}
```

### Configuration Method 3: Shell Script Wrapper

Create `/usr/local/bin/cortex-mcp-wrapper`:

```bash
#!/bin/bash
export CORTEX_API_KEY="sk_live_your_actual_api_key_here"
export CORTEX_API_URL="https://api.cortex.doaide.com"
export KNOL_USER_ID="your-user-id"
exec node /absolute/path/to/cortex-mcp/dist/index.js
```

Then in `mcp.json`:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "/usr/local/bin/cortex-mcp-wrapper"
    }
  }
}
```

## Cursor

### Configuration Method 1: Direct Setup

Edit `.cursor/settings.json`:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["/absolute/path/to/cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_KEY": "sk_live_your_actual_api_key_here",
        "CORTEX_API_URL": "https://api.cortex.doaide.com",
        "KNOL_USER_ID": "your-user-id"
      }
    }
  }
}
```

### Configuration Method 2: Via Project Settings

1. Open Cursor
2. Go to Settings → Extensions → MCP Servers
3. Click "Add Server"
4. Configure:
   - **Name**: cortex
   - **Command**: node
   - **Args**: `/absolute/path/to/cortex-mcp/dist/index.js`
   - **Environment Variables**:
     - `CORTEX_API_KEY`: Your API key
     - `CORTEX_API_URL`: https://api.cortex.doaide.com
     - `KNOL_USER_ID`: your-user-id

### Configuration Method 3: Workspace Configuration

Create `.cursor/settings.json` in your workspace:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["${workspaceFolder}/cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_KEY": "${env:CORTEX_API_KEY}",
        "CORTEX_API_URL": "https://api.cortex.doaide.com",
        "KNOL_USER_ID": "${env:KNOL_USER_ID}"
      }
    }
  }
}
```

Then set environment variables:

```bash
export CORTEX_API_KEY="sk_live_your_actual_api_key_here"
export KNOL_USER_ID="your-user-id"
```

## Windsurf

### Configuration Method 1: System-wide

Edit `~/.windsurf/mcp.json`:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["/absolute/path/to/cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_KEY": "sk_live_your_actual_api_key_here",
        "CORTEX_API_URL": "https://api.cortex.doaide.com",
        "KNOL_USER_ID": "your-user-id"
      }
    }
  }
}
```

### Configuration Method 2: Project-specific

Create `.windsurf/mcp.json` in your project root:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["./cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_KEY": "sk_live_your_actual_api_key_here",
        "CORTEX_API_URL": "https://api.cortex.doaide.com",
        "KNOL_USER_ID": "your-user-id"
      }
    }
  }
}
```

### Configuration Method 3: Environment Variables Only

Create `.windsurf/mcp.json`:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["/absolute/path/to/cortex-mcp/dist/index.js"]
    }
  }
}
```

And set environment variables in your shell profile:

```bash
# In ~/.bashrc, ~/.zshrc, or equivalent
export CORTEX_API_KEY="sk_live_your_actual_api_key_here"
export CORTEX_API_URL="https://api.cortex.doaide.com"
export KNOL_USER_ID="your-user-id"
```

## Multiple Environments

### Development Setup

`.cursor/settings.dev.json`:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["/path/to/cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_KEY": "sk_dev_test_key",
        "CORTEX_API_URL": "http://localhost:8080",
        "KNOL_USER_ID": "dev-user"
      }
    }
  }
}
```

### Production Setup

`.cursor/settings.prod.json`:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["/path/to/cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_KEY": "sk_prod_secure_key",
        "CORTEX_API_URL": "https://api.cortex.doaide.com",
        "KNOL_USER_ID": "prod-user"
      }
    }
  }
}
```

## Docker Setup

If running in a Docker container:

### Dockerfile

```dockerfile
FROM node:18-alpine

WORKDIR /app

# Copy MCP server
COPY cortex-mcp ./cortex-mcp
WORKDIR /app/cortex-mcp

# Build
RUN npm install && npm run build

# Set entry point
ENTRYPOINT ["node", "dist/index.js"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  cortex-api:
    image: cortex/api:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://...

  mcp-server:
    build: ./cortex-mcp
    environment:
      - CORTEX_API_URL=http://cortex-api:8080
      - CORTEX_API_KEY=${CORTEX_API_KEY}
      - KNOL_USER_ID=${KNOL_USER_ID}
    depends_on:
      - cortex-api
```

## Troubleshooting

### Server Won't Start

**Error**: `Cannot find module '@modelcontextprotocol/sdk'`

**Solution**:
```bash
cd /path/to/cortex-mcp
npm install
npm run build
```

### API Key Not Working

**Error**: `Cortex API error (401): Unauthorized`

**Solution**:
1. Verify API key is correct
2. Check API key has correct permissions
3. Verify CORTEX_API_URL is correct
4. Test API key directly:

```bash
curl -H "Authorization: Bearer YOUR_KEY" \
  https://api.cortex.doaide.com/v1/memory/search \
  -d '{"query":"test","limit":1}' \
  -H "Content-Type: application/json"
```

### Connection Refused

**Error**: `Failed to connect to localhost:8080`

**Solution**:
1. Verify Cortex API server is running
2. Check CORTEX_API_URL is correct
3. Test network connectivity: `curl http://localhost:8080/health`

### Tools Not Appearing

**Error**: Tools don't appear in Claude Code/Cursor/Windsurf

**Solution**:
1. Restart the tool
2. Check server logs for errors
3. Verify MCP configuration syntax is correct
4. Test server directly: `CORTEX_API_KEY=test node dist/index.js`

### Environment Variable Not Used

**Issue**: Environment variable seems to be ignored

**Solution**:
1. Verify variable is set in the correct scope
2. Use absolute paths (not relative)
3. Verify variable name matches exactly (case-sensitive)
4. Try setting in wrapper script instead

## Best Practices

### 1. Use Absolute Paths

Always use absolute paths to the MCP server:

```json
{
  "args": ["/absolute/path/to/cortex-mcp/dist/index.js"]
}
```

Not relative paths:

```json
{
  "args": ["./cortex-mcp/dist/index.js"]  // Avoid
}
```

### 2. Keep Keys Secure

Never commit API keys to version control:

```bash
# Add to .gitignore
echo ".cursor/settings.json" >> .gitignore
echo ".windsurf/mcp.json" >> .gitignore
```

Use environment variables instead:

```json
{
  "env": {
    "CORTEX_API_KEY": "${env:CORTEX_API_KEY}"
  }
}
```

### 3. Use Version Control

Keep MCP configuration in version control but exclude secrets:

```json
{
  "mcpServers": {
    "cortex": {
      "command": "node",
      "args": ["${workspaceFolder}/cortex-mcp/dist/index.js"],
      "env": {
        "CORTEX_API_URL": "https://api.cortex.doaide.com"
      }
    }
  }
}
```

Then set `CORTEX_API_KEY` and `KNOL_USER_ID` locally.

### 4. Test Before Production

Always test in development first:

```bash
export CORTEX_API_URL="http://localhost:8080"
export CORTEX_API_KEY="test-key"
npm start
```

### 5. Monitor Logs

Check logs for errors:

```bash
# Claude Code
tail -f ~/.config/Claude\ Code/logs/mcp.log

# Cursor
tail -f ~/.cursor/logs/mcp.log

# Windsurf
tail -f ~/.windsurf/logs/mcp.log
```

## Example Usage Patterns

### Pattern 1: Store Session Context

```
User: "Store my current context about the database schema"

Claude: Using knol_remember with session_id for context tracking...
  Memory stored: "User working on database schema optimization..."
```

### Pattern 2: Recall Previous Work

```
User: "What was I working on last time?"

Claude: Using cortex://recent resource to retrieve recent memories...
  Retrieved 10 recent memories about your work
```

### Pattern 3: Knowledge Graph Exploration

```
User: "Show me related concepts to TypeScript"

Claude: Using knol_entity_neighbors with entity_type filtering...
  Found 8 related entities: React, Node.js, JavaScript, etc.
```

### Pattern 4: Cross-session Learning

```
User: "Learn from all my previous work on this topic"

Claude: Using knol_search with graph_depth for deep context retrieval...
  Found 15 related memories across sessions with entity relationships
```
