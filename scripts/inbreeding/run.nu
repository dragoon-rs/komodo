const BIN = "./target/release/examples/inbreeding"

export def main [
    --output: path = "./inbreeding.nuon",
    --baseline,
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
    if $baseline {
        ^$BIN ...[
            $options.nb_bytes,
            -k $options.k
            -n $options.n
            --nb-measurements $options.nb_measurements
            --measurement-schedule $options.measurement_schedule
            --measurement-schedule-start $options.measurement_schedule_start
            -t $options.max_t
            --test-case end-to-end
            --prng-seed $prng_seed
        ] | lines | into float | save --force baseline.nuon

        print "baseline saved to `baseline.nuon`"
    }

    $options.strategies | each {|s|
        let res = 1..$options.nb_scenarii | each {
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
            ]
                | lines
                | parse "{t}, {diversity}"
                | into float diversity
        }

        let diversity = $res
            | flatten
            | group-by t --to-table
            | update items { get diversity | math avg }
            | rename --column { group: "t", items: "diversity" }
            | into int t # NOTE: $.t needs to be converted to int here because
                         # `group-by --to-table` converts the grouping key to
                         # string

        {
            strategy: $s,
            diversity: $diversity,
        }
    } | save --force $output

    print $"results saved to `($output)`"
}
