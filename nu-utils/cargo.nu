use std log
use error.nu "error throw"

def get-workspace-bins []: nothing -> table<name: string, toml: path> {
    open Cargo.toml
        | get workspace.members
        | each { path join "Cargo.toml" }
        | wrap toml
        | insert name { get toml | open | get package.name }
}

def get-workspace-bin-names []: nothing -> table<value: string, description: string> {
    get-workspace-bins | each {{
        value: $in.name,
        description: ($in.toml | open | get package.description? | default "")
    }}
}

# run a binary from the workspace
export def --wrapped "cargo bin" [
    bin: string@get-workspace-bin-names, # the name of the binary to run, press tab to autocomplete
    --debug, # run in debug mode
    --build, # build the binary in the specified mode
    ...args: string, # arguments to pass to the binary
] {
    let bin_span = (metadata $bin).span

    let bins = get-workspace-bins
    let bin = $bins | where name == $bin | into record

    if $build {
        if $debug {
            cargo build --manifest-path $bin.toml
        } else {
            cargo build --release --manifest-path $bin.toml
        }
    }

    let target = if $debug {
        "debug"
    } else {
        "release"
    }

    let bin = "target" | path join $target $bin.name
    if not ($bin | path exists) {
        error throw {
            err: "binary not found",
            label: "hasn't been compiled, compile it with --build",
            span: $bin_span,
        }
    }

    ^$bin ...$args
}
