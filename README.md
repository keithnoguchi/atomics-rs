# Rust Atomics and Locks

[![CI](https://github.com/keithnoguchi/atomics-rs/actions/workflows/ci.yml/badge.svg)](
https://github.com/keithnoguchi/atomics-rs/actions)

[mara bos]: https://m-ou.se/
[the library team]: https://www.rust-lang.org/governance/teams/library

Kudos to [Mara Bos] and [the library team].

## Examples

- [Chapter 1: A channel with `Mutex` and `Condvar`](ch01/src/main.rs)
- [Chapter 2: A lazy one-time atomic value initialization](ch02/src/main.rs)
- [Chapter 3: A memory fence with `Acquire`, `Relaxed`, and `Release`](ch03/src/main.rs)
- [Chapter 4: A spin lock](ch04/src/main.rs)
- [Chapter 5: An one-shot channel](ch05/src/main.rs)
- [Chapter 6: An `Arc<T>` and `Weak<T>`](ch06/src/main.rs)
- [Chapter 7: A Memory Ordering Bug](ch07/src/main.rs)
- [Chapter 8: `wait()` and `wake_one()` with `futex(2)`](ch08/src/main.rs)
- [Chapter 9: A mutex with `atomic-wait`](ch09/src/main.rs)

Happy Hacking!
