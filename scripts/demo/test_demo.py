#!/usr/bin/env python3
"""
Knol Demo — End-to-end Verification Tool

Tests the full demo pipeline from the terminal:
  1. LLM API connectivity  (Gemini / OpenAI)
  2. Memory extraction quality
  3. Admin demo config endpoint (optional)
  4. Multi-turn conversation with memory recall

Usage:
    python test_demo.py                                          # Auto-detect from admin API
    python test_demo.py --provider gemini --api-key AIza...      # Explicit provider + key
    python test_demo.py --admin-url http://localhost:8084        # Fetch config from admin
    python test_demo.py --provider openai --api-key sk-...       # Test with OpenAI

Environment variables:
    GEMINI_API_KEY, OPENAI_API_KEY, ANTHROPIC_API_KEY
    DEMO_ADMIN_API_URL (default: http://localhost:8084)
"""

import asyncio
import json
import os
import sys
import time
import argparse
from typing import Optional

try:
    import httpx
except ImportError:
    print("Error: httpx is required. Install with: pip install httpx")
    sys.exit(1)


# ── Styling ──────────────────────────────────────────────────────

class C:
    """Terminal colors."""
    BOLD = "\033[1m"
    DIM = "\033[2m"
    GREEN = "\033[32m"
    RED = "\033[31m"
    YELLOW = "\033[33m"
    BLUE = "\033[34m"
    VIOLET = "\033[35m"
    CYAN = "\033[36m"
    RESET = "\033[0m"

def ok(msg: str):
    print(f"  {C.GREEN}✓{C.RESET} {msg}")

def fail(msg: str):
    print(f"  {C.RED}✗{C.RESET} {msg}")

def info(msg: str):
    print(f"  {C.BLUE}ℹ{C.RESET} {msg}")

def warn(msg: str):
    print(f"  {C.YELLOW}⚠{C.RESET} {msg}")

def header(msg: str):
    print(f"\n{C.BOLD}{C.VIOLET}{'─' * 60}{C.RESET}")
    print(f"{C.BOLD}{C.VIOLET}  {msg}{C.RESET}")
    print(f"{C.BOLD}{C.VIOLET}{'─' * 60}{C.RESET}")

def sub(msg: str):
    print(f"\n{C.BOLD}  {msg}{C.RESET}")


# ── LLM Clients ──────────────────────────────────────────────────

SYSTEM_PROMPT = """You are the AI inside "Knol", a persistent memory system for AI applications.

CURRENT MEMORY STORE:
{memories}

Respond in this exact JSON format:
{{
  "response": "Your conversational response. Reference stored memories when relevant.",
  "new_memories": [
    {{
      "content": "A concise extracted memory",
      "type": "fact|preference|event|relationship|temporal_change|goal",
      "entities": [
        {{"name": "EntityName", "type": "person|organization|technology|concept|location"}}
      ]
    }}
  ],
  "referenced_memory_indices": [0, 2]
}}

RULES:
- Extract ALL meaningful facts, preferences, relationships from the user's message
- Entity names should be capitalized properly
- Reference stored memories by index when relevant
- Response must be valid JSON only"""


async def call_gemini(client: httpx.AsyncClient, api_key: str, model: str,
                      user_msg: str, memory_ctx: str) -> dict:
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    prompt = SYSTEM_PROMPT.format(memories=memory_ctx or "(empty)")

    resp = await client.post(url, json={
        "systemInstruction": {"parts": [{"text": prompt}]},
        "contents": [{"role": "user", "parts": [{"text": user_msg}]}],
        "generationConfig": {
            "temperature": 0.7,
            "maxOutputTokens": 1024,
            "responseMimeType": "application/json"
        }
    }, timeout=30)
    resp.raise_for_status()
    data = resp.json()
    text = data["candidates"][0]["content"]["parts"][0]["text"]
    return json.loads(text)


