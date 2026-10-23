"""Built-in recipe definitions and recipe engine."""

from typing import Any

BUILTIN_RECIPES: list[dict[str, Any]] = [
    {
        "id": "tiktok_montage",
        "name": "TikTok Montage",
        "description": "Auto-zoom on kills, beat-synced cuts, Hormozi captions, 9:16",
        "is_builtin": True,
        "output_format": "9:16",
        "caption_style": "hormozi",
        "transition_style": "cut",
        "effects_per_moment": {
            "kill": [{"type": "zoom", "scale": 1.3, "duration_ms": 800}],
            "multi_kill": [
                {"type": "zoom", "scale": 1.5, "duration_ms": 1000},
                {"type": "shake", "intensity": 0.3, "duration_ms": 500},
                {"type": "impact_text", "text": "ACE", "style": "bold"},
            ],
            "death": [{"type": "speed", "factor": 0.5}],
            "reaction": [{"type": "zoom", "scale": 1.2, "duration_ms": 600}],
        },
        "max_duration_s": 60,
        "beat_sync": True,
    },
    {
        "id": "youtube_highlight",
        "name": "YouTube Highlight",
        "description": "Wider cuts, scene context preserved, standard captions, 16:9",
        "is_builtin": True,
        "output_format": "16:9",
        "caption_style": "bold",
        "transition_style": "fade",
        "effects_per_moment": {
            "kill": [{"type": "zoom", "scale": 1.15, "duration_ms": 600}],
            "multi_kill": [{"type": "zoom", "scale": 1.2, "duration_ms": 800}],
        },
        "max_duration_s": 180,
        "beat_sync": False,
    },
    {
        "id": "discord_share",
        "name": "Discord Share",
        "description": "Minimal edits, full gameplay visible, no captions",
        "is_builtin": True,
        "output_format": "16:9",
        "caption_style": None,
        "transition_style": "cut",
        "effects_per_moment": {},
        "max_duration_s": 120,
        "beat_sync": False,
    },
    {
        "id": "reaction_clip",
        "name": "Reaction Clip",
        "description": "Facecam tracked, zoom on reactions, audio peaks emphasized",
        "is_builtin": True,
        "output_format": "9:16",
        "caption_style": "karaoke",
        "transition_style": "cut",
        "effects_per_moment": {
            "reaction": [
                {"type": "zoom", "scale": 1.4, "duration_ms": 1000},
                {"type": "speed", "factor": 0.8},
            ],
            "audio_peak": [{"type": "zoom", "scale": 1.2, "duration_ms": 500}],
        },
        "max_duration_s": 60,
        "beat_sync": False,
    },
    {
        "id": "skill_showcase",
        "name": "Skill Showcase",
        "description": "Slow-mo on best moment, clean captions, no memes",
        "is_builtin": True,
        "output_format": "9:16",
        "caption_style": "bar",
        "transition_style": "fade",
        "effects_per_moment": {
            "kill": [{"type": "speed", "factor": 0.5}],
            "multi_kill": [
                {"type": "speed", "factor": 0.3},
                {"type": "zoom", "scale": 1.2, "duration_ms": 1500},
            ],
        },
        "max_duration_s": 45,
        "beat_sync": False,
    },
]


def apply_recipe_to_moments(recipe: dict, key_moments: list[dict]) -> list[dict]:
    """Given a recipe and key moments, produce the list of effects to apply."""
    effects_map = recipe.get("effects_per_moment", {})
    applied: list[dict] = []

    for moment in key_moments:
        moment_type = moment.get("type", "event")
        moment_effects = effects_map.get(moment_type, [])

        for effect_template in moment_effects:
            effect = dict(effect_template)
            effect["timestamp_ms"] = moment.get("timestamp_ms", 0)
            effect["moment_label"] = moment.get("label", "")
            applied.append(effect)

    return applied


def get_recipe_by_id(recipe_id: str) -> dict | None:
    """Look up a built-in recipe by ID."""
    for r in BUILTIN_RECIPES:
        if r["id"] == recipe_id:
            return r
    return None
