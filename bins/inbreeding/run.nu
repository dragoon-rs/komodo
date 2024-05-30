use consts.nu
use ../../.nushell error "error throw"

const VALID_HEX_CHARS = "abcdefABCDEF0123456789"

def check-hex [-n: int]: [
    string -> record<
        ok: bool,
        err: record<msg: string, label: string, help: string>,
    >
] {
    let s = $in

    if ($s | str length) != $n {
        return {
            ok: false,
            err: {
                msg: "invalid HEX length"
                label : $"length is ($s | str length)",
                help: "length should be 64",
            },
        }
    }

    for c in ($s | split chars | enumerate) {
        if not ($VALID_HEX_CHARS | str contains $c.item) {
            return {
                ok: false,
                err: {
                    msg: "bad HEX character",
                    label: $"found '($c.item)' at ($c.index)",
                    help: $"expected one of '($VALID_HEX_CHARS)'",
                },
            }
        }
    }

    { ok: true, err: {} }
}

export def main [
    --options: record<
        nb_bytes: int,
        k: int,
        n: int,
        nb_measurements: int,
        nb_scenarii: int,
        measurement_schedule: int,
        measurement_schedule_start: int,
        max_t: int,
        strategies: list<string>,
        environment: string,
    >,
    --prng-seed: string = "0000000000000000000000000000000000000000000000000000000000000000",
] {
    if $options.measurement_schedule_start > $options.max_t {
        error make --unspanned {
            msg: $"measurement schedule will start after the max t, ($options.measurement_schedule_start) > ($options.max_t)"
        }
    }

    let res = $prng_seed | check-hex -n 64
    if not $res.ok {
        error throw {
            err: $res.err.msg,
            label: $res.err.label,
            span: (metadata $prng_seed).span,
            help: $res.err.help,
        }
    }

    let exp_hash = $options | reject strategies | sort | to nuon | hash sha256

    for s in $options.strategies {
        let output_dir = [
            $consts.CACHE,
            $"($prng_seed)",
            ([$options.environment, $s, $options.k, $options.n, $options.nb_bytes] | str join '-')
        ] | path join
        mkdir $output_dir
        print $"data will be dumped to `($output_dir)`"

        for i in 1..$options.nb_scenarii {
            let seed = [ $prng_seed, $exp_hash, $s, $i ] | str join | hash sha256
            let output = [ $output_dir, $seed ] | path join

            ^$consts.BIN ...[
                $options.nb_bytes,
                -k $options.k
                -n $options.n
                --nb-measurements $options.nb_measurements
                --measurement-schedule $options.measurement_schedule
                --measurement-schedule-start $options.measurement_schedule_start
                -t $options.max_t
                --test-case recoding
                --strategy $s
                --environment $options.environment
                --prng-seed $seed
            ] out> $output
        }

        print $"data has been dumped to `($output_dir)`"
    }
}
