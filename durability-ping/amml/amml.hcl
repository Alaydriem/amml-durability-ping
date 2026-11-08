namespace   = "amml"
name        = "durability-ping"
entry       = "amml_durability_ping.dll"
api_version = 2
amml_version = "26.62.0"
entries = [
  "ClientInstance.update",
  "ClientInstance.getLocalPlayer",
  "Player.playerInventory",
  "FillingContainer.mItems",
  "ItemStackBase.getDamageValue",
  "Item.getMaxDamage",
  "Item.mFullName",
  "ClientInstance.isShowingMenu",
  "GameMode.vtable",
  "GameMode.mPlayer",
  "GameMode.startDestroyBlock",
  "GameMode.continueDestroyBlock",
  "GameMode.useItem",
  "GameMode.attack",
  "SurvivalMode.vtable",
  "Item.useOn",
]

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
