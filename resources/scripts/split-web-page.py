#!/usr/bin/env python3
"""Split a monolithic robominer-web page into mod.rs + render.rs + tests.rs.

See CONTRIBUTING.md for when to use this script and how to set line boundaries.
"""

from __future__ import annotations

import sys
from pathlib import Path


def split_page(
    source: Path,
    render_start: int,
    helper_start: int,
    tests_start: int,
    render_imports: str,
    tests_imports: str,
    extra_mod_tail: str = "",
) -> None:
    lines = source.read_text().splitlines(keepends=True)
    target = source.parent / source.stem

    mod_body = lines[: render_start - 1]
    render_body = lines[render_start - 1 : helper_start - 1]
    helper_body = lines[helper_start - 1 : tests_start - 1]
    tests_body = lines[tests_start - 1 :]

    mod_rs = (
        "".join(mod_body)
        + """
mod render;

#[cfg(test)]
mod tests;
"""
        + "".join(helper_body)
        + extra_mod_tail
    )
    mod_rs = mod_rs.replace(
        f"Response::html(render_{source.stem}(",
        f"Response::html(render::render_{source.stem}(",
    )

    render_rs = render_imports + "".join(render_body)

    tests_inner = "".join(tests_body)
    if tests_inner.startswith("#[cfg(test)]\n"):
        tests_inner = tests_inner[len("#[cfg(test)]\n") :]
    tests_inner = tests_inner.replace("mod tests {\n", "", 1)
    stripped = tests_inner.rstrip()
    if stripped.endswith("}"):
        lines = stripped.splitlines()
        if lines and lines[-1].strip() == "}":
            stripped = "\n".join(lines[:-1]) + "\n"
    tests_inner = stripped + ("\n" if not stripped.endswith("\n") else "")
    dedented = []
    for line in tests_inner.splitlines(keepends=True):
        if line.startswith("    "):
            dedented.append(line[4:])
        else:
            dedented.append(line)
    tests_rs = tests_imports + "".join(dedented)

    target.mkdir(parents=True, exist_ok=True)
    (target / "mod.rs").write_text(mod_rs)
    (target / "render.rs").write_text(render_rs)
    (target / "tests.rs").write_text(tests_rs)
    source.unlink()
    print(f"Split {source.name} -> {target}/")


if __name__ == "__main__":
    print(
        """split-web-page.py — split robominer-web/src/<page>.rs into a page module.

Edit and uncomment the split_page(...) example in this file (above sys.exit), then run:

  python3 resources/scripts/split-web-page.py

Boundaries (1-based line numbers in the source file):
  render_start  — first line of the render_* function
  helper_start  — first line after render (handler helpers stay in mod.rs)
  tests_start   — first #[cfg(test)] module

See CONTRIBUTING.md § "Splitting a web page module".
"""
    )
    sys.exit(1)

    # Example (uncomment and adjust before running):
    #
    # root = Path(__file__).resolve().parents[2]
    # web = root / "robominer-web/src"
    #
    # split_page(
    #     web / "example_page.rs",
    #     render_start=42,
    #     helper_start=300,
    #     tests_start=350,
    #     render_imports="""use crate::html::layout;
    # use crate::example_page::ExamplePageState;
    #
    # """,
    #     tests_imports="""use crate::{Request, ServerConfig};
    #
    # use super::render::render_example_page;
    # use super::{ExamplePageState, example_page};
    #
    # """,
    # )
