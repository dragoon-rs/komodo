const MODULES = [
    "nu-utils/math.nu",
    "nu-utils/formats.nu",
    "benchmarks/",
]

def log-load [m: string] {
    print $"[(ansi cyan_bold).env.nu(ansi reset)] loading (ansi purple)($m)(ansi reset)"
}

log-load $MODULES.0
use $MODULES.0 *
log-load $MODULES.1
use $MODULES.1 *
log-load $MODULES.2
use $MODULES.2
