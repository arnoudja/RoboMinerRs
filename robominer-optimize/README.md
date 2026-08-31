# robominer-optimize

Offline genetic-algorithm harness for experimenting with robot programs. This crate is **not** on the production web/engine path; it reads program sources and sim configuration, then evolves populations in memory.

## When to use it

- Exploring program variants against a fixed mining area layout
- Balance experiments that do not require MySQL state
- Prototyping crossover/mutation strategies before hand-writing benchmark programs

For comparing recommended player programs against seeded areas, prefer the diagnostic benchmark in `robominer-domain`:

```sh
cargo test -p robominer-domain benchmark_recommended_programs -- --nocapture
```

## Running

Build and run the CLI binary:

```sh
cargo run -p robominer-optimize -- --help
```

The optimizer depends on `robominer-program` and `robominer-sim` only; it does not need a database URL.

## Layout

| Path | Role |
| --- | --- |
| `src/ga.rs` | Genetic algorithm loop |
| `src/genome.rs` | Program representation and mutation |
| `src/lib.rs` | Crate entry and shared helpers |

When changing GA behaviour, add or extend unit tests under `src/` — this crate has lighter test coverage than the core gameplay crates.
