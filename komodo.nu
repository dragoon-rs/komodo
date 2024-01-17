const KOMODO_BINARY = "./target/release/komodo"
const BLOCK_DIR = "blocks/"

def "nu-complete log-levels" []: nothing -> list<string> {
    [
        "TRACE"
        "DEBUG",
        "INFO",
        "WARN",
        "ERROR",
    ]
}

def run-komodo [
    --input: path = "",
    -k: int = 0,
    -n: int = 0,
    --generate-powers,
    --powers-file: path = "",
    --reconstruct,
    --verify,
    --combine,
    --inspect,
    --log-level: string,
    ...block_hashes: string,
]: nothing -> any {
    with-env {RUST_LOG: $log_level} {
        let res = do {
            ^$KOMODO_BINARY ...([
                $input
                $k
                $n
                ($generate_powers | into string)
                $powers_file
                ($reconstruct | into string)
                ($verify | into string)
                ($combine | into string)
                ($inspect | into string)
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
        ls $BLOCK_DIR | get name | path parse | get stem
    } catch {
        []
    }
}

export def "komodo build" []: nothing -> nothing {
    ^cargo build --package komodo --release
}

export def "komodo setup" [
    input: path,
    --powers-file: path = "powers.bin",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> nothing {
    (
        run-komodo
            --log-level $log_level
            --input $input
            --generate-powers
            --powers-file $powers_file
    )
}

export def "komodo prove" [
    input: path,
    --fec-params: record<k: int, n: int>,
    --powers-file: path = "powers.bin",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> list<string> {
    (
        run-komodo
            --log-level $log_level
            --input $input
            -k $fec_params.k
            -n $fec_params.n
            --powers-file $powers_file
    )
}

export def "komodo verify" [
    ...blocks: string@"list-blocks",
    --powers-file: path = "powers.bin",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> table<block: string, status: int> {
    run-komodo --log-level $log_level --powers-file $powers_file --verify ...$blocks
}

export def "komodo reconstruct" [
    ...blocks: string@"list-blocks",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> list<int> {
    run-komodo --log-level $log_level --reconstruct ...$blocks
}

export def "komodo combine" [
    ...blocks: string@"list-blocks",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> string {
    run-komodo --log-level $log_level --combine ...$blocks | get 0
}

export def "komodo inspect" [
    ...blocks: string@"list-blocks",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> table<shard: record<k: int, comb: list<any>, bytes: list<string>, hash: string, size: int>, commits: list<string>, m: int> {
    run-komodo --log-level $log_level --inspect ...$blocks
}

export def "komodo ls" []: nothing -> list<string> {
    list-blocks
}
