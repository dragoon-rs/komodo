# Welcome to Komodo, a tool to encode and prove data.
#
# please run `komodo --help` or `komodo <tab>` to have a look at more information

use nu-utils binary ["bytes from_int"]

const KOMODO_BINARY = "./target/release/komodo"
const DEFAULT_LOG_LEVEL = "INFO"

def home-dir []: nothing -> path {
    $env.KOMODO_HOME? | default (
        $env.XDG_DATA_HOME? | default "~/.local/share" | path join "komodo"
    ) | path expand
}

def block-dir []: nothing -> path {
    home-dir | path join "blocks"
}

def "nu-complete log-levels" []: nothing -> list<string> {
    [
        "TRACE"
        "DEBUG",
        "INFO",
        "WARN",
        "ERROR",
    ]
}

def "nu-complete encoding-methods" []: nothing -> list<string> {
    [
        "vandermonde"
        "random",
    ]
}

def run-komodo [
    --input: path = "",
    --nb-bytes: int = 0,
    -k: int = 0,
    -n: int = 0,
    --generate-powers,
    --reconstruct,
    --verify,
    --combine,
    --inspect,
    --encoding-method: string = "",
    --log-level: string,
    ...block_hashes: string,
]: nothing -> any {
    let home_dir = home-dir
    if not ($home_dir | is-empty) {
        mkdir $home_dir
    }
    let block_dir = block-dir
    if not ($block_dir | is-empty) {
        mkdir $block_dir
    }

    with-env {RUST_LOG: $log_level} {
        let res = do {
            ^$KOMODO_BINARY ...([
                $input
                $k
                $n
                ($generate_powers | into string)
                $home_dir
                ($reconstruct | into string)
                ($verify | into string)
                ($combine | into string)
                ($inspect | into string)
                $nb_bytes
                $encoding_method
            ] | append $block_hashes)
        } | complete

        print --no-newline $res.stdout
        if $res.exit_code != 0 {
            error make --unspanned { msg: $"($res.stderr) \(($res.exit_code)\)" }
        }
        $res.stderr | from json
    }
}

def list-blocks []: nothing -> list<string> {
    try {
        ls (block-dir) | get name | path parse | get stem
    } catch {
        []
    }
}

# build Komodo from source, updating the application
export def "komodo build" []: nothing -> nothing {
    ^cargo build --package komodo --release
}

# create a random trusted setup for a given amount of data
#
# # Examples
# ```nushell
# # create a trusted setup well suited for a file called `my_target_file.txt`
# komodo setup (open my_target_file.txt | into binary | bytes length)
# ```
# ---
# ```nushell
# # create a trusted setup for 50k bytes and make sure the setup has been created
# komodo setup 50_000
# use std assert; assert ("~/.local/share/komodo/powers" | path exists)
export def "komodo setup" [
    nb_bytes: int, # the size of the biggest expected data during the lifetime of the application
    --log-level: string@"nu-complete log-levels" = $DEFAULT_LOG_LEVEL # change the log level
]: nothing -> nothing {
    (
        run-komodo
            --log-level $log_level
            --nb-bytes $nb_bytes
            --generate-powers
    )
}

# encode and _prove_ a bunch of input bytes
#
# # Examples
# ```nushell
# # encode and prove `tests/dragoon_32x32.png` with a _3 x 5_ Vandermonde encoding
# komodo prove tests/dragoon_32x32.png --fec-params {k: 3, n: 5} --encoding-method vandermonde
# ```
# ```
# ─┬────────────────────────────────────────────────────────────────
# 0│44614daf1f5ebb86f1c69293b82c7795a5a35b4d12718b551648223441028e3
# 1│8be575889246fbc49f4c748ac2dc1cd8a4ef71d16e91c9343660a5f79f086
# 2│6de9fd5fdfe8c08b3132e0d527b14a2a4e4be9a543af1f13d2c397bd113846e4
# 3│f1c34065cbfc3267f9d41558a465ba6335fd45229ff2eae5b34a8f30467562
# 4│7aa698f338605462205c5ff46b5463720d073de92a19f897cc4ae6c286ab87
# ─┴────────────────────────────────────────────────────────────────
# ```
export def "komodo prove" [
    input: path, # the path to the input file to encode and prove
    --fec-params: record<k: int, n: int>, # the parameters of the encoding
    --encoding-method: string@"nu-complete encoding-methods" = "random", # the encoding method, e.g. _random_ or _vandermonde_
    --log-level: string@"nu-complete log-levels" = $DEFAULT_LOG_LEVEL # change the log level
]: nothing -> list<string> {
    # NOTE: the next two runtime checks on the type of `--fec-params` might be
    # a bug on the Nushell side
    if $fec_params == null {
        error make --unspanned {
            msg: "`komodo prove` requires `--fec-params` to be given"
        }
    }

    let type = $fec_params | describe --detailed | update columns { sort }
    let expected = { type: record, columns: { k: int, n: int } }
    if $type != $expected {
        error make {
            msg: $"(ansi red_bold)invalid `--fec-params`(ansi reset)",
            label: {
                text: $"expected ($expected) got ($type)",
                span: (metadata $fec_params).span,
            }
        }
    }

    (
        run-komodo
            --log-level $log_level
            --input $input
            -k $fec_params.k
            -n $fec_params.n
            --encoding-method $encoding_method
    )
}

