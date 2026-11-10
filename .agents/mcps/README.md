# MCP servers

Configuration snippets for MCP servers that are useful in this repository. One
directory per server:

```text
.agents/mcps/
├── context7/
│   └── mcp.jsonc
├── github/
│   └── mcp.jsonc
└── <server>/
    ├── mcp.jsonc
    ├── server.py        # when the server lives in this repository
    └── README.md
```

Nothing loads these files automatically. Each `mcp.jsonc` uses the OpenCode
shape; the sections below give the same server for Claude Code and Antigravity
CLI. No client configuration is committed as active.

## Clients

| Client | File | Root key |
| --- | --- | --- |
| OpenCode | `opencode.jsonc`, project or global | `mcp.servers` |
| Claude Code | `.mcp.json` at the project root | `mcpServers` |
| Antigravity CLI | `~/.gemini/config/mcp_config.json` globally; `.agents/mcp_config.json` per workspace | `mcpServers` |

### OpenCode

Copy the entry under `mcp.servers`, or rebuild it with the command at the top of
the file:

```sh
opencode mcp add <name> --url <remote-url>
opencode mcp add <name> -- npx -y <package>
```

Add `--global` when the server should be available in every project. A local
server uses `command` as an array and `environment`; `{env:NAME}` expands from
the environment.

### Claude Code

Project scope is a `.mcp.json` file at the repository root, which Claude Code
asks you to approve the first time. `--scope project` writes it:

```sh
claude mcp add --transport http context7 --scope project https://mcp.context7.com/mcp
```

Remote servers need an explicit transport, because an entry with a URL and no
`type` is read as a local server:

```json
{
  "mcpServers": {
    "context7": {
      "type": "http",
      "url": "https://mcp.context7.com/mcp"
    }
  }
}
```

A local server splits the command and its arguments, and uses `env`:

```json
{
  "mcpServers": {
    "github": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-e", "GITHUB_PERSONAL_ACCESS_TOKEN",
        "-e", "GITHUB_READ_ONLY",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_PERSONAL_ACCESS_TOKEN}",
        "GITHUB_READ_ONLY": "1"
      }
    }
  }
}
```

`${VAR}` and `${VAR:-default}` expand in `command`, `args`, `env`, `url`, and
`headers`, so a shared `.mcp.json` can point at a token you keep in the shell.

### Antigravity CLI

Antigravity CLI reads two files: `~/.gemini/config/mcp_config.json` for every
project, and `.agents/mcp_config.json` for the current workspace. The workspace
file sits next to this directory. Remote servers use `serverUrl` (`url` also
works):

```json
{
  "mcpServers": {
    "context7": {
      "serverUrl": "https://mcp.context7.com/mcp"
    }
  }
}
```

Local servers use `command` as a string plus `args`:

```json
{
  "mcpServers": {
    "github": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-e", "GITHUB_PERSONAL_ACCESS_TOKEN",
        "-e", "GITHUB_READ_ONLY",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "<token>",
        "GITHUB_READ_ONLY": "1"
      }
    }
  }
}
```

Antigravity documents no variable expansion, so a token has to be written in the
file. Keep that server in the global config, outside the repository, and leave
the workspace file free of credentials.

## Rules

- Never commit a token. Use `{env:NAME}` in OpenCode and `${NAME}` in Claude
  Code. Antigravity has no expansion, so credentialed servers stay in the global
  file.
- Prefer read-only servers. The GitHub entry runs with `GITHUB_READ_ONLY=1`, so
  it exposes no write tools.
- A server written for this repository keeps its code in its own directory, next
  to its `mcp.jsonc`, and documents how to run it. For Python, use
  [PEP 723](https://peps.python.org/pep-0723/) inline dependencies and run it
  with `uv run --script server.py`; no virtualenv or lockfile is committed.
