"""Unit tests for MemoryClient and AsyncMemoryClient."""

import pytest
from datetime import datetime
from unittest.mock import MagicMock, AsyncMock, patch
import httpx

from memory_sdk import MemoryClient, AsyncMemoryClient


class TestMemoryClientInit:
    """Test MemoryClient initialization."""

    def test_client_initialization(self):
        """Test that client initializes with correct headers and base URL."""
        client = MemoryClient(api_key="test-key", base_url="http://api.example.com")
        assert client._base_url == "http://api.example.com"
        assert client._client.headers["Authorization"] == "Bearer test-key"
        assert client._client.headers["Content-Type"] == "application/json"
        client.close()

    def test_client_initialization_with_trailing_slash(self):
        """Test that client strips trailing slash from base_url."""
        client = MemoryClient(api_key="test-key", base_url="http://api.example.com/")
        assert client._base_url == "http://api.example.com"
        client.close()

    def test_client_context_manager(self):
        """Test that client works as context manager."""
        with MemoryClient(api_key="test-key") as client:
            assert client is not None
        # After exiting context, client should be closed


class TestMemoryClientAdd:
    """Test MemoryClient.add() method."""

    def test_add_basic(self):
        """Test basic add method with required parameters."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"episode_id": "ep-123", "status": "processing"}
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.add("Test content")

            mock_client.post.assert_called_once()
            call_args = mock_client.post.call_args
            assert call_args[0][0] == "/v1/memory"
            assert call_args[1]["json"]["content"] == "Test content"
            assert call_args[1]["json"]["role"] == "user"
            assert result == {"episode_id": "ep-123", "status": "processing"}

    def test_add_with_all_parameters(self):
        """Test add method with all optional parameters."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"episode_id": "ep-123"}
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            metadata = {"source": "test"}
            result = client.add(
                "Test content",
                user_id="user-123",
                role="assistant",
                session_id="session-456",
                agent_id="agent-789",
                metadata=metadata,
            )

            call_args = mock_client.post.call_args
            payload = call_args[1]["json"]
            assert payload["content"] == "Test content"
            assert payload["user_id"] == "user-123"
            assert payload["role"] == "assistant"
            assert payload["session_id"] == "session-456"
            assert payload["agent_id"] == "agent-789"
            assert payload["metadata"] == metadata

    def test_add_batch(self):
        """Test add_batch method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = [
                {"episode_id": "ep-1", "status": "processing"},
                {"episode_id": "ep-2", "status": "processing"},
            ]
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            items = [
                {"content": "Content 1", "user_id": "user-1"},
                {"content": "Content 2", "user_id": "user-2"},
            ]
            result = client.add_batch(items)

            call_args = mock_client.post.call_args
            assert call_args[0][0] == "/v1/memory/batch"
            assert call_args[1]["json"] == items
            assert len(result) == 2


class TestMemoryClientSearch:
    """Test MemoryClient.search() method."""

    def test_search_basic(self):
        """Test basic search with required parameters."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"results": [], "total": 0, "query_ms": 10}
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.search("What are user preferences?")

            call_args = mock_client.post.call_args
            assert call_args[0][0] == "/v1/memory/search"
            payload = call_args[1]["json"]
            assert payload["query"] == "What are user preferences?"
            assert payload["limit"] == 10

    def test_search_with_new_parameters(self):
        """Test search with new enhanced parameters."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"results": [], "total": 0}
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.search(
                "Test query",
                user_id="user-123",
                session_id="session-456",
                agent_id="agent-789",
                tags=["tag1", "tag2"],
                entity_types=["person", "org"],
                min_importance=0.7,
                apply_decay=True,
                graph_depth=4,
                limit=20,
            )

            call_args = mock_client.post.call_args
            payload = call_args[1]["json"]
            assert payload["query"] == "Test query"
            assert payload["user_id"] == "user-123"
            assert payload["session_id"] == "session-456"
            assert payload["agent_id"] == "agent-789"
            assert payload["tags"] == ["tag1", "tag2"]
            assert payload["entity_types"] == ["person", "org"]
            assert payload["min_importance"] == 0.7
            assert payload["apply_decay"] is True
            assert payload["graph_depth"] == 4
            assert payload["limit"] == 20

    def test_search_with_temporal_filters(self):
        """Test search with temporal filter parameters."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"results": []}
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            after_time = datetime(2024, 1, 1, 12, 0, 0)
            before_time = datetime(2024, 1, 31, 12, 0, 0)

            result = client.search(
                "Test",
                after=after_time,
                before=before_time,
            )

            call_args = mock_client.post.call_args
            payload = call_args[1]["json"]
            assert "temporal_filter" in payload
            assert payload["temporal_filter"]["after"] == after_time.isoformat()
            assert payload["temporal_filter"]["before"] == before_time.isoformat()


