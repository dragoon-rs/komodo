const MODULES = [
    "benchmarks/",
]

def log-load [m: string] {
    print $"[(ansi cyan_bold).env.nu(ansi reset)] loading (ansi purple)($m)(ansi reset)"
}

log-load $MODULES.0
use $MODULES.0
