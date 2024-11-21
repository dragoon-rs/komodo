let DATA_SIZES = seq 0 15 | each { 2 ** $in * 4096b }
const KS = [8, 128, 1024, 4096]
const BFS = [2, 4]
const NS = [2]
const RPOS = [1]
const QS = [50]
const HS = ["sha3-512"]
const FFS = ["fp128", "bls12-381"]

const DATA = "benchmarks/results/fri.ndjson"
const OUTPUT_DIR = "benchmarks/results/figures/"

if not ($DATA | path dirname | path exists) {
    print $"creating directory for (ansi purple)($DATA)(ansi reset)"
    $DATA | path dirname | mkdir $in
}