class TestMemoryClientGraphOperations:
    """Test MemoryClient graph operations."""

    def test_traverse_entity(self):
        """Test traverse_entity method with correct parameters."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"entity_id": "e-1", "traversal": []}
            mock_client = MagicMock()
            mock_client.get.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.traverse_entity("entity-123", depth=4, limit=100)

            call_args = mock_client.get.call_args
            assert call_args[0][0] == "/v1/graph/entities/entity-123/traverse"
            assert call_args[1]["params"]["depth"] == 4
            assert call_args[1]["params"]["limit"] == 100

    def test_find_path(self):
        """Test find_path method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"path": ["e1", "e2", "e3"], "length": 2}
            mock_client = MagicMock()
            mock_client.get.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.find_path("entity-1", "entity-2", max_depth=6)

            call_args = mock_client.get.call_args
            assert call_args[0][0] == "/v1/graph/path/entity-1/entity-2"
            assert call_args[1]["params"]["max_depth"] == 6

    def test_get_entity_neighbors(self):
        """Test get_entity_neighbors method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"neighbors": []}
            mock_client = MagicMock()
            mock_client.get.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.get_entity_neighbors("entity-123", rel_type="knows", limit=30)

            call_args = mock_client.get.call_args
            assert call_args[0][0] == "/v1/graph/entities/entity-123/neighbors"
            assert call_args[1]["params"]["rel_type"] == "knows"
            assert call_args[1]["params"]["limit"] == 30


class TestMemoryClientWebhooks:
    """Test MemoryClient webhook operations."""

    def test_list_webhooks(self):
        """Test list_webhooks method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = [
                {"id": "wh-1", "url": "http://example.com/webhook"},
                {"id": "wh-2", "url": "http://example.com/webhook2"},
            ]
            mock_client = MagicMock()
            mock_client.get.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.list_webhooks()

            mock_client.get.assert_called_once_with("/v1/webhooks")
            assert len(result) == 2
            assert result[0]["id"] == "wh-1"

    def test_create_webhook(self):
        """Test create_webhook method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {
                "webhook_id": "wh-123",
                "url": "http://example.com/webhook",
                "status": "active",
            }
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.create_webhook(
                "http://example.com/webhook",
                ["memory.created", "entity.updated"],
                secret="secret-key",
            )

            call_args = mock_client.post.call_args
            assert call_args[0][0] == "/v1/webhooks"
            payload = call_args[1]["json"]
            assert payload["url"] == "http://example.com/webhook"
            assert payload["events"] == ["memory.created", "entity.updated"]
            assert payload["secret"] == "secret-key"

    def test_delete_webhook(self):
        """Test delete_webhook method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_client = MagicMock()
            mock_client.delete.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            client.delete_webhook("wh-123")

            mock_client.delete.assert_called_once_with("/v1/webhooks/wh-123")


class TestMemoryClientMemoryOperations:
    """Test MemoryClient memory export/import operations."""

    def test_export_memories(self):
        """Test export_memories method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {
                "export_id": "exp-123",
                "status": "processing",
                "download_url": "http://example.com/download",
            }
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            result = client.export_memories("user-123", include_graph=True, format="csv")

            call_args = mock_client.post.call_args
            assert call_args[0][0] == "/v1/memory/export"
            payload = call_args[1]["json"]
            assert payload["user_id"] == "user-123"
            assert payload["include_graph"] is True
            assert payload["format"] == "csv"

    def test_import_memories(self):
        """Test import_memories method."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {
                "import_id": "imp-123",
                "status": "processing",
                "imported": 5,
                "skipped": 0,
            }
            mock_client = MagicMock()
            mock_client.post.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            items = [
                {"content": "Memory 1", "user_id": "user-1"},
                {"content": "Memory 2", "user_id": "user-1"},
            ]
            result = client.import_memories(items, conflict_strategy="merge")

            call_args = mock_client.post.call_args
            assert call_args[0][0] == "/v1/memory/import"
            payload = call_args[1]["json"]
            assert payload["items"] == items
            assert payload["conflict_strategy"] == "merge"


