## Dependency Graph

```mermaid
flowchart LR
    Config[viewer-ctl.toml] --> Ctl[viewer-ctl]
    Api[viewer-api] --> Servers[viewer servers]
    Dioxus[viewer-api-dioxus] --> Frontends[viewer frontends]
    Ctl --> Servers
    Ctl --> Frontends
```
