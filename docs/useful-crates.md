# Potentially Useful Crates

For a crate to be used in the OS it must be a no\_std crate. Below are a list
of crates that I think may be useful to build rsv6:

* [spin](https://crates.io/crates/spin): a spinlock crate
* [x86](https://crates.io/crates/x86): Useful for interacting with hardware
* [pic8259\_simple](https://crates.io/crates/pic8259_simple): Phil Opperman
  uses this to interact with the PICs in his blog-os, we can probably use it as
  well
* [pc-keyboard](https://crates.io/crates/pc-keyboard): could be used to deal
  with keyboard scancodes
