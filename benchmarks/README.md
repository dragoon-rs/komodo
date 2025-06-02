# Table of contents
- [Requirements](#requirements)
- [Run the benchmarks](#run-the-benchmarks)
    - [define them](#define-them)
    - [run them](#run-them)
- [Plot the benchmarks](#plot-the-benchmarks)

## requirements
> :bulb: **Note**
>
> these should only be required for plotting results

- install [GPLT](https://gitlab.isae-supaero.fr/a.stevan/gplt)
- create a virtual environment
```bash
const VENV = "~/.local/share/venvs/gplt/bin/activate.nu" | path expand
```
```bash
virtualenv ($VENV | path dirname --num-levels 2)
```
- activate the virtual environment
```bash
overlay use $VENV
```

## Run the benchmarks
### define them

> :bulb: **Note**
>
> the FRI benchmarks don't use a module from [src/bin/](src/bin/) with PLNK but rather an
> [example](../examples/fri.rs)

```bash
const RESULTS_DIR = "/path/to/komodo-benchmark-results/"
```

### run them
```bash
nu benchmarks/run.nu $RESULTS_DIR
```

> the following `watch` can be used to see the results as they are dumped to `$RESULTS_DIR`
> ```bash
> watch $RESULTS_DIR { |op, path|
>     $"($op)  ($path)"
> }
> ```
