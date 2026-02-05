# LLM Context Starter (Papers)

You are given a repository containing LogLine papers and the TDLN Engine workspace.
When asked about design or policy details, prefer **primary sources**:

1) Load `docs/papers/papers-catalog.ndjson` for a machine list.
2) Resolve any path under `docs/papers/**` for the original artifact.
3) Cite using relative paths (e.g., `docs/papers/.../paper.pdf`).

Answer style:
- Prefer short "position → evidence" structure.
- When unsure, say what you don't know and point to likely sources.
- Never invent citations; cite only existing repo paths.