class TestMemoryClientErrorHandling:
    """Test MemoryClient error handling."""

    def test_raise_for_status_on_error(self):
        """Test that raise_for_status is called on response errors."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.raise_for_status.side_effect = httpx.HTTPStatusError(
                "404 Not Found",
                request=MagicMock(),
                response=mock_response,
            )
            mock_client = MagicMock()
            mock_client.get.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            with pytest.raises(httpx.HTTPStatusError):
                client.get("nonexistent-id")

    def test_delete_raises_on_error(self):
        """Test that delete raises error on non-2xx status."""
        with patch("httpx.Client") as mock_httpx:
            mock_response = MagicMock()
            mock_response.raise_for_status.side_effect = httpx.HTTPStatusError(
                "403 Forbidden",
                request=MagicMock(),
                response=mock_response,
            )
            mock_client = MagicMock()
            mock_client.delete.return_value = mock_response
            mock_httpx.return_value = mock_client

            client = MemoryClient(api_key="test-key")
            client._client = mock_client

            with pytest.raises(httpx.HTTPStatusError):
                client.delete("memory-id")


class TestAsyncMemoryClientInit:
    """Test AsyncMemoryClient initialization."""

    @pytest.mark.asyncio
    async def test_async_client_initialization(self):
        """Test that async client initializes correctly."""
        client = AsyncMemoryClient(api_key="test-key", base_url="http://api.example.com")
        assert client._base_url == "http://api.example.com"
        assert client._client.headers["Authorization"] == "Bearer test-key"
        await client.close()

    @pytest.mark.asyncio
    async def test_async_client_context_manager(self):
        """Test that async client works as async context manager."""
        async with AsyncMemoryClient(api_key="test-key") as client:
            assert client is not None


class TestAsyncMemoryClientSearch:
    """Test AsyncMemoryClient.search() method."""

    @pytest.mark.asyncio
    async def test_async_search_with_new_parameters(self):
        """Test async search with new enhanced parameters."""
        with patch("httpx.AsyncClient") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"results": [], "total": 0}
            mock_client = AsyncMock()
            mock_client.post = AsyncMock(return_value=mock_response)
            mock_httpx.return_value = mock_client

            client = AsyncMemoryClient(api_key="test-key")
            client._client = mock_client

            result = await client.search(
                "Test query",
                user_id="user-123",
                tags=["tag1", "tag2"],
                entity_types=["person", "org"],
                min_importance=0.7,
                apply_decay=True,
                graph_depth=4,
            )

            call_args = mock_client.post.call_args
            payload = call_args[1]["json"]
            assert payload["query"] == "Test query"
            assert payload["tags"] == ["tag1", "tag2"]
            assert payload["entity_types"] == ["person", "org"]
            assert payload["graph_depth"] == 4


class TestAsyncMemoryClientGraphOperations:
    """Test AsyncMemoryClient graph operations."""

    @pytest.mark.asyncio
    async def test_async_traverse_entity(self):
        """Test async traverse_entity method."""
        with patch("httpx.AsyncClient") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {"entity_id": "e-1", "traversal": []}
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response)
            mock_httpx.return_value = mock_client

            client = AsyncMemoryClient(api_key="test-key")
            client._client = mock_client

            result = await client.traverse_entity("entity-123", depth=5, limit=75)

            call_args = mock_client.get.call_args
            assert call_args[0][0] == "/v1/graph/entities/entity-123/traverse"
            assert call_args[1]["params"]["depth"] == 5
            assert call_args[1]["params"]["limit"] == 75


class TestAsyncMemoryClientWebhooks:
    """Test AsyncMemoryClient webhook operations."""

    @pytest.mark.asyncio
    async def test_async_create_webhook(self):
        """Test async create_webhook method."""
        with patch("httpx.AsyncClient") as mock_httpx:
            mock_response = MagicMock()
            mock_response.json.return_value = {
                "webhook_id": "wh-123",
                "url": "http://example.com/webhook",
            }
            mock_client = AsyncMock()
            mock_client.post = AsyncMock(return_value=mock_response)
            mock_httpx.return_value = mock_client

            client = AsyncMemoryClient(api_key="test-key")
            client._client = mock_client

            result = await client.create_webhook(
                "http://example.com/webhook",
                ["memory.created"],
            )

            call_args = mock_client.post.call_args
            assert call_args[0][0] == "/v1/webhooks"
            payload = call_args[1]["json"]
            assert payload["url"] == "http://example.com/webhook"
            assert payload["events"] == ["memory.created"]

    @pytest.mark.asyncio
    async def test_async_delete_webhook(self):
        """Test async delete_webhook method."""
        with patch("httpx.AsyncClient") as mock_httpx:
            mock_response = MagicMock()
            mock_client = AsyncMock()
            mock_client.delete = AsyncMock(return_value=mock_response)
            mock_httpx.return_value = mock_client

            client = AsyncMemoryClient(api_key="test-key")
            client._client = mock_client

            await client.delete_webhook("wh-456")

            mock_client.delete.assert_called_once_with("/v1/webhooks/wh-456")
