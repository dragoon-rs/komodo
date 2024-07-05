const MODULES = [
    ".nushell/math.nu",
    ".nushell/formats.nu",
]

def log-load [m: string] {
    print $"[(ansi cyan_bold).env.nu(ansi reset)] loading (ansi purple)($m)(ansi reset)"
}

log-load $MODULES.0
use $MODULES.0 *
log-load $MODULES.1
use $MODULES.1 *
