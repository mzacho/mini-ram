# Building

With cargo and a default Rust toolchain with version rustc 1.78.0-nightly:

```
cargo build
```

# Running

For the SHA256 demo first generate a mac of some message:

```
MSG=$(echo -n foo)
MAC=$(echo -n $MSG | sha256sum | cut -d ' ' -f 1)
```

Start the VOLE dealer on an open port:

```
cargo run -- -p vole --port 5000
```

Start the prover, passing it the name of the program (see frontend/src/main.rs) with `-x`, time bound with `-t` and witness with `--arg`:

```
cargo run -- -p prover
    --port 5001 \
    --vole-port 5000 \
    -x verify_compress -t 3895 --arg $MSG,$MAC
```

after the prover has encoded the witness then start the verifier

```
cargo run -- -p prover
    --port 5001 \
    --vole-port 5000 \
    -x verify_compress -t 3895 --arg $MAC
```
