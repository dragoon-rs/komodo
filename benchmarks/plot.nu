use make.nu
use ffmpeg.nu *
use ../log.nu [ "log warning", "log error", "log info" ]

const PROTOCOLS = {
    "semi-avid": { name: "Semi-AVID", color: {r: 255, g: 128, b: 166} },
    "kzg"      : { name: "KZG+"     , color: {r: 128, g: 255, b: 232} },
    "aplonk"   : { name: "aPlonK"   , color: {r: 199, g: 128, b: 255} },
    "fri"      : { name: "FRI"      , color: {r: 255, g: 181, b: 128} },
}

def missing-flag [f: string, v: any, --name: string] {
    if $v == null {
        let header = if $name == null {
            ""
        } else {
            $"(ansi purple)($name)(ansi reset): "
        }
        error make --unspanned { msg: $"(ansi red_bold)invalid_args(ansi reset):($header)missing ($f)" }
    }
}

def blend-color [it: record, --worst]: [ nothing -> record<r: int, g: int, b: int> ] {
    $PROTOCOLS
        | items { |k, v|
            if ($it | get $k) != null {
                [($it | get $k), $v.color]
            }
        }
        | compact
        | each { { x: $in.0, ...$in.1 } }
        | update r { |it| $it.r * ($it.x | get (if $worst { "w" } else { "b" })) | into int }
        | update g { |it| $it.g * ($it.x | get (if $worst { "w" } else { "b" })) | into int }
        | update b { |it| $it.b * ($it.x | get (if $worst { "w" } else { "b" })) | into int }
        | reject x
        | where r != null
        | {
            r: ($in.r | math sum),
            g: ($in.g | math sum),
            b: ($in.b | math sum),
        }
}

def uniq-color [it: record, --worst]: [ nothing -> record<r: int, g: int, b: int> ] {
    let best = $it | select ...($PROTOCOLS | columns) | transpose p v | compact v | sort-by { $in.v | get (if $worst { "w" } else { "b" }) } | where v != null | last
    $PROTOCOLS | get $best.p | get color
}

const CURVES = {
    bn254: { bits: 254, bytes_without_truncation: 31 },
}

