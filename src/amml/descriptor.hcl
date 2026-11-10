namespace    = "alaydriem"
name         = "durability-ping"
display_name = "Durability Ping"
version      = "26.63.4"
author       = "Alaydriem"
config_path  = "config.hcl"

claim "Input" {
  mode   = "Shared"
  reason = "Reads the override key that lets a protected tool be used"
}

keybind "override" {
  label       = "Use a protected tool"
  default     = "Alt"
  capability  = "Input"
  scope       = "Global"
  config_path = "keys.override"
}

section "alert" {
  title = "Alert"

  field "alert.enabled" {
    label   = "Play the alert"
    help    = "Plays a tone when the held tool is low"
    kind    = "bool"
    default = true
    apply   = "Live"
  }

  field "alert.percent" {
    label   = "Alert at percent left"
    help    = "Alert when this percent of durability is left. 0 turns this part off"
    kind    = { int = { min = 0, max = 100, step = 1 } }
    default = 10
    apply   = "Live"
  }

  field "alert.uses" {
    label   = "Alert at uses left"
    help    = "Alert when this many uses are left. 0 turns this part off"
    kind    = { int = { min = 0, max = 2000, step = 1 } }
    default = 20
    apply   = "Live"
  }

  field "alert.volume" {
    label   = "Alert volume"
    help    = "How loud the tone is"
    kind    = { int = { min = 0, max = 100, step = 5 } }
    default = 60
    apply   = "Live"
  }
}

section "block" {
  title = "Block"

  field "block.enabled" {
    label   = "Protect low tools"
    help    = "Stops a tool from being used when it is almost broken"
    kind    = "bool"
    default = false
    apply   = "Live"
  }

  field "block.percent" {
    label   = "Block at percent left"
    help    = "Block when this percent of durability is left. 0 turns this part off"
    kind    = { int = { min = 0, max = 100, step = 1 } }
    default = 2
    apply   = "Live"
  }

  field "block.uses" {
    label   = "Block at uses left"
    help    = "Block when this many uses are left. 0 turns this part off"
    kind    = { int = { min = 0, max = 2000, step = 1 } }
    default = 3
    apply   = "Live"
  }
}

section "keys" {
  title = "Keys"

  field "keys.override" {
    label   = "Use a protected tool"
    help    = "Hold this key to use a tool that is blocked"
    kind    = "chord"
    default = "Alt"
    apply   = "Live"
  }
}
