export def "error throw" [err: record<err: string, label: string, span: record<start: int, end: int>>] {
    error make {
        msg: $"(ansi red_bold)($err.err)(ansi reset)",
        label: {
            text: $err.label,
            span: $err.span,
        },
    }
}
