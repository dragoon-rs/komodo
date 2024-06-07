use ../src/.nushell/consts.nu CACHE
use ../

const FIGURES_DIR = ($CACHE | path join figures)

mkdir $FIGURES_DIR

for exp in (inbreeding list) {
    let img = $FIGURES_DIR | path join $exp | path parse --extension '' | update extension "png" | path join
    inbreeding load $exp | inbreeding plot --save $img
}
