# Tokens

The docs site carries the same values in
[`stylesheets/docs-theme.css`](../stylesheets/docs-theme.css). Whoever changes a token here changes it
there in the same commit.

Names and types follow the [Design Tokens Format 2025.10](https://www.designtokens.org/TR/2025.10/)
(W3C DTCG): each name is the token's path, and the columns are its `color`, `fontFamily`, and `fontWeight`
values.

## Color

| Token | Light | Dark | Where |
| --- | --- | --- | --- |
| Freight ink · `color.freight-ink` | `#121212` | `#121212` | Banner background, docs site header in both schemes, the mark's manifest bars |
| Label white · `color.label-white` | `#FAFAF7` | `#FAFAF7` | Wordmark, text on ink, the mark's eyelet |
| Safety orange · `color.safety-orange` | `#FF5A1F` | `#FF5A1F` | The mark, the tagline, the banner's bottom strap, the docs header's rule, accent |
| Steel · `color.steel` | `#8A939C` | `#8A939C` | The mark's string, secondary banner text |
| Link · `color.link` | `#C2410C` | `#FF7A45` | Links on the docs site; safety orange is too light for text on white |

Safety orange is an accent. It never fills a section, and text is never set in it on a white background.

## Type

| Token | Face | Weight | Where |
| --- | --- | --- | --- |
| Display · `font.display` | Barlow Condensed | 800 | Wordmark, banner, `h1` and `h2` on the docs site; `h1` in capitals |
| Tagline · `font.tagline` | Barlow Condensed | 600, tracked +12% | The tagline and the banner's secondary line |
| Text · `font.text` | Barlow | 400 and 600 | Running text on the docs site |

All three come from Google Fonts. In the SVG assets the letters are outlines, so no font loads.