# verify the integrity of any number of blocks
#
# # Examples
# ```nushell
# # verify the integrity of two blocks (note the use of the spread operator introduced in Nushell 0.89.0)
# # > **Note**
# # > file: `tests/dragoon_32x32.png`
# # > parameters: k = 3 and n = 5
# # > method: vandermonde
# komodo verify ...[
#     44614daf1f5ebb86f1c69293b82c7795a5a35b4d12718b551648223441028e3,
#     7aa698f338605462205c5ff46b5463720d073de92a19f897cc4ae6c286ab87,
# ]
# ```
# ```
# #┬─────────────────────────────block─────────────────────────────┬status
# 0│44614daf1f5ebb86f1c69293b82c7795a5a35b4d12718b551648223441028e3│true
# 1│7aa698f338605462205c5ff46b5463720d073de92a19f897cc4ae6c286ab87 │true
# ─┴───────────────────────────────────────────────────────────────┴──────
# ```
export def "komodo verify" [
    ...blocks: string@"list-blocks", # the list of blocks to verify
    --log-level: string@"nu-complete log-levels" = $DEFAULT_LOG_LEVEL # change the log level
]: nothing -> table<block: string, status: int> {
    run-komodo --log-level $log_level --verify ...$blocks
}

# reconstruct the original data from a subset of blocks
#
# `komodo reconstruct` might throw an error in some cases
# - when there are too few blocks
# - when the blocks are linearly dependant, and thus the decoding cannot be applied
# - when the blocks belong to different data
#
# # Examples
# ```nushell
# # applying a valid reconstruction
# # > **Note**
# # > file: `tests/dragoon_32x32.png`
# # > parameters: k = 3 and n = 5
# # > method: vandermonde
# let bytes = komodo reconstruct ...[
#     44614daf1f5ebb86f1c69293b82c7795a5a35b4d12718b551648223441028e3,
#     7aa698f338605462205c5ff46b5463720d073de92a19f897cc4ae6c286ab87,
#     8be575889246fbc49f4c748ac2dc1cd8a4ef71d16e91c9343660a5f79f086,
# ]
# $bytes | bytes at 0..10
# ```
# ```
# Length: 10 (0xa) bytes | printable whitespace ascii_other non_ascii
# 00000000:   89 50 4e 47  0d 0a 1a 0a  00 00                      ×PNG__•_00
# ```
# ---
# ```nushell
# # giving too few blocks
# # > **Note**
# # > file: `tests/dragoon_32x32.png`
# # > parameters: k = 3 and n = 5
# # > method: vandermonde
# komodo reconstruct ...[
#     44614daf1f5ebb86f1c69293b82c7795a5a35b4d12718b551648223441028e3,
#     7aa698f338605462205c5ff46b5463720d073de92a19f897cc4ae6c286ab87,
# ]
# ```
# ```
# Error:   × could not decode: Expected at least 3, got 2 (1)
# ```
# ---
# ```nushell
# # after combining _44614d_ and _6de9fd_ (see [`komodo combine`]), try to decode with linear dependencies
# # > **Note**
# # > file: `tests/dragoon_32x32.png`
# # > parameters: k = 3 and n = 5
# # > method: vandermonde
# # > recoding: _44614d_ <+> _6de9fd_ => _86cdd1_
# komodo reconstruct ...[
#     44614daf1f5ebb86f1c69293b82c7795a5a35b4d12718b551648223441028e3,
#     6de9fd5fdfe8c08b3132e0d527b14a2a4e4be9a543af1f13d2c397bd113846e4,
#     86cdd1b7ed79618696ab82d848833cbe448719a513b850207936e4dce6294,
# ]
# ```
# ```
# Error:   × could not decode: Matrix is not invertible at row 2 (1)
# ```
export def "komodo reconstruct" [
    ...blocks: string@"list-blocks", # the blocks that should be used to reconstruct the original data
    --log-level: string@"nu-complete log-levels" = $DEFAULT_LOG_LEVEL # change the log level
]: nothing -> binary {
    run-komodo --log-level $log_level --reconstruct ...$blocks | bytes from_int
}

