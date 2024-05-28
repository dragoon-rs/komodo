export def check-file [file: path, --span: record<start: int, end: int>] {
    if not ($file | path exists) {
        error make {
            msg: "invalid path",
            label: {
                text: "no such file",
                span: $span,
            }
        }
    }
}
