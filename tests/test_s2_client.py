from __future__ import annotations

import io
import json
import os
import unittest
import urllib.error
import warnings
from unittest import mock

from bridges.providers import s2_client


class _FakeResponse:
    def __init__(self, payload: dict[str, object]) -> None:
        self._payload = payload

    def read(self) -> bytes:
        return json.dumps(self._payload).encode("utf-8")

    def __enter__(self) -> "_FakeResponse":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        return None


class SemanticScholarClientTests(unittest.TestCase):
    def test_make_request_adds_api_key_header_when_present(self) -> None:
        captured_headers: dict[str, str] = {}

        def fake_urlopen(request, timeout):  # type: ignore[no-untyped-def]
            del timeout
            captured_headers.update(dict(request.header_items()))
            return _FakeResponse({"data": []})

        with mock.patch.dict(os.environ, {"SEMANTIC_SCHOLAR_API_KEY": "demo-key"}, clear=False):
            with mock.patch("urllib.request.urlopen", side_effect=fake_urlopen):
                result = s2_client._make_request("https://example.com")

        self.assertEqual(result, {"data": []})
        self.assertEqual(captured_headers["X-api-key"], "demo-key")

    def test_make_request_retries_http_429_then_succeeds(self) -> None:
        attempts = {"count": 0}

        def fake_urlopen(request, timeout):  # type: ignore[no-untyped-def]
            del request, timeout
            attempts["count"] += 1
            if attempts["count"] == 1:
                raise urllib.error.HTTPError(
                    "https://example.com",
                    429,
                    "Too Many Requests",
                    {"Retry-After": "1"},
                    None,
                )
            return _FakeResponse({"data": [{"paperId": "123"}]})

        with warnings.catch_warnings():
            warnings.simplefilter("ignore", ResourceWarning)
            with mock.patch("urllib.request.urlopen", side_effect=fake_urlopen):
                with mock.patch("time.sleep") as sleep_mock:
                    result = s2_client._make_request("https://example.com")

        self.assertEqual(result["data"], [{"paperId": "123"}])
        self.assertEqual(attempts["count"], 2)
        sleep_mock.assert_called_once_with(1.0)

    def test_make_request_returns_terminal_error_after_retries(self) -> None:
        http_error = urllib.error.HTTPError(
            "https://example.com",
            429,
            "Too Many Requests",
            {},
            None,
        )

        with mock.patch("urllib.request.urlopen", side_effect=http_error):
            with mock.patch("time.sleep"):
                result = s2_client._make_request("https://example.com")

        self.assertEqual(result["data"], [])
        self.assertIn("HTTP Error 429", result["error"])


if __name__ == "__main__":
    unittest.main()
