# Instalação

[English](install.md) · Português

Conclua [os companheiros](companions.pt-BR.md) primeiro.

Não rode um comando de workflow até ponytail, i-have-adhd e humanizer aparecerem como **Installed** em
Customize → Plugins.

Não instale o tightship como plugin do Claude Code na mesma máquina. O Cursor pode carregar plugins do Claude
Code como **Imported**, e as mesmas peças seriam carregadas duas vezes.

Com os quatro catálogos já adicionados (companheiros primeiro, depois este repositório), instale o tightship:

```sh
agent plugin install tightship@tightship
```

No editor, você pode instalar em Customize → Plugins depois que os catálogos existirem.

A instalação não pede opção no shell. Hooks e o binário leem opções só em `~/.cursor/tightship/options.json`,
não nos valores do Customize → Configure. Defina opções com [tightship options](options.pt-BR.md) depois da
instalação.

## Confira

Em Customize → Plugins, o tightship aparece como **Installed** com skills e comandos listados. ponytail,
i-have-adhd e humanizer continuam instalados desde a etapa dos companheiros.

Numa sessão interativa do `agent`, pergunte quais comandos slash estão disponíveis. A resposta deve listar
`/spec` e os outros comandos do workflow.

Depois de um `cargo xtask build` local, você pode confirmar o mesmo com `agent --plugin-dir dist/cursor` em
vez da instalação pelo marketplace.
