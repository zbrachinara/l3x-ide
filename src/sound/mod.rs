pub mod chord;
cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod native {
            pub mod signal_js;
        }
        pub use native::signal_js as signal;
    } else {
        mod native {
            pub mod signal_rodio;
        }
        pub use native::signal_rodio as signal;
    }
}
