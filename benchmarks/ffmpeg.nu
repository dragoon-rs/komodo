use ../log.nu [ "log info" ]

def "ffmpeg stack" [...imgs: string, --output (-o): path = "a.png", --stack: string] {
    ffmpeg ...[
        -y
        -hide_banner
        -loglevel warning
        ...($imgs | each { [-i $in] } | flatten)
        -filter_complex $"($stack)=inputs=($imgs | length)"
        $output
    ]
    log info $"Generated ($output)"
}

export def "ffmpeg hstack" [...imgs: string, --output (-o): path] {
    ffmpeg stack ...$imgs --output $output --stack "hstack"
}

export def "ffmpeg vstack" [...imgs: string, --output (-o): path] {
    ffmpeg stack ...$imgs --output $output --stack "vstack"
}

export def "ffmpeg grid" [imgs: list<list<string>>, --output (-o): path] {
    if ($imgs | each { length } | any {|| $in != ($imgs.0 | length)}) {
        error make {
            msg: $"(ansi red_bold)invalid_args(ansi reset)",
            label: {
                text: "not a rectangular grid",
                span: (metadata $imgs).span,
            },
            help: $"rows have following lengths: ($imgs | each { length })",
        }
    }
    if ($imgs | length) == 0 or ($imgs.0 | length) == 0 {
        error make {
            msg: $"(ansi red_bold)invalid_args(ansi reset)",
            label: {
                text: "empty grid",
                span: (metadata $imgs).span,
            },
        }
    }

    match [($imgs | length), ($imgs.0 | length)] {
        [1, 1] => {
            cp $imgs.0.0 $output;
            log info $"Generated ($output) \(by copy\)"
        },
        [1, _] => { ffmpeg hstack --output $output ...$imgs.0 },
        [_, 1] => { ffmpeg vstack --output $output ...($imgs | each { $in.0 }) },
        [_, _] => {
            ffmpeg vstack --output $output ...($imgs | each { |row|
                let output = mktemp --tmpdir XXXXXXX.png
                ffmpeg hstack --output $output ...$row
                $output
            })
        },
    }
}
