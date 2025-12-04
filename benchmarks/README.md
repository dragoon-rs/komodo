> [!note]
> missing benchmarks from previous version (1.0.1) are listed in [`TODO.md`](TODO.md)

> [!tip]
> everything below is made to be run from the root of the _Komodo_ repo

## an example
```bash
use benchmarks/make.nu
make bench (0..18 | each { 1024 * 2 ** $in }) -k 2 -n 6 --nb-rounds 10
```

## run all
```bash
use benchmarks/make.nu

let cpu = make lscpu | to json | hash sha256
let git = git rev-parse HEAD | str trim

const RHO = 1 / 2
let BN254_F_SIZE = 254 / 8 | math floor | $in * 8

make build

mkdir benchmarks/results

1..6
    | each { 2 ** $in }
    | each {{ k: $in, n: ($in / $RHO | into int) }}
    | each { |it|
        make bench (0..8 | each { 1024 * 2 ** $in }) -k $it.k -n $it.n --nb-rounds 1 --protocol semi-avid
            | insert k $it.k
            | insert n $it.n
    }
    | flatten
    | insert git $git
    | insert cpu $cpu
    | save --force benchmarks/results/semi-avid.ndjson

1..6
    | each { 2 ** $in }
    | each {{ k: $in, n: ($in / $RHO | into int) }}
    | each { |it|
        make bench (0..8 | each { 1024 * 2 ** $in }) -k $it.k -n $it.n --nb-rounds 1 --protocol kzg
            | insert k $it.k
            | insert n $it.n
    }
    | flatten
    | insert git $git
    | insert cpu $cpu
    | save --force benchmarks/results/kzg.ndjson

1..6
    | each { 2 ** $in }
    | each {{ k: $in, n: ($in / $RHO | into int) }}
    | each { |it|
        make bench (0..8 | each { $BN254_F_SIZE * 2 ** $in }) -k $it.k -n $it.n --nb-rounds 1 --protocol aplonk
            | insert k $it.k
            | insert n $it.n
    }
    | flatten
    | insert git $git
    | insert cpu $cpu
    | save --force benchmarks/results/aplonk.ndjson

1..6
    | each { 2 ** $in }
    | each {{ k: $in, n: ($in / $RHO | into int) }}
    | each { |it|
        make bench (0..8 | each { 1024 * 2 ** $in }) -k $it.k -n $it.n --nb-rounds 1 --protocol fri --fri-ff 2 --fri-bf (1 / $RHO | into int) --fri-rpo 1 --fri-q 50
            | insert k $it.k
            | insert n $it.n
    }
    | flatten
    | insert git $git
    | insert cpu $cpu
    | save --force benchmarks/results/fri.ndjson
```

## plot all
```bash
def get-values []: [ table -> list<float> ] {
    update times { try { math avg } }
        | group-by --to-table k
        | sort-by { $in.k | into int }
        | reverse
        | get items
        | each { each { try { $in.times | math log 10 } catch { -1 } } }
        | flatten
        | wrap _
        | update _ { if $in == -1 { "NaN" } else { $in }}
        | get _
}

def plot-steps [
    data: table,
    steps: list<string>,
    --name: string,
    --width (-W): int,
    --height (-H): int,
] {
    for step in $steps {
        $data
            | where step == $step
            | get-values
            | uv run benchmarks/heat_map.py ...$in -W $width -H $height --save $"($name)-($step).png"
    }
}

plot-steps (open benchmarks/results/semi-avid.ndjson) --name semi-avid -W 9 -H 6 ["t_prove_k", "t_build_n", "t_verify_n"]
plot-steps (open benchmarks/results/kzg.ndjson      ) --name kzg       -W 9 -H 6 ["t_commit_m", "t_prove_n", "t_verify_n", "t_verify_batch_3"]
plot-steps (open benchmarks/results/aplonk.ndjson   ) --name aplonk    -W 9 -H 6 ["t_commit_m", "t_prove_n", "t_verify_n"]
plot-steps (open benchmarks/results/fri.ndjson      ) --name fri       -W 9 -H 6 ["t_evaluate_kn", "t_encode_n", "t_prove_n", "t_verify_n", "t_decode_k"]
```


> [!important]
>
> ---
>
> **Notations**: in the following, for any $n \in \mathbb{N}$ and $f$ a function
> from $\mathbb{N}$ to $\mathbb{N}$,
>
> $$n = f(\bullet) \leftrightarrow \exists k \in \mathbb{N}: n = f(k)$$
> e.g with $f: x \mapsto 2^x$, $n = 2^\bullet$ means that $n$ is a power of $2$.
>
> ---
>
> The $\text{aPlonK}$ method requires the data to have a certain shape.
> Once the data has been arranged in an $m \times k$ matrix, the number of
> polynomials is $m = 2^\bullet$.
>
> $\text{aPlonK}$ also requires $k = 2^\bullet$ for the folding of $\text{IPA}$
> to work.
>
> Finally, the relation between $k$, $m$, the size $\phi$ of an element of
> $\mathbb{F}$ _without bit truncation_ and the size $\delta$ of the input data
> $\Delta$ is
>
> $$\delta = km\delta = \delta \times 2^\bullet$$
>
> In the end, the size of $\Delta$ is a _power of 2 multiple_ of $\delta$ and
> below are values of $\phi$ with the corresponding elliptic curve and number of
> bits in the prime order $p$ of $\mathbb{F}$
>
> | curve              | $p$ | $\phi$ |
> | ------------------ | --- | ------ |
> | $\text{BLS12-381}$ | 381 | 376    |
> | $\text{BN254}$     | 254 | 248    |
