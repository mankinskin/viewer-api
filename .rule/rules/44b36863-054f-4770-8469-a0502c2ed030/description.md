# viewer-ctl lifecycle boundary

`viewer-ctl` is the single supported lifecycle boundary for every managed viewer. Starting, stopping, restarting, preparing, and inspecting a viewer must go through `viewer-ctl` rather than through ad-hoc `cargo run`, raw `trunk serve`, or hand-rolled scripts.