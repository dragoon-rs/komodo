# Table of contents
- [Requirements](#requirements)
- [Run](#run)
- [Plot](#plot)

## Requirements
> [!note] NOTE
>
> these should only be required for plotting results

- install [GPLT](https://gitlab.isae-supaero.fr/a.stevan/gplt) (for plotting)
- install [Virtualenv](https://github.com/pypa/virtualenv) (for managing Python virtual environments)
- activate the `benchmarks` module
```bash
use benchmarks
```

> [!tip] TIP
>
> the `benchmarks` module is imported in `.env.nu` and can be used automatically
> with tools like [`nuenv`].

## Run
> [!note] NOTE
>
> the FRI benchmarks don't use a module from [src/bin/](src/bin/) with PLNK but rather an
> [example](../examples/fri.rs)

```bash
const RESULTS_DIR = "/path/to/komodo-benchmark-results/"
```

> [!tip] TIP
>
> during the course of development of this project, the benchmarks results,
> synched with [gitlab.isae-supaero.fr:dragoon/komodo-benchmark-results], have
> been stored locally in `../komodo-benchmark-results`.
>
> therefore, the `$RESULTS_DIR` constant is defined in `.env.nu` and can be
> exported automatically with tools like [`nuenv`].

```bash
(benchmarks run -o $RESULTS_DIR
    --rust-build "release"
    field group setup commit linalg fec
    --curves [ "bls12381", "pallas", "bn254" ]
    --degrees (0..13 | each { 2 ** $in })
    --matrix-sizes (0..7 | each { 2 ** $in })
    --data-sizes (0..18 | each { 512 * 2 ** $in })
    --ks [2, 4, 8, 16, 32, 64]
    --rhos [1.00, 0.50, 0.33, 0.20]
)
```
```bash
(benchmarks run -o $RESULTS_DIR
    --rust-build "release"
    semi-avid kzg aplonk
    --curves     [ "bn254" ]
    --data-sizes ([ 496kib, 124mib ] | into int)
    --ks         [ 8 ]
    --rhos       [ 0.5 ]
)
```

> [!important] IMPORTANT NOTE about aPlonK
>
> The aPlonK method requires the data to have a certain shape. Namely the number of polynomials,
> once the data has been arranged in a matrix needs to be a power of 2. We can use the script below
> to list all the possible input sizes for BLS12-381 (381 bits) and BN254 (254 bits):
> ```bash
> def pretty-filesize []: [
>     number -> string,
>     list<number> -> list<string>,
> ] {
>     def convert []: [ filesize -> string ] {
>         if $in < 1kib {
>             format filesize B
>         } else if $in < 1mib {
>             format filesize KiB
>         } else if $in < 1gib {
>             format filesize MiB
>         } else if $in < 1tib {
>             format filesize GiB
>         } else if $in < 1pib {
>             format filesize TiB
>         } else {
>             format filesize PiB
>         }
>     }
>     $in | into filesize | if ($in | describe --detailed).type == "list" { each { convert } } else { convert }
> }
>
> def possible-aplonk-inputs [bits: int, n: int] {
>     let bits = ($bits / 8 | math floor) * 8
>
>     seq 0 $n | each { |i|
>         let x = $bits * 2 ** $i
>         {
>             bits: $x,
>             literal: ($x | pretty-filesize | str replace ' ' '' | str downcase),
>         }
>     }
> }
> ```
> ## BLS12-381
> | bits      | literal    |
> | --------- | ---------- |
> | 376       | 376b       |
> | 752       | 752b       |
> | 1504      | 1.46875kib |
> | 3008      | 2.9375kib  |
> | 6016      | 5.875kib   |
> | 12032     | 11.75kib   |
> | 24064     | 23.5kib    |
> | 48128     | 47kib      |
> | 96256     | 94kib      |
> | 192512    | 188kib     |
> | 385024    | 376kib     |
> | 770048    | 752kib     |
> | 1540096   | 1.46875mib |
> | 3080192   | 2.9375mib  |
> | 6160384   | 5.875mib   |
> | 12320768  | 11.75mib   |
> | 24641536  | 23.5mib    |
> | 49283072  | 47mib      |
> | 98566144  | 94mib      |
> | 197132288 | 188mib     |
> | 394264576 | 376mib     |
> | 788529152 | 752mib     |
>
> ## BN254
> | bits       | literal   |
> | ---------- | --------- |
> | 248        | 248b      |
> | 496        | 496b      |
> | 992        | 992b      |
> | 1984       | 1.9375kib |
> | 3968       | 3.875kib  |
> | 7936       | 7.75kib   |
> | 15872      | 15.5kib   |
> | 31744      | 31kib     |
> | 63488      | 62kib     |
> | 126976     | 124kib    |
> | 253952     | 248kib    |
> | 507904     | 496kib    |
> | 1015808    | 992kib    |
> | 2031616    | 1.9375mib |
> | 4063232    | 3.875mib  |
> | 8126464    | 7.75mib   |
> | 16252928   | 15.5mib   |
> | 32505856   | 31mib     |
> | 65011712   | 62mib     |
> | 130023424  | 124mib    |
> | 260046848  | 248mib    |
> | 520093696  | 496mib    |
> | 1040187392 | 992mib    |

> [!tip] TIP
>
> the following `watch` can be used to see the results as they are dumped to `$RESULTS_DIR`
> ```bash
> let target = $RESULTS_DIR | path expand
> clear
> watch $target --glob '*.ndjson' { |op, path|
>     let now = date now | format date '%Y-%m-%dT%H:%M:%S'
>     let filename = $path | str replace $target '' | str trim --left --char '/'
>     $"($op)\t| (ansi cyan)($now)(ansi reset) | (ansi purple)($filename)(ansi reset)"
> }
> ```

## Plot
```bash
benchmarks plot -i $RESULTS_DIR -o out field group setup commit linalg fec --cpu-hash-pattern '^ee672bb3' --src-hash-pattern '^c8f3481f' --rust-build 'debug'
```
```bash
benchmarks plot -i $RESULTS_DIR -o out field group setup commit linalg fec --cpu-hash-pattern '^ee672bb3' --src-hash-pattern '^(00086216|04541a50)' --rust-build 'release'
```
```bash
use regex.nu *

let srcs = [
    "00086216", # 909176b0, da197416
    "b4212615", # 825cec65
    "04541a50", # a780eb1d, 206fe9d2, 5727d551
    "544835dd", # 5727d551
    "fa141146", # 5727d551
    "491b37ea", # 5727d551
    "4daad4a0", # 5727d551
    "b92ecb2d", # 5727d551
    "64f25f1d", # 5727d551
    "53e7bf81", # 5727d551
    "e591a733", # 5727d551
    "43ad9f18", # 062a39b5
    "90394e42", # 057569f7
    "18b67518", # 0acf66a2
    "e8d6f8fa", # 943e5932
    "0d0768b9", # e7b9968c
]
(benchmarks plot
    -i $RESULTS_DIR
    -o out
    protocols
    --git-hash-pattern ''
    --cpu-hash-pattern ('ee672bb315ea00fe5815f0e20db6aa88017c1ba8355794f411c10a6057377e57' | regex exact)
    --src-hash-pattern ($srcs | regex or | regex start)
    --rust-build 'release'
)
```

[gitlab.isae-supaero.fr:dragoon/komodo-benchmark-results]: https://gitlab.isae-supaero.fr/dragoon/komodo-benchmark-results
[`nuenv`]: https://github.com/nushell/nu_scripts/blob/main/nu-hooks/nu-hooks/nuenv/hook.nu
