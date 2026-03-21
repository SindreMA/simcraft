"""Item, enchant, and gem info endpoints.

Uses local Raidbots game data files for instant lookups.
"""

from typing import Any

from fastapi import APIRouter, HTTPException

from schemas import ItemInfoRequest
from services import game_data

router = APIRouter(tags=["items"])


def _normalize_bonus(bonus_ids: list[int] | None) -> str:
    if not bonus_ids:
        return ""
    return ":".join(str(b) for b in sorted(bonus_ids))


def _fallback(item_id: int) -> dict[str, Any]:
    return {
        "item_id": item_id,
        "name": f"Item {item_id}",
        "quality": 1,
        "quality_name": "common",
        "icon": "inv_misc_questionmark",
        "ilevel": 0,
    }


@router.get("/api/item-info/{item_id}")
async def get_item_info(
    item_id: int,
    bonus_ids: str = "",
):
    bonus_list = [int(b) for b in bonus_ids.split(",") if b.strip()] if bonus_ids else []
    return game_data.get_item_info(item_id, bonus_list or None) or _fallback(item_id)


@router.post("/api/item-info/batch")
async def get_item_info_batch(req: ItemInfoRequest):
    """Fetch info for multiple items at once."""
    items_list = req.items
    if not items_list and req.item_ids:
        items_list = [{"item_id": iid} for iid in req.item_ids]

    if not items_list or len(items_list) > 100:
        raise HTTPException(status_code=400, detail="Provide 1-100 items")

    seen: set[str] = set()
    unique_items: list[dict] = []
    for item in items_list:
        iid = item.get("item_id", 0)
        bonus = item.get("bonus_ids") or []
        key = f"{iid}:{_normalize_bonus(bonus)}"
        if key not in seen:
            seen.add(key)
            unique_items.append({"item_id": iid, "bonus_ids": bonus})

    results: dict[str, dict[str, Any]] = {}
    for item in unique_items:
        iid = item["item_id"]
        bonus = item["bonus_ids"]
        results[str(iid)] = game_data.get_item_info(iid, bonus or None) or _fallback(iid)

    return results


@router.get("/api/enchant-info/{enchant_id}")
async def get_enchant_info(enchant_id: int):
    """Look up enchant name from local game data."""
    return game_data.get_enchant_info(enchant_id) or {"enchant_id": enchant_id, "name": ""}


@router.get("/api/gem-info/{gem_id}")
async def get_gem_info(gem_id: int):
    """Look up gem info by item ID from local game data."""
    return game_data.get_gem_info(gem_id) or {"gem_id": gem_id, "name": "", "icon": "", "quality": 3}


@router.post("/api/max-upgrade-ilevels")
async def get_max_upgrade_ilevels(items: list[dict]):
    """For each item, return the max upgrade ilevel given its bonus_ids."""
    results: dict[str, int] = {}
    for item in items[:200]:
        item_id = item.get("item_id", 0)
        bonus_ids = item.get("bonus_ids", [])
        upgraded = game_data.upgrade_bonus_ids_to_max(bonus_ids)
        info = game_data.get_item_info(item_id, upgraded)
        if info:
            key = f"{item_id}:{','.join(str(b) for b in sorted(bonus_ids))}"
            results[key] = info["ilevel"]
    return results


@router.get("/api/upgrade-options")
async def get_upgrade_options(bonus_ids: str):
    """Get all upgrade levels for an item given its bonus IDs."""
    ids = [int(b) for b in bonus_ids.split(",") if b.strip()]
    options = game_data.get_upgrade_options(ids)
    if not options:
        return {"options": []}
    return {"options": options}
