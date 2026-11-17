# Instale o tightship no Cursor

[English](README.md) · Português

Como instalar, configurar e remover o plugin tightship no editor Cursor e no agente de linha de comando `agent`.

## Antes de começar

- Editor Cursor **3.25.6** ou posterior.
- CLI `agent` **`2026.10.14`** ou posterior. Confira com `agent --version`.
- No Windows, PowerShell 7 para os comandos de hook.
- Git, quando a memória está ligada, já que o vault é um repositório git.
- Quais peças carregam em cada superfície deste harness: a [matriz de suporte](../README.pt-BR.md).

## Páginas

| Página | O que cobre | Tipo |
| --- | --- | --- |
| [Instalação](install.pt-BR.md) | O instalador unificado e o caminho manual em pasta local | How-to |
| [As opções](options.pt-BR.md) | `options.json`, o diálogo Configure e os comandos `tightship options` | Referência |
| [Atualização](update.pt-BR.md) | Versões novas, sozinhas ou pelo instalador | How-to |
| [Desinstalação](uninstall.pt-BR.md) | Remover o tightship e a pasta de opções | How-to |

## Experimente a partir de um clone

Quem faz build a partir do código pode carregar o plugin sem o marketplace:

```sh
cargo xtask build
agent --plugin-dir dist/cursor
```

Ou copie `dist/cursor/` para `~/.cursor/plugins/local/tightship/`, recarregue a janela do editor e use o CLI
com essa pasta. Quem adota o plugin deve seguir a [instalação](install.pt-BR.md).