def main [
    data_filepath: path,
    --clean,
    --regular,
    --normalized,
    --nb
    --stitch,
    --plot,
    --compare,
] {
    if $clean { rm -vf *.png }

    let data = open $data_filepath | insert m { |it| $it.bytes / ($it.k * ($CURVES | get $it.curve).bytes_without_truncation) }

    let complete_x = 3..18 | each { 31 * 2 ** $in }
    let complete_y = 1..10 | each {      2 ** $in }

    let xticks  = false
    let yticks  = false
    let cbar    = false
    let figsize = { h: 9. } | insert w { $in.h * 1.5948158253751705 }
    # let figsize = { h: 9. } | insert w { $in.h * (16 / 9) }

    if $plot and $regular {
        $data
            | (make plot
                --plot-nb=$nb
                --complete-x $complete_x
                --complete-y $complete_y
                --xticks=$xticks
                --yticks=$yticks
                --cbar=$cbar
                --cmap "viridis"
                --figsize $figsize
                [
                    [p           , __k               ];
                    ["semi-avid" , "t_commit_k"      ],
                    ["semi-avid" , "t_build_n"       ],
                    ["semi-avid" , "t_verify_n"      ],
                    ["kzg"       , "t_commit_m"      ],
                    ["kzg"       , "t_prove_n"       ],
                    ["kzg"       , "t_verify_n"      ],
                  # ["kzg"       , "t_verify_batch_3"],
                    ["aplonk"    , "t_commit_m"      ],
                    ["aplonk"    , "t_prove_n"       ],
                    ["aplonk"    , "t_verify_n"      ],
                  # ["fri"       , "t_evaluate_kn"   ],
                  # ["fri"       , "t_encode_n"      ],
                    ["fri"       , "t_prove_n"       ],
                    ["fri"       , "t_verify_n"      ],
                  # ["fri"       , "t_decode_k"      ],
                ]
            )
    }

    if $plot and $normalized {
        $data
            | each { |it|
                match $it.__k {
                    "t_commit_k"       => { $it | update __k "t_commit_1" | update v { |it| try { $it.v / $it.k } } },
                    "t_build_n"        => { $it | update __k "t_build_1"  | update v { |it| try { $it.v / $it.n } } },
                    "t_verify_n"       => { $it | update __k "t_verify_1" | update v { |it| try { $it.v / $it.n } } },
                    "t_prove_n"        => { $it | update __k "t_prove_1"  | update v { |it| try { $it.v / $it.n } } },
                    "t_commit_m"       => { $it | update __k "t_commit_1" | update v { |it| try { $it.v / $it.m } } },
                    "t_decode_k"
                  | "t_evaluate_kn"
                  | "t_encode_n"
                  | "t_verify_batch_3"
                  | _ => $it,
                }
            }
            | (make plot
                --complete-x $complete_x
                --complete-y $complete_y
                --xticks=$xticks
                --yticks=$yticks
                --cbar=$cbar
                --cmap "inferno"
                --figsize $figsize
                [
                    [p           , __k               ];
                    ["semi-avid" , "t_commit_1"      ],
                    ["semi-avid" , "t_build_1"       ],
                    ["semi-avid" , "t_verify_1"      ],
                    ["kzg"       , "t_commit_1"      ],
                    ["kzg"       , "t_prove_1"       ],
                    ["kzg"       , "t_verify_1"      ],
                  # ["kzg"       , "t_verify_batch_3"],
                    ["aplonk"    , "t_commit_1"      ],
                    ["aplonk"    , "t_prove_1"       ],
                    ["aplonk"    , "t_verify_1"      ],
                  # ["fri"       , "t_evaluate_kn"   ],
                  # ["fri"       , "t_encode_n"      ],
                    ["fri"       , "t_prove_1"       ],
                    ["fri"       , "t_verify_1"      ],
                  # ["fri"       , "t_decode_k"      ],
                ]
            )
    }

    if $stitch and ($nb or $regular or $normalized) {
        ffmpeg grid --output semi-avid.png ([
            (if $nb         { [semi-avid-t_build_n.nb.png , semi-avid-t_commit_k.nb.png , semi-avid-t_verify_n.nb.png] }),
            (if $regular    { [semi-avid-t_build_n.png    , semi-avid-t_commit_k.png    , semi-avid-t_verify_n.png   ] }),
            (if $normalized { [semi-avid-t_build_1.png    , semi-avid-t_commit_1.png    , semi-avid-t_verify_1.png   ] }),
        ] | compact)

        ffmpeg grid --output kzg.png ([
            (if $nb         { [kzg-t_commit_m.nb.png , kzg-t_prove_n.nb.png , kzg-t_verify_n.nb.png ] }),
            (if $regular    { [kzg-t_commit_m.png    , kzg-t_prove_n.png    , kzg-t_verify_n.png    ] }),
            (if $normalized { [kzg-t_commit_1.png    , kzg-t_prove_1.png    , kzg-t_verify_1.png    ] }),
        ] | compact)

        ffmpeg grid --output aplonk.png ([
            (if $nb         { [aplonk-t_commit_m.nb.png , aplonk-t_prove_n.nb.png , aplonk-t_verify_n.nb.png ] }),
            (if $regular    { [aplonk-t_commit_m.png    , aplonk-t_prove_n.png    , aplonk-t_verify_n.png    ] }),
            (if $normalized { [aplonk-t_commit_1.png    , aplonk-t_prove_1.png    , aplonk-t_verify_1.png    ] }),
        ] | compact)

        ffmpeg grid --output fri.png ([
            (if $nb         { [fri-t_prove_n.nb.png , fri-t_verify_n.nb.png ] }),
            (if $regular    { [fri-t_prove_n.png    , fri-t_verify_n.png    ] }),
            (if $normalized { [fri-t_prove_1.png    , fri-t_verify_1.png    ] }),
        ] | compact)
    } else {
        log warning "nothing to stitch"
    }

    if $compare {
        def sort-by-keys []: [ record -> record ] {
            transpose k v | sort-by k | transpose --header-row | into record
        }

        let data = $data
            | reject build git cpu curve seed
            | make compact-results
            | update v { try { math sum } }
            | where ([
                (not ($it.p == "fri" and $it.__k == "t_evaluate_kn"   ))
                (not ($it.p == "fri" and $it.__k == "t_decode_k"      ))
                (not ($it.p == "fri" and $it.__k == "t_encode_n"      ))
                (not ($it.p == "kzg" and $it.__k == "t_verify_batch_3"))
            ] | all { $in })
            | insert step {
                match [$in.p, $in.__k] {
                    ["semi-avid", "t_commit_k"] => "commit",
                    ["semi-avid", "t_build_n" ] => "prove",
                    ["semi-avid", "t_verify_n"] => "verify",
                    ["kzg"      , "t_commit_m"] => "commit",
                    ["kzg"      , "t_prove_n" ] => "prove",
                    ["kzg"      , "t_verify_n"] => "verify",
                    ["aplonk"   , "t_commit_m"] => "commit",
                    ["aplonk"   , "t_prove_n" ] => "prove",
                    ["aplonk"   , "t_verify_n"] => "verify",
                    ["fri"      , "t_prove_n" ] => "prove",
                    ["fri"      , "t_verify_n"] => "verify",
                }
            }
            | group-by --to-table { select k n bytes step | sort-by-keys | to nuon | hash sha256 }
            | each {
                let len = $in.items.v | compact | length
                let sum = $in.items.v | compact | math sum
                let max = $in.items.v | compact | math max
                $in.items | update v { |it| try { {
                    w: ( $it.v         /  $sum               ),
                    b: (($it.v - $max) / ($sum - $len * $max)),
                } } }
            }
            | each { {
                ...($in | reject p v __k | uniq | into record),
                ...($in
                    | select p v
                    | transpose --header-row
                    | into record
                    | default null semi-avid
                    | default null kzg
                    | default null aplonk
                    | default null fri
                )
            } }

        def into_matrix []: [
            table<k: int, bytes: int, color: record<r: int, g: int, b: int>>
            ->
            list<list<list<int>>>
        ] {
            select bytes k color
                | rename x y z
                | sort-by y
                | reverse
                | group-by --to-table y
                | each { $in.items | sort-by x | get z }
                | each { each { [$in.r, $in.g, $in.b, 255] } }
        }

        for step in [ "commit", "prove", "verify" ] {
            for x in [
                [name, fn];
                ["worst-blend", { blend-color $in --worst }],
                ["best-blend" , { blend-color $in         }],
                ["worst"      , { uniq-color  $in --worst }],
                ["best"       , { uniq-color  $in         }],
            ] {
                $data
                    | where step == $step
                    | insert color { do $x.fn }
                    | into_matrix
                    | uv run benchmarks/imshow.py ($in | to json) --figsize ($in.0 | length) ($in | length) --dpi 150 --save $"($step)-($x.name).png"
            }
        }

        $PROTOCOLS
            | values
            | get color
            | each { [[$in.r, $in.g, $in.b, 255]] }
            | uv run benchmarks/imshow.py ($in | to json) --figsize ($in.0 | length) ($in | length) --dpi 150 --save $"legend.png"

        if $stitch {
            ffmpeg grid --output cmp.png [
                [commit-best.png        , prove-best.png        , verify-best.png       ],
                [commit-best-blend.png  , prove-best-blend.png  , verify-best-blend.png ],
                [commit-worst.png       , prove-worst.png       , verify-worst.png      ],
                [commit-worst-blend.png , prove-worst-blend.png , verify-worst-blend.png],
            ]
        }
    }
}
