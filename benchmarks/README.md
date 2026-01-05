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
const BN254 = { name: "bn254", bits: 254, bytes_without_truncation: 31 }

(make
    (make cartesian-product ...[
            ["semi-avid", "kzg", "aplonk", "fri"]
            (1..10 | each { 2 ** $in })
            (3..18 | each { 2 ** $in * $BN254.bytes_without_truncation })
        ]
        | each {{ p: $in.0, k: $in.1, b: $in.2 }}
        | insert n { 2 * $in.k }
    )
    # --email
    # --shutdown
    --commit
    --seed 1
    # --push
    --curve $BN254.name
)
```

## plot
```bash
let data = make load-all

let seeds = seq 0 9 | each { 176606841000 + $in }

print --no-newline "filtering... "
let data = $data
    | where ([
        ($it.bytes mod 248 == 0),
        ($it.build         == "release"),
        ($it.cpu           == "ee672bb"),
        ($it.git           == "af83d3e"),
        ($it.curve         == "bn254"),
        ($it.seed          in $seeds),
    ] | all { $in })
print "done"

let tmp = mktemp --tmpdir XXXXXXX.json
print --no-newline "saving... "
$data | save -fp $tmp
print "done"
nu benchmarks/plot.nu ...[
    $tmp
    # --nb
    # --regular
    # --normalized
    # --clean
    # --plot
    # --compare
    # --stitch
]
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
> $$\delta = km\phi = \phi \times 2^\bullet$$
>
> In the end, the size of $\Delta$ is a _power of 2 multiple_ of $\phi$ and
> below are values of $\phi$ with the corresponding elliptic curve and number of
> bits in the prime order $p$ of $\mathbb{F}$
>
> | curve              | $p$ (bits) | $\phi$ (bits) |
> | ------------------ | ---------- | ------------- |
> | $\text{BLS12-381}$ | 381        | 376           |
> | $\text{BN254}$     | 254        | 248           |