# combine two blocks by computing a random linear combination
#
# # Examples
# # > **Note**
# # > file: `tests/dragoon_133x133.png`
# # > parameters: k = 7 and n = 23
# # > method: random
# ```nushell
# komodo combine ...[
#     1b112a11cd89dad619aadc18cb2c15c315453e177f1117c79d4ae4e219922,
#     31c9bfe2845cc430d666413d8b8b51aee0d010aa89275a8c7d9d9ca1c9e05c,
# ]
# ```
# ```
# b785bf5b93d7811792db7234d1b1ee7347398cee617243612d3225fa245545
# ```
# ---
# ```nushell
# # not giving exactly 2 blocks
# # > **Note**
# # > file: `tests/dragoon_133x133.png`
# # > parameters: k = 7 and n = 23
# # > method: random
# komodo combine ...[
#     c22fe3c72cbc52fc55b46a3f9783f5c9a1e5fb59875f736332cf1b970b8,
#     1b112a11cd89dad619aadc18cb2c15c315453e177f1117c79d4ae4e219922,
#     f3f423df47cd7538accd38abe9ad6670b894243647af98fbfa9776e9cf7ff8e,
# ]
# ```
# ```
# Error:   × expected exactly 2 blocks, found 3 (1)
# ```
export def "komodo combine" [
    ...blocks: string@"list-blocks", # the blocks to combine, should contain two hashes
    --log-level: string@"nu-complete log-levels" = $DEFAULT_LOG_LEVEL # change the log level
]: nothing -> string {
    run-komodo --log-level $log_level --combine ...$blocks | get 0
}

# open one or more blocks and inspect their content
#
# # Examples
# ```nushell
# # inspect a single block
# # # > **Note**
# # # > file: `tests/dragoon_133x133.png`
# # # > parameters: k = 7 and n = 23
# # # > method: random
# # # >
# # # > `$.commits` and `$shard.bytes` have been truncated for readability
# komodo inspect 374f23fd1f25ae4050c414bc169550bdd10f49f775e2af71d2aee8a87dc
# | into record
# | update commits { parse --regex '\((?<f>\d{7})\d+, (?<s>\d{7})\d+\)' }
# | update shard.bytes { length }
# ```
# ```
# ───────┬──────────────────────────────────────────────────────────────────────
#        │─────┬────────────────────────────────────────────────────────────────
# shard  │k    │7
#        │     │─┬───────────────────────────────────────
#        │comb │0│79293070128283035155183921571762200246
#        │     │1│251822311562506197186167674369775071480
#        │     │2│271375591445361086306725794586025747695
#        │     │3│170387538935153872006296956270886059735
#        │     │4│67758248758369211569040944574905941217
#        │     │5│245908698054074962369032280439731463970
#        │     │6│323120636634748190275497410128071523309
#        │     │─┴───────────────────────────────────────
#        │bytes│89
#        │hash │d116d9e2bdb0e03fb2bdd6e716a929198f1012ae62a83e773eb2c21917f4b12c
#        │size │19102
#        │─────┴────────────────────────────────────────────────────────────────
#        │#┬───f───┬───s───
# commits│0│3258872│1117914
#        │1│1336159│2841207
#        │2│3908964│4603563
#        │3│3956175│1154567
#        │4│2056568│3904956
#        │5│2957425│2772456
#        │6│2395336│2282274
#        │─┴───────┴───────
# m      │89
# ───────┴──────────────────────────────────────────────────────────────────────
# ```
export def "komodo inspect" [
    ...blocks: string@"list-blocks", # the blocks to inspect
    --log-level: string@"nu-complete log-levels" = $DEFAULT_LOG_LEVEL # change the log level
]: nothing -> table<shard: record<k: int, comb: list<any>, bytes: list<string>, hash: string, size: int>, commits: list<string>, m: int> {
    run-komodo --log-level $log_level --inspect ...$blocks
}

# list all the blocks that are currently in the store
export def "komodo ls" []: nothing -> list<string> {
    list-blocks
}

# clean the Komodo home from all blocks and trusted setup
export def "komodo clean" []: nothing -> nothing {
    rm --force --recursive (home-dir)
}

def pretty-code []: string -> string {
    $"`(ansi default_dimmed)($in)(ansi reset)`"
}

# the main entry point of Komodo, will only print some help
export def main []: nothing -> nothing {
    let help = [
        $"the location of the files generated by Komodo can be configured via ("$env.KOMODO_HOME" | pretty-code) which will default to",
        $"- ("$env.XDG_DATA_HOME/komodo" | pretty-code) if ("$env.XDG_DATA_HOME" | pretty-code) is set",
        $"- ("~/.local/share/komodo/" | pretty-code) otherwise"
    ]

    print ($help | str join "\n")
}
