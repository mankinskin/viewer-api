## Versioning rules

- Additive changes (new routes, new GraphQL fields, new frontend props with defaults) are allowed without a version bump.
- Removing or renaming routes, fields, props, or events requires a version bump and a spec entry that documents the migration path.
- The shared E2E suites must keep passing across all consumers before a contract change is merged; failures in one viewer block the change for all viewers.