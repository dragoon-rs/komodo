let files = ^git lf | lines | where { ($in | path parse).extension == "nu" }

for f in $files {
    let path = $env.FILE_PWD | path dirname | path join $f

    try {
        nu-check $path --debug
    } catch { |e|
        let err = $e.debug
            | ansi strip
            | parse --regex '(.*)msg: "Found : (?<msg>.*)", span: (?<span>.*)'
            | into record

        error make --unspanned {
            msg: ([
                $"(ansi red_bold)($e.msg)(ansi reset):",
                $"    file: (ansi purple)($f)(ansi reset)",
                $"    err:  ($err.msg)",
            ] | str join "\n")
        }
    }
}
