//! GPU resources + initialisation for the graph3d view.

#![cfg(target_arch = "wasm32")]

mod init;

pub(crate) use init::{init_gpu, GpuResources};
