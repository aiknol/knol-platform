#!/usr/bin/env python3
"""
Seed the Knol demo with sample data.

This script populates the Knol platform with sample memories for demonstration
and testing purposes. It supports both direct API calls and mock mode for offline testing.

Usage:
    python seed_data.py [--api-key KEY] [--base-url URL] [--mock] [--verbose]

Examples:
    python seed_data.py                           # Use default settings
    python seed_data.py --mock                    # Run in mock mode (no API calls)
    python seed_data.py --api-key custom-key      # Use custom API key
    python seed_data.py --base-url http://prod    # Target production server
"""

import asyncio
import json
import sys
import argparse
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
from typing import Optional
from pathlib import Path

try:
    import httpx
except ImportError:
    print("Error: httpx is required. Install it with: pip install httpx")
    sys.exit(1)


@dataclass
class Memory:
    """Represents a memory to be stored in the system."""
    content: str
    role: str = "user"
    memory_type: str = "fact"
    entities: list = None
    metadata: dict = None

    def __post_init__(self):
        if self.entities is None:
            self.entities = []
        if self.metadata is None:
            self.metadata = {"type": self.memory_type}
        elif "type" not in self.metadata:
            self.metadata["type"] = self.memory_type

    def to_dict(self) -> dict:
        """Convert memory to API request format."""
        return {
            "content": self.content,
            "role": self.role,
            "metadata": self.metadata,
        }


