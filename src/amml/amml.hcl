namespace   = "alaydriem"
name        = "durability-ping"
entry       = "alaydriem_durability_ping.dll"
api_version = 3

uses {
  feature "alert" {
    event "client.tick" {
      version  = 1
      required = true
    }
    event "world.left" {
      version  = 1
      required = true
    }
    event "registry.provider_ready" {
      version  = 1
      required = true
    }
    event "registry.provider_gone" {
      version  = 1
      required = true
    }
    area "client.read" {
      version  = 1
      required = true
    }
    area "inventory.read" {
      version  = 1
      required = true
    }
    event "mods.loaded" {
      version  = 1
      required = true
    }
    event "input.lease_suspended" {
      version  = 1
      required = true
    }
    event "input.lease_resumed" {
      version  = 1
      required = true
    }
    event "input.lease_lost" {
      version  = 1
      required = true
    }
    event "input.injected_released" {
      version  = 2
      required = true
    }
    event "overlay.screenshot_taken" {
      version  = 1
      required = true
    }
    event "overlay.screenshot_written" {
      version  = 1
      required = true
    }
    area "input.lease" {
      version  = 1
      required = true
    }
  }
  feature "guard" {
    event "player.block_break_start" {
      version  = 1
      required = true
    }
    event "player.block_break_continue" {
      version  = 1
      required = true
    }
    event "player.item_use" {
      version  = 1
      required = true
    }
    event "player.attack" {
      version  = 1
      required = true
    }
    event "player.item_use_on_block" {
      version  = 1
      required = true
    }
  }
}

listing {
  title       = "Durability Ping"
  summary     = "A tone when your tool is low, and an optional guard before it breaks."
  description = <<-EOT
    Plays a tone when the tool in your hand is low on durability, and can guard it before it breaks.
  EOT
  authors     = ["Charles R. Portwood II"]
  license     = "BSD-3-Clause"
  source      = "https://github.com/alaydriem/amml-durability-ping"
  homepage    = "https://github.com/alaydriem/amml-durability-ping#readme"
  tags        = ["durability", "tools", "client", "utility"]
  icon        = "icon.png"
}
