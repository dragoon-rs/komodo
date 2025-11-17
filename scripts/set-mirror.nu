def get-pin [--pinentry: string = "tty", --prompt: string, --title: string]: [
    nothing -> string,
    nothing -> nothing,
] {
    let pinentry_bin = $"pinentry-($pinentry)"
    if (which $pinentry_bin | is-empty) {
        let pinentries = $env.PATH
            | each { try { ls $in } catch { [] } }
            | get name
            | flatten
            | tee { print }
            | path parse
            | reject parent
            | path join
            | uniq
            | where $it =~ "^pinentry-"

        error make {
            msg: $"(ansi red_bold)invalid_pinentry(ansi reset)",
            label: {
                text: $"'($pinentry)' is not a valid pinentry",
                span: (metadata $pinentry).span,
            },
            help: $"choose among: ($pinentries)",
        }

    }

    let script = [
        ...(if $title != null {[ $"SETTITLE ($title)" ]} else {[]})
        ...(if $prompt != null {[ $"SETPROMPT ($prompt)" ]} else {[]})
        "GETPIN"
        "BYE"
    ]

    let res = $script
        | str join "\n"
        | ^$pinentry_bin --lc-ctype "UTF-8" -T (tty)
        | lines

    if ($res | last) != "OK" {
        return
    }

    $res
        | parse "D {res}"
        | into record
        | get -i res
        | default ""
}

def main [user: string, password?: string] {
    let password = if $password == null { get-pin --pinentry curses } else { $password }
    glab repo mirror ...[
        https://gitlab.isae-supaero.fr/dragoon/komodo
        --url $"https://($user):($password)@github.com/dragoon-rs/komodo"
        --direction push
        --protected-branches-only
        --enabled
    ]
}
