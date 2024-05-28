```shell
cargo run
    | lines
    | parse "{curve}: {fq} -> {fr}"
    | into int fq fr
    | insert x { (1 - $in.fr / $in.fq) * 100 | math round --precision 1 }
```

which gives the followin table

| curve                            | fq  | fr  | x    |
| -------------------------------- | --- | --- | ---- |
| ark_bls12_377                    | 377 | 253 | 32.9 |
| ark_bls12_381                    | 381 | 255 | 33.1 |
| ark_bn254                        | 254 | 254 | 0    |
| ark_bw6_761                      | 761 | 377 | 50.5 |
| ark_cp6_782                      | 782 | 377 | 51.8 |
| ark_curve25519                   | 255 | 253 | 0.8  |
| ark_ed25519                      | 255 | 253 | 0.8  |
| ark_ed_on_bls12_377              | 253 | 251 | 0.8  |
| ark_ed_on_bls12_381              | 255 | 252 | 1.2  |
| ark_ed_on_bls12_381_bandersnatch | 255 | 253 | 0.8  |
| ark_ed_on_bn254                  | 254 | 251 | 1.2  |
| ark_ed_on_bw6_761                | 377 | 374 | 0.8  |
| ark_ed_on_cp6_782                | 377 | 374 | 0.8  |
| ark_ed_on_mnt4_298               | 298 | 296 | 0.7  |
| ark_ed_on_mnt4_753               | 753 | 750 | 0.4  |
| ark_mnt4_298                     | 298 | 298 | 0    |
| ark_mnt4_753                     | 753 | 753 | 0    |
| ark_mnt6_298                     | 298 | 298 | 0    |
| ark_mnt6_753                     | 753 | 753 | 0    |
| ark_pallas                       | 255 | 255 | 0    |
| ark_secp256k1                    | 256 | 256 | 0    |
| ark_secp256r1                    | 256 | 256 | 0    |
| ark_secp384r1                    | 384 | 384 | 0    |
| ark_secq256k1                    | 256 | 256 | 0    |
| ark_vesta                        | 255 | 255 | 0    |
