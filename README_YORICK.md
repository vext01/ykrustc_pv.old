# Yorick-specific Notes

Experimental code for hardware tracing is gated by `#[cfg(yk_hwt)]`. Software
tracing support will still be available if this compile-time variable is unset.
