# Table of contents
- [Requirements](#requirements)
- [Run the benchmarks](#run-the-benchmarks)
    - [define them](#define-them)
    - [run them](#run-them)
- [Plot the benchmarks](#plot-the-benchmarks)

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

## Run the benchmarks
### define them

> :bulb: **Note**
>
> the FRI benchmarks don't use a module from [src/bin/](src/bin/) with PLNK but rather an
> [example](../examples/fri.rs)

```bash
const RESULTS_DIR = "/path/to/komodo-benchmark-results/"

let benchmarks = {
    linalg: {
        enabled: true,
        sizes: (seq 0 7 | each { 2 ** $in }),
        output: "linalg.ndjson",
        append: true,
    },
    setup: {
        enabled: true,
        degrees: (seq 0 13 | each { 2 ** $in }),
        curves: [ bls12381, pallas, bn254 ],
        output: "setup.ndjson",
        append: true,
    },
    commit: {
        enabled: true,
        degrees: (seq 0 13 | each { 2 ** $in }),
        curves: [ bls12381, pallas, bn254 ],
        output: "commit.ndjson",
        append: true,
    },
    recoding: {
        enabled: true,
        sizes: (seq 0 18 | each { 512 * 2 ** $in }),
        ks: [2, 4, 8, 16],
        curves: [ bls12381 ],
        output: "recoding.ndjson",
        append: true,
    },
    fec: {
        enabled: true,
        sizes: (seq 0 18 | each { 512 * 2 ** $in }),
        ks: [2, 4, 8, 16],
        curves: [ bls12381 ],
        output: "fec.ndjson",
        append: true,
    },
    fri: {
        enabled: true,
        sizes: (seq 0 15 | each { 2 ** $in * 4096b }),
        ks: [8, 128, 1024, 4096],
        blowup_factors: [2, 4],
        ns: [2],
        remainder_plus_ones: [1],
        nb_queries: [50],
        hashes: ["sha3-512"],
        ffs: ["fp128", "bls12-381"],
        output: "fri.ndjson",
        append: true,
    },
    field: {
        enabled: true,
        nb_measurements: 1000,
        output: "field.ndjson",
        append: true,
    },
    curve_group: {
        enabled: true,
        nb_measurements: 1000,
        output: "curve_group.ndjson",
        append: true,
    },
}
```

### run them
```bash
benchmarks run --output-dir $RESULTS_DIR $benchmarks
```

> the following `watch` can be used to see the results as they are dumped to `$RESULTS_DIR`
> ```bash
> watch $RESULTS_DIR { |op, path|
>     $"($op)  ($path)"
> }
> ```

## Plot the benchmarks
```bash
let plots = {
    linalg: { file: "linalg.ndjson" },
    setup: { file: "setup.ndjson" },
    commit: { file: "commit.ndjson" },
    fec: { file: "fec.ndjson" },
    recoding: { file: "recoding.ndjson" },
    fri: [
        [name,       y_type,   single, identity, normalize];
        [evaluating, duration, false,  false,    false    ],
        [encoding,   duration, false,  false,    false    ],
        [proving,    duration, false,  false,    false    ],
        [decoding,   duration, false,  false,    false    ],
        [verifying,  duration, true,   false,    false    ],
        [proofs,     filesize, false,  true,     true     ],
        [commits,    filesize, true,   true,     true     ],
        [proofs,     filesize, false,  true,     false    ],
        [commits,    filesize, true,   true,     false    ],
    ],
    field: {
        title: "field operations",
        file: field.ndjson,
        simple_operations: [ "exponentiation", "legendre", "inverse", "sqrt" ],
    },
    curve_group: {
        title: "curve group operations",
        file: curve_group.ndjson,
        simple_operations: [ "random sampling", "scalar multiplication", "affine scalar multiplication" ],
    },
}
```
```bash
benchmarks plot $plots --input-dir "/path/to/komodo-benchmark-results/<hash>" --output-dir "./figures/"
```
