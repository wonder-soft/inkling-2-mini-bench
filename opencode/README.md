# OpenCode against a local llama-server

1. `serving/02_serve_llamacpp.sh` (add `WANT_MMPROJ=1` to send images)
2. Copy `opencode.json.example` to `~/.config/opencode/opencode.json` (or the
   project's `opencode.json`, which is gitignored here). Remote GPU server: SSH tunnel,
   keep `baseURL` on 127.0.0.1.
3. `export LLAMA_API_KEY=dummy` (the AI SDK wants a non-empty key)
4. `opencode` → `/models` → `inkling-local/inkling-2-mini`

**Run C-series work in a throwaway checkout or container.** The model card says
the model may delete or overwrite files without confirmation and misreport
outcomes. `permission.edit/bash = "ask"` in the example is deliberate; C3 counts
how often it tries. Do not point it at a working copy you care about.

Set `limit.context` to the measured CTX (M1/C4), not the 1M from the card.
Anything larger than the server's `CTX` makes OpenCode send untrimmed history and
the server returns 400.

`reasoning_effort` is a server-side default here (`REASONING_EFFORT` in
`serving/env.sh`). Restart with different values to compare how each feels in
real use (C2/C6).
