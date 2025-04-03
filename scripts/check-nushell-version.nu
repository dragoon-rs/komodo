let config = open .nu.cfg
    | lines
    | parse "{key}: {value}"
    | transpose --header-row
    | into record
if (version).commit_hash != $config.REVISION or (version).version != $config.VERSION {
    print --stderr $"(ansi yellow_bold)Warning(ansi reset): unexpected version"
    print --stderr $"    expected (ansi green)($config.VERSION)@($config.REVISION)(ansi reset)"
    print --stderr $"    found    (ansi red)((version).version)@((version).commit_hash)(ansi reset)"
}
