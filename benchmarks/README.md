## requirements
- install [GPLT](https://gitlab.isae-supaero.fr/a.stevan/gplt)

```nushell
use .nushell/math.nu *
use .nushell/formats.nu *
```

## atomic operations
```nushell
cargo run --release --package benchmarks --bin field_operations -- --nb-measurements 1000 out> field.ndjson
cargo run --release --package benchmarks --bin curve_group_operations -- --nb-measurements 1000 out> curve_group.ndjson
```
```nushell
use .nushell/parse.nu read-atomic-ops

gplt multi_bar --title "simple field operations" -l "time (in ns)" (
    open field.ndjson
        | read-atomic-ops --exclude [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
gplt multi_bar --title "complex field operations" -l "time (in ns)" (
    open field.ndjson
        | read-atomic-ops --include [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
gplt multi_bar --title "simple curve group operations" -l "time (in ns)" (
    open curve_group.ndjson
        | read-atomic-ops --exclude [ "random sampling", "scalar multiplication", "affine scalar multiplication" ]
        | to json
)
gplt multi_bar --title "complex curve group operations" -l "time (in ns)" (
    open curve_group.ndjson
        | read-atomic-ops --include [ "random sampling", "scalar multiplication", "affine scalar multiplication" ]
        | to json
)
```

## linear algebra
```nushell
let sizes = seq 0 7 | each { 2 ** $in }

let out_linalg = $sizes | benchmarks linalg run

benchmarks linalg plot $out_linalg inverse
```

## trusted setup and commit
```nushell
let degrees = seq 0 13 | each { 2 ** $in }
let curves = [ bls12381, pallas, bn254 ]

let out_setup = $degrees | benchmarks setup run --curves $curves
let out_commit = $degrees | benchmarks commit run --curves $curves

benchmarks setup plot $out_setup
benchmarks commit plot $out_commit
```

## end-to-end benchmarks
```nushell
let sizes = seq 0 18 | each { 512 * 2 ** $in }
let ks = [2, 4, 8, 16]
let curves = [ bls12381 ]
```

### run
```nushell
let out_recoding = $sizes | benchmarks recoding run --ks $ks --curves $curves
let out_fec = $sizes | benchmarks fec run --ks $ks --curves $curves
```

### plot
```nushell
benchmarks recoding plot $out_recoding
benchmarks fec plot encoding $out_fec
benchmarks fec plot decoding $out_fec
benchmarks fec plot e2e $out_fec
benchmarks fec plot combined $out_fec --recoding $out_recoding
benchmarks fec plot ratio $out_fec --recoding $out_recoding
```
