#!/usr/bin/env nu

use options.nu

const BIN = "./target/release/examples/inbreeding"

def main [--output: path = "./inbreeding.nuon", --baseline] {
    if $baseline {
        ^$BIN ...[
            $options.NB_BYTES,
            -k $options.K
            -n $options.N
            --nb-measurements $options.NB_MEASUREMENTS
            --measurement-schedule $options.MEASUREMENT_SCHEDULE
            -t $options.MAX_T
            --test-case end-to-end
        ] | lines | into float | save --force baseline.nuon

        print "baseline saved to `baseline.nuon`"
    }

    let strategies = seq 1 $options.K | each { $"single:($in)" } | append $options.EXTRA_STRATEGIES

    $strategies | each {|s|
        let diversity = ^$BIN ...[
            $options.NB_BYTES,
            -k $options.K
            -n $options.N
            --nb-measurements $options.NB_MEASUREMENTS
            --measurement-schedule $options.MEASUREMENT_SCHEDULE
            -t $options.MAX_T
            --test-case recoding
            --strategy $s
            --environment $options.ENVIRONMENT
        ] | lines | into float

        {
            strategy: $s,
            diversity: $diversity,
        }
    } | save --force $output

    print $"results saved to `($output)`"
}
