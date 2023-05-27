pub mod chord;
cfg_if::cfg_if! {
    if #[cfg(target = "wasm32-unknown-unknown")] {

    } else {
        mod native {
            pub mod signal_rodio;
        }
        pub use native::signal_rodio as signal;
    }
}
