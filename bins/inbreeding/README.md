- build the example for best performance with [`inbreeding build`](./build.nu)
- run the experiment with [`inbreeding run`](./run.nu)
- plot the results with [`inbreeding plot`](./plot.nu)

# Example
```bash
use ./bins/inbreeding
```
```bash
const PRNG_SEED = 123
const OPTS = {
    nb_bytes: (10 * 1_024),
    k: 10,
    n: 20,
    nb_measurements: 10,
    nb_scenarii: 100,
    measurement_schedule: 1,
    measurement_schedule_start: 0,
    max_t: 150,
    strategies: [
        "single:1",
        "double:0.5:1:2",
        "single:2"
        "double:0.5:2:3",
        "single:3"
        "single:5",
        "single:10",
    ],
    environment: "fixed:0",
}
```
```bash
inbreeding build
```
```bash
inbreeding run --options $OPTS --prng-seed $PRNG_SEED
```
```bash
let experiment = $"($PRNG_SEED)-($OPTS.environment)"
inbreeding load $experiment | inbreeding plot --options { k: $OPTS.k }
```
