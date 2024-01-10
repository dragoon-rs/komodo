const KOMODO_BINARY = "./target/release/komodo"

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
    args: record<bytes: path, k: int, n: int, do_generate_powers: bool, powers_file: path, do_reconstruct_data: bool, do_verify_blocks: bool, block_files: list<string>>,
    --log-level: string,
]: nothing -> any {
    with-env {RUST_LOG: $log_level} {
        let res = do {
            ^$KOMODO_BINARY ...([
                $args.bytes
                $args.k
                $args.n
                ($args.do_generate_powers | into string)
                $args.powers_file
                ($args.do_reconstruct_data | into string)
                ($args.do_verify_blocks | into string)
            ] | append $args.block_files)
        } | complete

        print $res.stdout
        $res.stderr | from json
    }
}

export def "komodo build" [] {
    ^cargo build --package komodo --release
}

export def "komodo setup" [
    bytes: path,
    --powers-file: path = "powers.bin",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> nothing {
    run-komodo --log-level $log_level {
        bytes: $bytes,
        k: 0,
        n: 0,
        do_generate_powers: true,
        powers_file: $powers_file,
        do_reconstruct_data: false,
        do_verify_blocks: false,
        block_files: [],
    }
}

export def "komodo prove" [
    bytes: path,
    --fec-params: record<k: int, n: int>,
    --powers-file: path = "powers.bin",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> list<string> {
    run-komodo --log-level $log_level {
        bytes: $bytes,
        k: $fec_params.k,
        n: $fec_params.n,
        do_generate_powers: false,
        powers_file: $powers_file,
        do_reconstruct_data: false,
        do_verify_blocks: false,
        block_files: [],
    }
}

export def "komodo verify" [
    ...blocks: path,
    --powers-file: path = "powers.bin",
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> table<block: string, status: int> {
    run-komodo --log-level $log_level {
        bytes: "",
        k: 0,
        n: 0,
        do_generate_powers: false,
        powers_file: $powers_file,
        do_reconstruct_data: false,
        do_verify_blocks: true,
        block_files: $blocks,
    }
}

export def "komodo reconstruct" [
    ...blocks: path,
    --log-level: string@"nu-complete log-levels" = "INFO"
]: nothing -> list<int> {
    run-komodo --log-level $log_level {
        bytes: "",
        k: 0,
        n: 0,
        do_generate_powers: false,
        powers_file: "",
        do_reconstruct_data: true,
        do_verify_blocks: false,
        block_files: $blocks,
    }
}
