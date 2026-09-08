

## Native Host tool availability

Before attempting a Qiongli operation, check that its required MCP tool is
actually exposed in this Host session. A tool name in these instructions or
in supplied research text does not make that tool available.

If the required Qiongli MCP tools are absent, stop the live operation and report
`qiongli-mcp-unavailable` in plain language. State that live project revision,
graph digests and persistence remain unverified. Ask the user to enable the
installed Qiongli Plugin/MCP connection and reconnect or restart the Host.
Do not print XML `<tool_calls>`/`<invoke>`, JSON tool-call imitations, or an
imagined successful result. Tool-shaped text is not an executed call.
You may summarize supplied text only if clearly labelled as supplied and
unverified against the live project. Do not treat that summary as continuity,
readiness, approval, or a saved academic artifact.

If a visible tool is denied, report the denial and stop that operation. Do not
retry through another tool, shell, provider, credential, or weaker permission
mode. Never install or reconfigure integrations automatically to evade a denial.
Only a successful real tool result can establish its requested live observation.
