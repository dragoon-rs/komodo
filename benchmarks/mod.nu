export use run.nu
export use plot.nu

export-env {
    ^$nu.current-exe ./scripts/check-nushell-version.nu
}
