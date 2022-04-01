# litchi-rs

An x86-64 kernel with ~100% Rust in a week. The continuation of [Litchi](https://github.com/BugenZhao/Litchi).

![Screenshot](https://user-images.githubusercontent.com/25862682/161102150-9dd3f27f-5cbd-4c06-9559-c3b127eeb78b.png)

## Try it

Make sure the Rust toolchains and `qemu-system-x86_64` are installed, then simply execute...

```bash
make qemu
```

## Roadmap

### Booting

- [x] Bare-metal UEFI application.
- [x] In-memory ELF program loader.
- [x] Locate kernel executable with UEFI's simple file system.
- [x] Jump into the kernel.
- [x] Prepare `BootInfo` and pass to the kernel.
- [ ] ...

### Kernel Initializations

- [x] Serial logger.
- [x] Global Descriptor Table & Task State Segment.
- [x] Physical frame allocator, based on the boot info.
- [x] Kernel page table.
- [x] Kernel heap allocation & `extern crate alloc`.
- [x] Resolve ACPI table for interrupts & multiprocessors.
- [x] Trap handlers for critical faults.
- [x] Local APIC for the timer interrupt.
- [x] IO APIC for the UART serial.
- [ ] Bootstrap application processors.
- [ ] ...

### User Tasks

- [x] Load embedded ELF user programs.
- [x] RAII-style user memory allocator and mapper.
- [x] User library to provide init code.
- [x] Switch to user mode.
- [x] Frame-preserving timer interrupt handler for preemption.
- [x] Round-robin task scheduler.
- [x] System calls with shared memory.
- [x] User heap allocator.
- [x] Task recycling.
- [x] Idle task with kernel privilege.
- [x] Basic priority-based scheduler.
- [ ] Task spawning and forking.
- [ ] File or device resource management.
- [ ] Synchronization primitives.
- [ ] (Asynchronous) IO.
- [ ] A basic shell.
- [ ] ...

### Other Kernel Functionalities

- [x] Event-driven UART serial input handler.
- [ ] Multiprocessors.
- [ ] Kernel thread with async Rust.
- [ ] Simple file systems.
- [ ] IPC mechanisms.
- [ ] ...


## References

- https://os.phil-opp.com
- https://github.com/BugenZhao/Litchi
- https://github.com/alesharik/distros
- https://github.com/rcore-os/rboot
- https://github.com/rcore-os/trapframe-rs
- https://osdev.org
- https://pdos.csail.mit.edu/6.828/2018/schedule.html
- ...
