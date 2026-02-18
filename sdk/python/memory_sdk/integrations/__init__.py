"""Integrations for the Knol Memory SDK.

This package provides integrations with popular AI frameworks:
- LangChain: Memory backends for LangChain applications
- CrewAI: Memory and storage backends for CrewAI agents
"""

from __future__ import annotations

__all__ = [
    "KnolMemory",
    "KnolChatMessageHistory",
    "KnolRetriever",
    "KnolCrewMemory",
    "KnolCrewStorage",
]

# Lazy imports to avoid requiring optional dependencies
def __getattr__(name: str):
    """Lazy load integrations on demand."""
    if name == "KnolMemory":
        from memory_sdk.integrations.langchain import KnolMemory
        return KnolMemory
    elif name == "KnolChatMessageHistory":
        from memory_sdk.integrations.langchain import KnolChatMessageHistory
        return KnolChatMessageHistory
    elif name == "KnolRetriever":
        from memory_sdk.integrations.langchain import KnolRetriever
        return KnolRetriever
    elif name == "KnolCrewMemory":
        from memory_sdk.integrations.crewai import KnolCrewMemory
        return KnolCrewMemory
    elif name == "KnolCrewStorage":
        from memory_sdk.integrations.crewai import KnolCrewStorage
        return KnolCrewStorage
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