async def call_openai(client: httpx.AsyncClient, api_key: str, model: str,
                      user_msg: str, memory_ctx: str) -> dict:
    url = "https://api.openai.com/v1/chat/completions"
    prompt = SYSTEM_PROMPT.format(memories=memory_ctx or "(empty)")

    resp = await client.post(url, json={
        "model": model,
        "messages": [
            {"role": "system", "content": prompt},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.7,
        "max_tokens": 1024,
        "response_format": {"type": "json_object"}
    }, headers={"Authorization": f"Bearer {api_key}"}, timeout=30)
    resp.raise_for_status()
    data = resp.json()
    text = data["choices"][0]["message"]["content"]
    return json.loads(text)


# ── Test Suite ───────────────────────────────────────────────────

class DemoTester:
    def __init__(self, provider: str, api_key: str, model: str,
                 admin_url: Optional[str] = None, use_admin_proxy: bool = False):
        self.provider = provider
        self.api_key = api_key
        self.model = model
        self.admin_url = admin_url
        self.use_admin_proxy = use_admin_proxy
        self.memories: list[dict] = []
        self.passed = 0
        self.failed = 0
        self.rate_limited = False

    def memory_context(self) -> str:
        return "\n".join(
            f"[{i}] ({m['type']}) {m['content']}"
            for i, m in enumerate(self.memories)
        )

    async def call_llm(self, client: httpx.AsyncClient, msg: str) -> dict:
        ctx = self.memory_context()
        last_err = None
        for attempt in range(3):
            try:
                if self.use_admin_proxy:
                    if not self.admin_url:
                        raise ValueError("Admin proxy mode enabled but admin_url is missing")
                    resp = await client.post(
                        f"{self.admin_url}/admin/demo/extract",
                        json={"user_message": msg, "memory_context": ctx},
                        timeout=30,
                    )
                    resp.raise_for_status()
                    return resp.json()

                if self.provider == "gemini":
                    return await call_gemini(client, self.api_key, self.model, msg, ctx)
                elif self.provider == "openai":
                    return await call_openai(client, self.api_key, self.model, msg, ctx)
                else:
                    raise ValueError(f"Unsupported provider for browser demo: {self.provider}")
            except httpx.HTTPStatusError as e:
                if e.response.status_code == 429 and attempt < 2:
                    wait = (attempt + 1) * 5
                    warn(f"Rate limited (429) — retrying in {wait}s (attempt {attempt + 2}/3)")
                    await asyncio.sleep(wait)
                    last_err = e
                    continue
                raise
        raise last_err  # type: ignore

    def store_memories(self, new_memories: list[dict]):
        for m in new_memories:
            self.memories.append(m)

    def check(self, condition: bool, pass_msg: str, fail_msg: str):
        if condition:
            ok(pass_msg)
            self.passed += 1
        else:
            fail(fail_msg)
            self.failed += 1

    async def run(self):
        header("Knol Demo — End-to-End Verification")
        info(f"Provider: {self.provider}")
        info(f"Model: {self.model}")
        if self.use_admin_proxy:
            info("LLM auth: server-side credential via admin proxy")
        else:
            info(f"API key: {self.api_key[:8]}...{self.api_key[-4:]}" if len(self.api_key) > 12 else f"API key: {self.api_key[:8]}...")

        async with httpx.AsyncClient() as client:
            # ── Test 1: Admin config endpoint ──
            if self.admin_url:
                await self.test_admin_config(client)

            # ── Test 2: LLM API connectivity ──
            await self.test_api_connectivity(client)

            if not self.rate_limited:
                # ── Test 3: Memory extraction quality ──
                await self.test_memory_extraction(client)

                # ── Test 4: Multi-turn memory recall ──
                await self.test_memory_recall(client)

                # ── Test 5: Entity extraction ──
                await self.test_entity_extraction(client)
            else:
                info("Skipping tests 3-5 (rate limited)")
                info("The API key is valid — try again later or upgrade your plan")

        # ── Results ──
        header("Results")
        total = self.passed + self.failed
        if self.failed == 0:
            print(f"\n  {C.GREEN}{C.BOLD}All {total} tests passed!{C.RESET} 🎉\n")
        else:
            print(f"\n  {C.BOLD}{self.passed}/{total} passed{C.RESET}, {C.RED}{self.failed} failed{C.RESET}\n")

        return self.failed == 0

    async def test_admin_config(self, client: httpx.AsyncClient):
        sub("1. Admin Demo Config Endpoint")
        try:
            resp = await client.get(f"{self.admin_url}/admin/demo/config", timeout=5)
            self.check(resp.status_code == 200, "Endpoint reachable (200 OK)", f"HTTP {resp.status_code}")

            if resp.status_code == 200:
                data = resp.json()
                self.check("llm_provider" in data, f"Provider returned: {data.get('llm_provider', '?')}", "Missing llm_provider field")
                self.check("llm_ready" in data, "llm_ready field present", "Missing llm_ready field")
                self.check(data.get("enabled") is True, "Demo is enabled", "Demo is disabled!")

                llm_ready = bool(data.get("llm_ready"))
                if llm_ready:
                    ok("LLM credential is configured server-side")
                else:
                    warn("LLM credential is not configured — demo will use fallback mode")

        except httpx.ConnectError:
            warn(f"Admin API not running at {self.admin_url} (skipping)")
        except Exception as e:
            warn(f"Admin API error: {e}")

    async def test_api_connectivity(self, client: httpx.AsyncClient):
        sub(f"2. LLM API Connectivity ({self.provider})")
        try:
            t0 = time.time()
            result = await self.call_llm(client, "Hello, just testing connectivity. Reply briefly.")
            latency = (time.time() - t0) * 1000

            self.check(True, f"API responded in {latency:.0f}ms", "")
            self.check("response" in result, "Response field present", "Missing 'response' field in JSON")
            self.check(isinstance(result.get("new_memories"), list), "new_memories is a list", "Invalid new_memories format")

            info(f"AI said: {result['response'][:100]}...")
        except httpx.HTTPStatusError as e:
            body = e.response.text[:200]
            if e.response.status_code == 429:
                self.check(True, "API key is valid (authenticated successfully)", "")
                warn("Rate limited (429) — quota exceeded. Remaining tests will be skipped.")
                self.rate_limited = True
            elif "API_KEY_INVALID" in body or "expired" in body:
                self.check(False, "", f"API key is invalid or expired — get a new key")
            else:
                self.check(False, "", f"HTTP {e.response.status_code}: {body}")
        except Exception as e:
            self.check(False, "", f"Connection failed: {e}")

    async def test_memory_extraction(self, client: httpx.AsyncClient):
        sub("3. Memory Extraction Quality")

        msg = "I'm Alex, a senior engineer at Meridian Health. I lead the platform team and prefer Rust over Go."
        try:
            result = await self.call_llm(client, msg)
            memories = result.get("new_memories", [])

            self.check(len(memories) >= 2, f"Extracted {len(memories)} memories (expected ≥2)", f"Only {len(memories)} memories extracted")

            # Check for entity extraction
            all_entities = [e for m in memories for e in m.get("entities", [])]
            entity_names = [e.get("name", "").lower() for e in all_entities]

            self.check(any("alex" in n for n in entity_names), "Found entity: Alex", "Missing entity: Alex")
            self.check(any("meridian" in n for n in entity_names), "Found entity: Meridian Health", "Missing entity: Meridian Health")

            # Check memory types
            types = [m.get("type") for m in memories]
            self.check("preference" in types or any("rust" in m.get("content", "").lower() for m in memories),
                        "Preference memory detected (Rust over Go)", "Preference not extracted")

            # Store for next test
            self.store_memories(memories)
            info(f"Stored {len(memories)} memories for recall test")

            for m in memories:
                entities_str = ", ".join(e.get("name", "?") for e in m.get("entities", []))
                print(f"    {C.DIM}[{m.get('type', '?')}]{C.RESET} {m.get('content', '?')[:80]} {C.DIM}({entities_str}){C.RESET}")

        except Exception as e:
            self.check(False, "", f"Extraction failed: {e}")

    async def test_memory_recall(self, client: httpx.AsyncClient):
        sub("4. Multi-turn Memory Recall")

        # Add more context
        msg2 = "My CTO Priya Sharma wants us to migrate from MongoDB to PostgreSQL by Q3."
        try:
            result2 = await self.call_llm(client, msg2)
            self.store_memories(result2.get("new_memories", []))
            info(f"Added {len(result2.get('new_memories', []))} more memories")

            # Now ask a recall question
            recall_msg = "What do you know about my situation? Summarize everything."
            result3 = await self.call_llm(client, recall_msg)
            response_text = result3.get("response", "").lower()

            self.check("alex" in response_text, "Recalled: Alex (name)", "Missing recall: Alex")
            self.check("meridian" in response_text, "Recalled: Meridian Health", "Missing recall: Meridian Health")
            self.check("priya" in response_text or "cto" in response_text, "Recalled: Priya/CTO", "Missing recall: Priya/CTO")
            self.check("mongo" in response_text or "postgres" in response_text or "migration" in response_text,
                        "Recalled: Migration context", "Missing recall: Migration")

            refs = result3.get("referenced_memory_indices", [])
            self.check(len(refs) >= 2, f"Referenced {len(refs)} stored memories", "Too few memory references")

            info(f"AI recall: {result3['response'][:150]}...")

        except Exception as e:
            self.check(False, "", f"Recall test failed: {e}")

    async def test_entity_extraction(self, client: httpx.AsyncClient):
        sub("5. Entity & Relationship Extraction")

        msg = "David Park, our compliance officer, said we need HIPAA encryption before the migration goes live."
        try:
            result = await self.call_llm(client, msg)
            memories = result.get("new_memories", [])
            all_entities = [e for m in memories for e in m.get("entities", [])]
            entity_names = [e.get("name", "").lower() for e in all_entities]
            entity_types = [e.get("type", "").lower() for e in all_entities]

            self.check(any("david" in n for n in entity_names), "Extracted entity: David Park", "Missing: David Park")
            self.check(any("person" in t for t in entity_types), "Identified person entity type", "Missing person type")
            self.check(any("hipaa" in n or "hipaa" in m.get("content", "").lower() for n in entity_names for m in memories),
                        "Captured HIPAA requirement", "Missing HIPAA context")

            # Check for relationship type
            types = [m.get("type") for m in memories]
            has_rel = "relationship" in types or any("compliance" in m.get("content", "").lower() for m in memories)
            self.check(has_rel, "Relationship/role captured", "Missing relationship extraction")

            self.store_memories(memories)
            info(f"Total memories after all tests: {len(self.memories)}")
            info(f"Total entities: {len(set(e.get('name', '') for m in self.memories for e in m.get('entities', [])))}")

        except Exception as e:
            self.check(False, "", f"Entity extraction failed: {e}")


# ── CLI ──────────────────────────────────────────────────────────

async def main():
    parser = argparse.ArgumentParser(
        description="Knol Demo — End-to-End Verification Tool",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python test_demo.py --provider gemini --api-key AIzaSy...
  python test_demo.py --provider openai --api-key sk-...
  python test_demo.py --admin-url http://localhost:8084
  GEMINI_API_KEY=AIza... python test_demo.py
        """
    )
    parser.add_argument("--provider", choices=["gemini", "openai"], default=None,
                        help="LLM provider (default: auto-detect)")
    parser.add_argument("--api-key", default=None, help="LLM API key")
    parser.add_argument("--model", default=None, help="Model override")
    parser.add_argument("--admin-url", default=None,
                        help="Admin API URL to fetch config from (default: $DEMO_ADMIN_API_URL or http://localhost:8084)")

    args = parser.parse_args()

    provider = args.provider
    api_key = args.api_key
    model = args.model
    admin_url = args.admin_url or os.environ.get("DEMO_ADMIN_API_URL", "http://localhost:8084")
    use_admin_proxy = False

    # Try to load config from admin API first.
    try:
        async with httpx.AsyncClient() as client:
            resp = await client.get(f"{admin_url}/admin/demo/config", timeout=3)
            if resp.status_code == 200:
                data = resp.json()
                if not provider:
                    provider = data.get("llm_provider", "gemini")
                if not model and data.get("llm_model"):
                    model = data["llm_model"]
                if not api_key and data.get("llm_ready") is True:
                    use_admin_proxy = True
                info(f"Loaded config from admin API at {admin_url}")
    except Exception:
        pass  # Admin not running, fall back to direct key mode

    # Fall back to environment variables
    if not use_admin_proxy and not api_key:
        if provider == "openai":
            api_key = os.environ.get("OPENAI_API_KEY", "")
        else:
            api_key = os.environ.get("GEMINI_API_KEY", "")
            if not api_key:
                api_key = os.environ.get("OPENAI_API_KEY", "")
                if api_key:
                    provider = "openai"

    if not provider:
        provider = "gemini"

    # Set default models
    if not model:
        model = "gemini-2.0-flash" if provider == "gemini" else "gpt-4o-mini"

    if not use_admin_proxy and not api_key:
        print(f"\n{C.RED}{C.BOLD}Error: No API key found.{C.RESET}\n")
        print("Provide an API key via:")
        print(f"  {C.DIM}--api-key AIzaSy...{C.RESET}")
        print(f"  {C.DIM}GEMINI_API_KEY=... python test_demo.py{C.RESET}")
        print(f"  {C.DIM}Or store it in admin panel and ensure /admin/demo/config returns llm_ready=true{C.RESET}")
        sys.exit(1)

    tester = DemoTester(
        provider=provider,
        api_key=api_key or "",
        model=model,
        admin_url=admin_url,
        use_admin_proxy=use_admin_proxy,
    )

    success = await tester.run()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    asyncio.run(main())
