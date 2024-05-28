const BIN = "./target/release/examples/inbreeding"
const CACHE = ($nu.home-path | path join .cache komodo inbreeding)

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
    --prng-seed: int = 0,
] {
    if $options.measurement_schedule_start > $options.max_t {
        error make --unspanned {
            msg: $"measurement schedule will start after the max t, ($options.measurement_schedule_start) > ($options.max_t)"
        }
    }

    let now = date now | format date "%s%f"

    for s in $options.strategies {
        let output_dir = [ $CACHE, $"($prng_seed)", $now, $options.environment, $"($s)" ] | path join
        mkdir $output_dir
        print $"data will be dumped to `($output_dir)`"

        for i in 1..$options.nb_scenarii {
            let output = [ $output_dir, $"($i)" ] | path join

            ^$BIN ...[
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
                --prng-seed $prng_seed
            ] out> $output
        }

        print $"data has been dumped to `($output_dir)`"
    }
}
