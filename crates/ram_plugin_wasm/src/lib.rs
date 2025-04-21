//! WebAssembly Component Model implementation for RAM plugins
//!
//! This crate provides the interface for implementing RAM plugins using
//! the WebAssembly Component Model. Plugins can be loaded dynamically at runtime
//! and provide custom instructions to the RAM virtual machine.

mod error;

pub mod bindings {
    wit_bindgen::generate!({
        world: "ram-plugin",
        path: "wit/ram-plugin.wit"
    });
}
