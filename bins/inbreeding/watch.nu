use consts.nu CACHE
use path.nu [ "remove-cache-prefix" ]

export def main [] {
    watch $CACHE { |op, path, new_path|
        if $op != "Create" {
            return
        }

        let now = date now | format date "%Y-%m-%d %H:%M:%S"
        let p = $path | remove-cache-prefix | parse "{seed}/{exp}/{id}" | into record

        if $p == {} {
            let p = $path | remove-cache-prefix | parse "{seed}/{exp}" | into record
            return $"($p.seed | str substring 0..7)  ($p.exp)            at ($now)"
        }

        $"($p.seed | str substring 0..7)  ($p.exp)  ($p.id | str substring 0..7)   at ($now)"
    }
}
