use consts.nu

export def remove-cache-prefix []: path -> string {
    str replace $"($consts.CACHE)(char path_sep)" ''
}
