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
        max_t: int,
        strategies: list<string>,
        environment: string,
    >
] {
    if $baseline {
        ^$BIN ...[
            $options.nb_bytes,
            -k $options.k
            -n $options.n
            --nb-measurements $options.nb_measurements
            --measurement-schedule $options.measurement_schedule
            -t $options.max_t
            --test-case end-to-end
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
                -t $options.max_t
                --test-case recoding
                --strategy $s
                --environment $options.environment
            ] | lines | into float
        }

        let diversity = $res
            | skip 1
            | reduce --fold $res.0 {|it, acc| $acc | zip $it | each { flatten }}
            | each { math avg }

        {
            strategy: $s,
            diversity: $diversity,
        }
    } | save --force $output

    print $"results saved to `($output)`"
}
