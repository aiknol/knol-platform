# Quick Start Guide

Get the Knol MCP server up and running in 5 minutes.

## 1. Build the Server

```bash
cd /path/to/knol-mcp
npm install
npm run build
```

This creates the compiled server at `dist/index.js`.

## 2. Get Your API Key

1. Log in to your Knol account
2. Go to Settings → API Keys
3. Create a new API key or copy an existing one
4. Keep it safe (you'll need it in step 3)

## 3. Configure Your Tool

Choose your AI coding tool and add the Knol MCP server configuration.

### For Claude Code

Edit `~/.config/Claude Code/mcp.json`:

```json
{
  "mcpServers": {
    "knol": {
      "command": "node",
      "args": ["/absolute/path/to/knol-mcp/dist/index.js"],
      "env": {
        "KNOL_API_KEY": "your-api-key-here",
        "KNOL_API_URL": "https://api.knol.io",
        "KNOL_USER_ID": "your-user-id"
      }
    }
  }
}
```

### For Cursor

Same as Claude Code, but edit `.cursor/settings.json`.

### For Windsurf

Same as Claude Code, but edit `~/.windsurf/mcp.json`.

## 4. Restart Your Tool

Close and reopen your AI coding tool. The Knol MCP server should now be available.

## 5. Try It Out

In your AI coding tool, ask:

```
"Store a memory: I prefer TypeScript for backend development"
```

The tool should respond with a confirmation, using the `knol_remember` tool.

Then ask:

```
"What do I prefer for backend development?"
```

The tool should search your memories and find the stored preference.

## Key Features

- **knol_remember**: Save memories from your coding sessions
- **knol_search**: Find memories by semantic search
- **knol_get**: Retrieve a specific memory by ID
- **knol_update**: Update existing memories
- **knol_delete**: Remove memories
- **knol_entities**: Explore knowledge graph entities
- **knol_entity_neighbors**: See related entities
- **knol://recent**: Quick access to your 10 most recent memories

## Environment Variables

| Variable | Value | Example |
|----------|-------|---------|
| `KNOL_API_KEY` | Your API key | `sk_live_abc123...` |
| `KNOL_API_URL` | API endpoint | `https://api.knol.io` |
| `KNOL_USER_ID` | Your user ID | `user@example.com` |

## Troubleshooting

### "Cannot find module" error

```bash
cd /path/to/knol-mcp
npm install
npm run build
```

### "Unauthorized" error

- Check your API key is correct
- Verify API key has the right permissions
- Make sure `KNOL_API_URL` matches your API endpoint

### Tools not appearing

- Restart your AI tool
- Check the configuration file syntax
- Verify all required environment variables are set

### Still stuck?

See `SETUP_EXAMPLES.md` for detailed configuration examples or `DEVELOPMENT.md` for troubleshooting tips.

## Next Steps

1. **Learn the Tools**: See `README.md` for detailed documentation on each tool
2. **Explore Examples**: Check `SETUP_EXAMPLES.md` for advanced configurations
3. **Develop**: See `DEVELOPMENT.md` if you want to contribute or customize

## Common Workflows

### Save important code insights

```
User: "Remember: the database schema uses UUID primary keys"
Claude uses knol_remember → Memory saved
```

### Retrieve context from previous sessions

```
User: "What was I working on related to authentication?"
Claude uses knol_search → Shows past authentication work
```

### Explore knowledge relationships

```
User: "What technologies are related to GraphQL?"
Claude uses knol_entity_neighbors → Shows Node.js, TypeScript, API design, etc.
```

## Tips

- Use `session_id` to group related memories from the same coding session
- Add `metadata` to memories for better categorization
- Set `importance` scores to prioritize key insights
- Use `knol://recent` to quickly check what you've been working on

## What's Next?

Your memories will now persist across sessions, helping your AI assistant provide better, more contextual help.

Enjoy building with Knol!