class MemorySeeder:
    """Manages seeding demo data into Knol."""

    def __init__(
        self,
        base_url: str = "http://localhost:8080/v1",
        api_key: str = "demo-api-key-12345",
        mock_mode: bool = False,
        verbose: bool = False,
    ):
        """
        Initialize the memory seeder.

        Args:
            base_url: Base URL of the Knol API
            api_key: API key for authentication
            mock_mode: If True, don't make actual API calls
            verbose: If True, print detailed information
        """
        self.base_url = base_url
        self.api_key = api_key
        self.mock_mode = mock_mode
        self.verbose = verbose
        self.stored_count = 0
        self.failed_count = 0
        self.demo_memories = self._build_demo_memories()

    def _build_demo_memories(self) -> list:
        """Build the list of demo memories."""
        return [
            Memory(
                content="User prefers dark mode and minimal UI designs",
                memory_type="preference",
            ),
            Memory(
                content="John Smith is the CTO of Acme Corp, based in San Francisco",
                memory_type="fact",
            ),
            Memory(
                content="Had a meeting with Sarah about the Q4 roadmap on December 15th",
                memory_type="event",
            ),
            Memory(
                content="User switched from Python to Rust for backend services in March 2024",
                memory_type="temporal_change",
            ),
            Memory(
                content="Alice and Bob are co-founders of StartupXYZ and have worked together for 5 years",
                memory_type="relationship",
            ),
            Memory(
                content="User is allergic to shellfish and prefers vegetarian restaurants",
                memory_type="preference",
            ),
            Memory(
                content="The project deadline was moved from January 15 to February 28",
                memory_type="temporal_change",
            ),
            Memory(
                content="Discussed AI strategy with the board. They approved $2M budget for LLM integration",
                memory_type="event",
            ),
            Memory(
                content="User's team uses Slack for communication and Jira for project management",
                memory_type="fact",
            ),
            Memory(
                content="Maria from marketing suggested partnering with TechConf for the annual summit",
                memory_type="event",
            ),
        ]

    def log(self, message: str, level: str = "INFO"):
        """Log a message with timestamp and level."""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        prefix = f"[{timestamp}] [{level}]"

        level_colors = {
            "INFO": "\033[94m",     # Blue
            "SUCCESS": "\033[92m",  # Green
            "ERROR": "\033[91m",    # Red
            "WARN": "\033[93m",     # Yellow
        }
        reset = "\033[0m"

        if sys.stdout.isatty():
            color = level_colors.get(level, "")
            print(f"{color}{prefix} {message}{reset}")
        else:
            print(f"{prefix} {message}")

    async def seed_async(self) -> dict:
        """
        Seed demo data asynchronously.

        Returns:
            Dictionary with seeding statistics
        """
        self.log(f"{'🌱 Seeding Knol Demo Data':<50}")
        self.log(f"{'='*50}")
        self.log(f"Base URL: {self.base_url}")
        self.log(f"Mode: {'Mock (no API calls)' if self.mock_mode else 'Live API'}")
        self.log(f"Total memories to seed: {len(self.demo_memories)}")
        self.log(f"{'='*50}\n")

        if self.mock_mode:
            await self._seed_mock()
        else:
            await self._seed_api()

        return self._get_stats()

    async def _seed_mock(self):
        """Simulate seeding without making API calls."""
        self.log("Running in mock mode (simulating API calls)", "INFO")

        for i, memory in enumerate(self.demo_memories, 1):
            # Simulate processing delay
            await asyncio.sleep(0.2)

            preview = memory.content[:50]
            if len(memory.content) > 50:
                preview += "..."

            self.log(
                f"[{i:2d}/{len(self.demo_memories)}] [{memory.memory_type:15s}] {preview}",
                "SUCCESS"
            )
            self.stored_count += 1

    async def _seed_api(self):
        """Seed data using the actual API."""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        async with httpx.AsyncClient(timeout=30.0) as client:
            for i, memory in enumerate(self.demo_memories, 1):
                try:
                    url = f"{self.base_url}/memory"
                    payload = memory.to_dict()

                    if self.verbose:
                        self.log(f"POST {url}", "INFO")
                        self.log(f"Payload: {json.dumps(payload, indent=2)}", "INFO")

                    response = await client.post(
                        url,
                        json=payload,
                        headers=headers,
                    )

                    preview = memory.content[:50]
                    if len(memory.content) > 50:
                        preview += "..."

                    if response.status_code in (200, 201):
                        self.stored_count += 1
                        data = response.json()
                        memory_id = data.get("id", "unknown")[:8]
                        self.log(
                            f"[{i:2d}/{len(self.demo_memories)}] [{memory.memory_type:15s}] {preview} (ID: {memory_id}...)",
                            "SUCCESS"
                        )
                    else:
                        self.failed_count += 1
                        self.log(
                            f"[{i:2d}/{len(self.demo_memories)}] [{memory.memory_type:15s}] FAILED ({response.status_code}): {preview}",
                            "ERROR"
                        )
                        if self.verbose:
                            self.log(f"Response: {response.text}", "ERROR")

                except httpx.ConnectError as e:
                    self.failed_count += 1
                    self.log(
                        f"Connection error: {e}. Is the API server running?",
                        "ERROR"
                    )
                    self.log(
                        f"Consider running with --mock flag for offline testing",
                        "WARN"
                    )
                    raise SystemExit(1)
                except httpx.TimeoutException as e:
                    self.failed_count += 1
                    self.log(f"Request timeout: {e}", "ERROR")
                    raise SystemExit(1)
                except Exception as e:
                    self.failed_count += 1
                    preview = memory.content[:50]
                    if len(memory.content) > 50:
                        preview += "..."
                    self.log(
                        f"[{i:2d}/{len(self.demo_memories)}] Error storing '{preview}': {str(e)}",
                        "ERROR"
                    )

    def _get_stats(self) -> dict:
        """Get seeding statistics."""
        return {
            "total": len(self.demo_memories),
            "stored": self.stored_count,
            "failed": self.failed_count,
            "success_rate": (self.stored_count / len(self.demo_memories) * 100) if self.demo_memories else 0,
        }

    def print_summary(self):
        """Print seeding summary."""
        stats = self._get_stats()
        total = stats["total"]
        stored = stats["stored"]
        failed = stats["failed"]
        success_rate = stats["success_rate"]

        self.log(f"\n{'='*50}")
        self.log(f"📊 Seeding Summary")
        self.log(f"{'='*50}")
        self.log(f"Total memories:     {total}")
        self.log(f"Successfully stored: {stored}")
        self.log(f"Failed:             {failed}")
        self.log(f"Success rate:       {success_rate:.1f}%")
        self.log(f"{'='*50}\n")

        if failed == 0:
            self.log("✅ All memories seeded successfully!", "SUCCESS")
        elif stored > 0:
            self.log(f"⚠️  Partially succeeded ({stored}/{total} memories stored)", "WARN")
        else:
            self.log("❌ Failed to seed memories", "ERROR")


async def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Seed Knol demo with sample data",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Default mode (live API)
  python seed_data.py

  # Mock mode (offline testing)
  python seed_data.py --mock

  # Custom API key and base URL
  python seed_data.py --api-key my-key --base-url https://api.example.com/v1

  # Verbose output
  python seed_data.py --verbose
        """,
    )

    parser.add_argument(
        "--api-key",
        default="demo-api-key-12345",
        help="API key for authentication (default: demo-api-key-12345)",
    )
    parser.add_argument(
        "--base-url",
        default="http://localhost:8080/v1",
        help="Base URL of Knol API (default: http://localhost:8080/v1)",
    )
    parser.add_argument(
        "--mock",
        action="store_true",
        help="Run in mock mode without making API calls",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Print detailed information about API requests and responses",
    )

    args = parser.parse_args()

    seeder = MemorySeeder(
        base_url=args.base_url,
        api_key=args.api_key,
        mock_mode=args.mock,
        verbose=args.verbose,
    )

    try:
        await seeder.seed_async()
        seeder.print_summary()
    except KeyboardInterrupt:
        seeder.log("\nSeeding interrupted by user", "WARN")
        sys.exit(1)
    except SystemExit:
        raise
    except Exception as e:
        seeder.log(f"Fatal error: {str(e)}", "ERROR")
        if args.verbose:
            import traceback
            traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
