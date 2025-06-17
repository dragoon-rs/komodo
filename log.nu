export def "str color" [color: string]: [ string -> string ] {
    $"(ansi $color)($in)(ansi reset)"
}

def log [level: string, color: string, msg: string] {
    print $"[($level | str color $color)] ($msg)"
}
export def "log fatal"   [msg: string] { log " FAT" "red_bold"       $msg }
export def "log error"   [msg: string] { log " ERR" "red"            $msg }
export def "log warning" [msg: string] { log "WARN" "yellow"         $msg }
export def "log info"    [msg: string] { log "INFO" "cyan"           $msg }
export def "log debug"   [msg: string] { log " DBG" "default_dimmed" $msg }
export def "log ok"      [msg: string] { log "  OK" "green"          $msg }
export def "log hint"    [msg: string] { log "HINT" "purple"         $msg }
