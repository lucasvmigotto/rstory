# Tokens (final)

| Role | 16-color | 256 | Truecolor (theme `rstory`) dark bg / light bg | Mono cue |
|---|---|---|---|---|
| `accent` | Cyan | 37 | #2AA198 / #1F7A73 | bold + underline |
| `selected` | reverse | reverse | reverse | reverse + `>` |
| `muted` | BrightBlack | 245 | #8A8A8A / #6C6C6C | none (never information-only) |
| `pinned` | Yellow | 178 | #D7AF00 / #8A6D00 | glyph `📌`/`[p]` |
| `warning` | Yellow | 178 | #D7AF00 / #8A6D00 | words "Paused"/"Read-only" |
| `danger` | Red | 160 | #D70000 / #AF0000 | word "Delete?" |

Contrast (truecolor): accent #1F7A73 on #FFFFFF = 5.4:1; #2AA198 on #1C1C1C = 5.9:1; muted #6C6C6C on #FFFFFF = 5.3:1; #8A8A8A on #1C1C1C = 5.3:1; pinned #8A6D00 on #FFFFFF = 4.9:1; danger #AF0000 on #FFFFFF = 6.9:1.

Glyphs: `📌`→`[p]` · `│`→`|` · `─`→`-` · `…`→`...` · `⏎`→`\n` · `✓`→`ok` · `!`→`warn` · `✗`→`FAIL` · `>` (selection, same in both).

Spacing: gutter 1 cell; age column 4 cells right-aligned; pin column 2 cells; preview split 50% (≥ 100 cols), 40% (60–99), hidden (< 60).
