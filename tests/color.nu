use ../nu-utils color [
    "color from-floats",
    "color from-ints",
    "color from-string",
    "color mix",
    "color to-hex",
    RED,
    GREEN,
]

use std assert

def "assert error" [code: closure, --msg: string, --body: string] {
    try {
        do $code
    } catch { |e|
        if $msg != null and ($e.msg | ansi strip) != $msg {
            error make {
                msg: $"(ansi red_bold)assertion: bad error message(ansi reset)",
                label: {
                    text: $"error should have message '($msg)'",
                    span: (metadata $code).span,
                },
                help: $"actual: ($e.msg | ansi strip)",
            }
        }
        if $body != null and not ($e.debug | ansi strip | str contains $body) {
            let actual = $e.debug | ansi strip | parse "{foo}text: {text}, span: {bar}" | into record | get text
            error make {
                msg: $"(ansi red_bold)assertion: bad error body(ansi reset)",
                label: {
                    text: $"error should contain '($body)'",
                    span: (metadata $code).span,
                },
                help: $"actual: ($actual)",
            }
        }
        return
    }

    error make --unspanned { msg: "should error" }
}

assert error { || color from-floats 2 1 1 } --msg "invalid RGB channel" --body "should be between 0 and 1, found 2"
assert error { || color from-floats 1 (-2 | into float) 1 } --msg "invalid RGB channel" --body "should be between 0 and 1, found -2"
assert error { || color from-floats 1 1 3.4 } --msg "invalid RGB channel" --body "should be between 0 and 1, found 3.4"

assert error { || color from-ints 256 0 0 } --msg "invalid RGB channel" --body "should be between 0 and 255, found 256"
assert error { || color from-ints 0 256 0 } --msg "invalid RGB channel" --body "should be between 0 and 255, found 256"
assert error { || color from-ints 0 0 256 } --msg "invalid RGB channel" --body "should be between 0 and 255, found 256"

assert error { || color from-string "foo" } --msg "invalid HEX color format" --body "format should be '#RRGGBB', found foo"
assert error { || color from-string "#foo" } --msg "invalid HEX color format" --body "format should be '#RRGGBB', found #foo"

assert error { || color from-string "#xxxxxx" } --msg "hexadecimal digits following \"0x\" should be in 0-9, a-f, or A-F, found xx"
assert error { || color from-string "#0123yy" } --msg "hexadecimal digits following \"0x\" should be in 0-9, a-f, or A-F, found yy"

assert equal (color from-floats 0.1 0.2 0.3) { r: 0.1, g: 0.2, b: 0.3 }
assert equal (color from-ints 1 2 3) { r: (1 / 255), g: (2 / 255), b: (3 / 255) }
assert equal (color from-string "#010203") { r: (1 / 255), g: (2 / 255), b: (3 / 255) }

assert equal (color from-string "#010203" | color to-hex) "#010203"

assert equal (color mix $RED $GREEN 0.5) (color from-floats 0.5 0.5 0.0)
