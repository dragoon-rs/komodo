# throws an error if the input is an empty list
export def check-list-arg [
    --cmd: string, # the name of the command
    --arg: string, # the name of the argument
    --span: record<start: int, end: int>, # the span of the arg (no span means an unspanned error)
]: [ list -> nothing ] {
    if ($in | is-empty) {
        if $span == null {
            error make --unspanned {
                msg: $"(ansi red_bold)invalid_arguments(ansi reset)",
                help: $"provide a non empty list as ($arg)",
            }
        } else {
            error make {
                msg: $"(ansi red_bold)invalid_arguments(ansi reset)",
                label: {
                    text: $"(ansi purple)($cmd)(ansi reset) needs (ansi purple)($arg)(ansi reset)",
                    span: $span
                },
                help: $"provide a non empty list as (ansi purple)($arg)(ansi reset)"
            }
        }
    }
}
