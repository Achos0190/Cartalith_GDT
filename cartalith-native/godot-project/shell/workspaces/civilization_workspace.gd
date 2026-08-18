extends Workspace
class_name CivilizationWorkspace

## CIVIL domain (§3): settlements, population, economy, politics, culture.
##
## The engine backs all five as *readable* data (`get_settlements`,
## `get_provinces`, `get_trade_balances`). It also backs placing a
## settlement and painting territory, which this dock has no surface for --
## `STRANDED_TOOLS.md` rows 10 and 12.

func _build() -> void:
	_not_built("Civilization",
		"Settlement, province and trade tables are live in Data ▸ World data tables; this dock is being built.")
