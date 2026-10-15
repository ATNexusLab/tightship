# Design

tightship has no screens of its own: it runs inside a harness's interface. Design covers the brand, the
tokens the README, the docs site, and the release page use, and nothing else.

The brand assets are SVG written by hand. No design tool owns them.

| Doc | What it holds |
| --- | --- |
| [Brand](brand/README.md) | What the brand stands for, its voice, the mark, the banner, and how to use each |
| [Tokens](tokens.md) | Every color and type token, in light and dark, and where each is applied |

There is no `pages/`: the status line is the only thing the plugin draws, and its
[data model](../data-model/status-line.md) specifies it.
