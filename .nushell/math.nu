export def ns-to-ms [col: cell-path]: table -> table {
    update $col { each { $in / 1_000_000 } }
}

export def compute-stats [col: cell-path]: table -> table<mean: float, stddev: float> {
    insert mean {|it| $it | get $col | math avg}
    | insert stddev {|it| $it | get $col | into float | math stddev}
}
