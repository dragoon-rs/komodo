use error.nu "error throw"

export const WHITE = { r: 1.0, g: 1.0, b: 1.0 }
export const BLACK = { r: 0.0, g: 0.0, b: 0.0 }
export const RED = { r: 1.0, g: 0.0, b: 0.0 }
export const GREEN = { r: 0.0, g: 1.0, b: 0.0 }
export const BLUE = { r: 0.0, g: 0.0, b: 1.0 }

export def "color from-floats" [
    r: float,
    g: float,
    b: float
]: nothing -> record<r: float, g: float, b: float> {
    if $r < 0.0 or $r > 1.0 {
        error throw {
            err: "invalid RGB channel",
            label: $"should be between 0 and 1, found ($r)",
            span: (metadata $r).span,
        }
    }
    if $g < 0.0 or $g > 1.0 {
        error throw {
            err: "invalid RGB channel",
            label: $"should be between 0 and 1, found ($g)",
            span: (metadata $g).span,
        }
    }
    if $b < 0.0 or $b > 1.0 {
        error throw {
            err: "invalid RGB channel",
            label: $"should be between 0 and 1, found ($b)",
            span: (metadata $b).span,
        }
    }

    { r: $r, g: $g, b: $b }
}

export def "color from-ints" [
    r: int,
    g: int,
    b: int
]: nothing -> record<r: float, g: float, b: float> {
    if $r < 0 or $r > 255 {
        error throw {
            err: "invalid RGB channel",
            label: $"should be between 0 and 255, found ($r)",
            span: (metadata $r).span,
        }
    }
    if $g < 0 or $g > 255 {
        error throw {
            err: "invalid RGB channel",
            label: $"should be between 0 and 255, found ($g)",
            span: (metadata $g).span,
        }
    }
    if $b < 0 or $b > 255 {
        error throw {
            err: "invalid RGB channel",
            label: $"should be between 0 and 255, found ($b)",
            span: (metadata $b).span,
        }
    }

    { r: ($r / 255 | into float), g: ($g / 255 | into float), b: ($b / 255 | into float) }
}

def try-string-to-int []: string -> int {
    try {
        $"0x($in)" | into int
    } catch {
        get debug | parse --regex 'CantConvert { to_type: "(?<to>.*)", from_type: "(?<from>.*)", span: Span { (?<span>.*) }, help: Some\("(?<help>.*)"\) }' | into record | error make --unspanned { msg: ($in.help | str replace --all '\"' '"') }
    }
}

export def "color from-string" [s: string]: nothing -> record<r: float, g: float, b: float> {
    let res = $s
        | parse --regex '^#(?<r>..)(?<g>..)(?<b>..)$'
        | into record

    if $res == {} {
        error throw {
            err: "invalid HEX color format",
            label: $"format should be '#RRGGBB', found ($s)",
            span: (metadata $s).span,
        }
    }

    {
        r: ($res.r | try-string-to-int | $in / 255),
        g: ($res.g | try-string-to-int | $in / 255),
        b: ($res.b | try-string-to-int | $in / 255),
    }
}

export def "color mix" [
    c1: record<r: float, g: float, b: float>,
    c2: record<r: float, g: float, b: float>,
    c: float,
]: nothing -> record<r: float, g: float, b: float> {
    {
        r: ($c * $c1.r + (1 - $c) * $c2.r),
        g: ($c * $c1.g + (1 - $c) * $c2.g),
        b: ($c * $c1.b + (1 - $c) * $c2.b),
    }
}

def float-to-u8-hex []: float -> string {
    $in * 255
        | math round --precision 0
        | into int
        | fmt
        | get lowerhex
        | parse "0x{n}"
        | into record
        | get n
        | into string
        | fill --alignment "right" --character '0' --width 2
}

export def "color to-hex" []: record<r: float, g: float, b: float> -> string {
    $"#($in.r | float-to-u8-hex)($in.g | float-to-u8-hex)($in.b | float-to-u8-hex)"
}

