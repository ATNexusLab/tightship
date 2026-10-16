# Instale o tightship no Cursor

[English](README.md) · Português

Como instalar, configurar e remover o plugin tightship no editor Cursor e no agente de linha de comando `agent`.

## Antes de começar

- Editor Cursor **3.25.6** ou posterior.
- CLI `agent` **`2026.10.14`** ou posterior. Confira com `agent --version`.
- No Windows, PowerShell 7 para os comandos de hook.
- Git, quando a memória está ligada, já que o vault é um repositório git.

## Páginas

| Página | O que cobre |
| --- | --- |
| [Os companheiros](companions.pt-BR.md) | Os três plugins a instalar antes do tightship |
| [Instalação](install.pt-BR.md) | Instalar o tightship pelo marketplace git |
| [As opções](options.pt-BR.md) | `options.json`, o diálogo Configure e os comandos `tightship options` |
| [Desinstalação](uninstall.pt-BR.md) | Remover o tightship, a pasta de opções e, se quiser, os companheiros |

## Experimente a partir de um clone

Quem faz build a partir do código pode carregar o plugin sem o marketplace:

```sh
cargo xtask build
agent --plugin-dir dist/cursor
```

Ou copie `dist/cursor/` para `~/.cursor/plugins/local/tightship/`, recarregue a janela do editor e use o CLI
com essa pasta. Quem instala pelo git deve seguir a [instalação](install.pt-BR.md).
