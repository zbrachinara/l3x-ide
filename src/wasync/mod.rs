cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod native {
            pub mod wasm;
        }
        pub use native::wasm::*;
    } else {
        mod native {
            pub mod pc;
        }
        pub use native::pc::*;
    }
}
