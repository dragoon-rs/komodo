export def "regex or"    []: [ list<string> -> string ]  { str join '|' | $"\(($in)\)" }
export def "regex start" []: [      string  -> string ]  { $"^($in)"  }
export def "regex end"   []: [      string  -> string ]  { $"($in)$"  }
export def "regex exact" []: [      string  -> string ]  { $"^($in)$" }
