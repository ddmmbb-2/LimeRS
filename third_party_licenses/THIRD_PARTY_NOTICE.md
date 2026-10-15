# Third-party data notices

## Built-in `assets/zhuyin.tsv` and `assets/pinyin_keys.tsv`

The underlying Mandarin pronunciation readings were derived by LimeRS from the
`xpinyin-database.def` file provided in the TeX Live `xpinyin` package.

- Original maintainer: Qing Lee (sobenlee@gmail.com)
- Original database: Unihan readings from Unicode 14.0.0; the TeX data was built in 2022
- Upstream package: https://ctan.org/pkg/xpinyin
- Upstream license notice: LaTeX Project Public License (LPPL) 1.3c or later
  https://www.latex-project.org/lppl/lppl-1-3c/
- Original document: `xpinyin-database.def`
- Transformation: extracted character/pinyin records, converted pinyin tones
  to standard Da-Qian Zhuyin keyboard key sequences, maintained tone-specific codes,
  and combined with independently authored test phrases/scores.

The derived pronunciation portions remain subject to the upstream data license;
they are **not** relicensed under the LimeRS source code's MIT license.
A copy of the LPPL TeX source is shipped as
`third_party_licenses/LPPL-1.3c-tex-source.tex`.

## Optional user-supplied Rime dictionaries

This ZIP does **not** redistribute the Rime dictionaries. The import feature
only reads a local dictionary the user supplies. Users need to comply with the
relevant licenses and provenance when importing, redistributing, or packaging
those data files. For example, the Rime Terra Pinyin recipe's code is
LGPL-3.0, and its dictionary comments mention CC-CEDICT (CC BY-SA 3.0).

- https://github.com/rime/rime-terra-pinyin
- https://github.com/rime/rime-luna-pinyin

## Fonts

No Microsoft font files are included. On Windows the application attempts to
load the user's system-installed Microsoft JhengHei (`msjh.ttc`).
