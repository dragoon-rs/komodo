# Table of contents
- [Requirements](#requirements)
- [Atomic operations](#atomic-operations)
- [Linear algebra](#linear-algebra)
- [Trusted setup and commit](#trusted-setup-and-commit)
- [End to end benchmarks](#end-to-end-benchmarks)
- [FRI](#fri)

## requirements
> :bulb: **Note**
>
> these should only be required for plotting results

- install [GPLT](https://gitlab.isae-supaero.fr/a.stevan/gplt)
- create a virtual environment
```bash
const VENV = "~/.local/share/venvs/gplt/bin/activate.nu" | path expand
```
```bash
virtualenv ($VENV | path dirname --num-levels 2)
```
- activate the virtual environment
```bash
overlay use $VENV
```
- activate required modules
```bash
use benchmarks
```

> :bulb: **Note**
>
> i personally use the [`nuenv` hook](https://github.com/nushell/nu_scripts/blob/main/nu-hooks/nu-hooks/nuenv/hook.nu)
> that reads [`.env.nu`](../.env.nu).

## atomic operations
```nushell
cargo run --release --package benchmarks --bin field_operations -- --nb-measurements 1000 out> field.ndjson
cargo run --release --package benchmarks --bin curve_group_operations -- --nb-measurements 1000 out> curve_group.ndjson
```
```nushell
use benchmarks/nu-lib/utils/parse.nu read-atomic-ops

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

## FRI
> :bulb: **Note**
>
> the FRI benchmarks don't use a module from [src/bin/](src/bin/) with PLNK but rather an
> [example](../examples/fri.rs)

- modify [benchmarks/params/fri.nu](benchmarks/params/fri.nu)
- source it
```nushell
source benchmarks/params/fri.nu
```
- run the benchmarks
```nushell
use std formats "to ndjson"

(benchmarks fri run
    --data-sizes $DATA_SIZES
    --ks $KS
    --blowup-factors $BFS
    --nb-queries $QS
    --hashes $HS
    --finite-fields $FFS
    --remainders $RPOS
    --folding-factors $NS
) | to ndjson out> $DATA
```

> the following `watch` call can be used to see the results as they are dumped to `$DATA`
> ```nushell
> use std formats "from ndjson"
>
> watch . {
>     open --raw $DATA
>         | lines
>         | last
>         | from ndjson
>         | into int evaluating encoding proving verifying decoding
>         | into duration evaluating encoding proving verifying decoding
>         | into filesize proofs commits d
>         | into record
> }
> ```

```nushell
benchmarks fri plot --dump-dir $OUTPUT_DIR --file $DATA evaluating encoding proving decoding --y-type "duration"
benchmarks fri plot --dump-dir $OUTPUT_DIR --file $DATA verifying --y-type "duration" --single

benchmarks fri plot --dump-dir $OUTPUT_DIR --file $DATA proofs --y-type "filesize" --identity --normalize
benchmarks fri plot --dump-dir $OUTPUT_DIR --file $DATA commits --y-type "filesize" --single --identity --normalize

benchmarks fri plot --dump-dir $OUTPUT_DIR --file $DATA proofs --y-type "filesize" --identity
benchmarks fri plot --dump-dir $OUTPUT_DIR --file $DATA commits --y-type "filesize" --single --identity
```
