# Public surface contracts

Every viewer exposes a versioned public contract through `viewer-api` (HTTP routes, GraphQL schema, the shared frontend package). The contract is the integration surface for tests, MCP tools, and downstream automation, and it must be evolved deliberately.