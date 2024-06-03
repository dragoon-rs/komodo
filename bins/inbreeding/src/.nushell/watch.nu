use consts.nu CACHE
use path.nu [ "remove-cache-prefix" ]

def pretty-hash []: [ string -> string ] {
    str substring 0..7
}

export def main [] {
    watch $CACHE { |op, path, new_path|
        if $op != "Create" {
            return
        }

        let path = $path | remove-cache-prefix

        let now = date now | format date "%Y-%m-%d %H:%M:%S"
        let p = $path | parse "{seed}/{exp}/{id}" | into record

        if $p == {} {
            let p = $path | parse "{seed}/{exp}" | into record
            if $p == {} {
                return $path
            }

            return $"($p.seed | pretty-hash)  ($p.exp)            at ($now)"
        }

        $"($p.seed | pretty-hash)  ($p.exp)  ($p.id | pretty-hash)   at ($now)"
    }
}
