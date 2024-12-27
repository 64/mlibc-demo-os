# mlibc-demo-os

This is a toy kernel intended to demonstrate how to port [mlibc](https://github.com/managarm/mlibc) to a new operating system. It is written in Rust and runs on a 64-bit RISC-V target.

For pedagogical purposes we cut some corners. For example we do not implement a scheduler (only a single hardcoded task may run). There are no external interrupts, meaning no pre-emption. Only a single hart is supported.

Currently this kernel can run a statically linked hello world program. We plan to support a dynamically linked hello world in future.

TODO: How to build the userspace (including cc-runtime, mlibc)

## Example output

```
$ cargo run
[INFO ] Booting mlibc-demo-os...
[INFO ] Loading userspace program...
[INFO ] Jumping to userspace entrypoint at 0x10158
mlibc: Entering ld.so
mlibc: Leaving ld.so, jump to 0x10158
Hello world!
[INFO ] Userspace program exited with status code 0
```
