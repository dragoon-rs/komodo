export def info [msg: string] {
    print $"[(ansi green_bold)INFO(ansi reset)] ($msg)"
}

export def warning [msg: string] {
    print $"[(ansi yellow_bold)WARNING(ansi reset)] ($msg)"
}
