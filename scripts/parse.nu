export def read-atomic-ops [
    --include: list<string> = [], --exclude: list<string> = []
]: list -> record {
    let raw = $in
        | insert t {|it| $it.times |math avg}
        | reject times
        | rename --column { label: "group", name: "species", t: "measurement" }

    let included = if $include != [] {
        $raw | where group in $include
    } else {
        $raw
    }

    $included
        | where group not-in $exclude
        | group-by group --to-table
        | reject items.group
        | update items { transpose -r | into record }
        | transpose -r
        | into record
}
