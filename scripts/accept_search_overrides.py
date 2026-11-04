#!/usr/bin/env python3
"""SEARCH-001-H binary smoke for explicit search configuration (offline, mock model).

Exercises real talos print-mode configuration loading and production registry construction.
No web_search tool call, external provider API call or paid credential is required.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main(binary: Path) -> None:
    binary = binary.resolve()
    ambient_key = "ambient-fixture-tavily-secret"
    stored_key = "stored-fixture-tavily-secret"
    ambient_url = "https://ambient-fixture-search.example.invalid/"
    with tempfile.TemporaryDirectory(prefix="talos-search-override-") as temporary:
        root = Path(temporary)
        home = root / "isolated-home"
        settings = home / ".talos"
        settings.mkdir(parents=True)
        config_path = settings / "config.toml"
        credential_path = settings / "credentials.toml"
        environment = os.environ.copy()
        environment.update({
            "HOME": str(home),
            "USERPROFILE": str(home),
            "TAVILY_API_KEY": ambient_key,
            "SEARXNG_URL": ambient_url,
        })
        arguments = [
            str(binary), "--mock", "--print", "--no-init", "--no-context",
            "SEARCH-001-H offline registry composition fixture",
        ]

        def verify(label: str, *, succeeds: bool, diagnostic: str = "") -> None:
            result = subprocess.run(
                arguments, cwd=root, env=environment, capture_output=True,
                text=True, timeout=45, check=False,
            )
            output = result.stdout + result.stderr
            # Keep diagnostics safe even if a regression starts exposing secret inputs.
            if any(value in output for value in (ambient_key, stored_key, ambient_url)):
                raise AssertionError(f"{label}: secret/URL leaked into CLI output")
            if (result.returncode == 0) != succeeds:
                raise AssertionError(
                    f"{label}: unexpected exit status {result.returncode}; "
                    f"diagnostic={output[:300]!r}"
                )
            if diagnostic and diagnostic not in output:
                raise AssertionError(
                    f"{label}: expected safe diagnostic missing; output={output[:300]!r}"
                )
            print(f"SEARCH_OVERRIDE_E2E {label} PASS")

        def config(search: str) -> None:
            config_path.write_text(
                'provider = "openai"\nmodel = "mock-fixture"\n\n' + search,
                encoding="utf-8",
            )

        # Ambient optional-provider variables are inert without an explicit selector.
        config('[search]\nbackend = "auto"\n')
        verify("auto_with_ambient_optional_values", succeeds=True)

        # Fallback cannot repair a missing explicitly selected credential, even when
        # TAVILY_API_KEY happens to exist in the environment.
        config('[search]\nbackend = "tavily"\nfallback_to_auto = true\n')
        verify(
            "tavily_missing_explicit_credential_fail_closed",
            succeeds=False,
            diagnostic="explicit Tavily search requires a configured tavily credential",
        )

        # Provider-key credential store is read only after backend=tavily selection.
        credential_path.write_text(f'tavily = "{stored_key}"\n', encoding="utf-8")
        verify("tavily_explicit_stored_credential", succeeds=True)

        config('[search]\nbackend = "searxng"\nfallback_to_auto = true\n')
        verify(
            "searxng_missing_explicit_endpoint_fail_closed",
            succeeds=False,
            diagnostic="explicit SearXNG search requires search.url",
        )
        config(
            '[search]\nbackend = "searxng"\n'
            'url = "https://search.example.org"\n'
            'fallback_to_auto = true\n'
        )
        verify("searxng_explicit_endpoint", succeeds=True)

        # Reverting selection to Auto must not use the retained premium credential.
        config('[search]\nbackend = "auto"\n')
        verify("rollback_auto_with_persisted_optional_credential", succeeds=True)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: accept_search_overrides.py PATH_TO_TALOS_BINARY")
    main(Path(sys.argv[1]))
