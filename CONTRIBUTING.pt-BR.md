# Como contribuir com o tightship

[English](CONTRIBUTING.md) · Português

Relatos de bug, ideias e pull requests são bem-vindos. Esta página mostra o caminho de uma mudança da issue
ao merge, como preparar o repositório e os gates que um pull request precisa passar.

Quem participa segue o [Código de Conduta](CODE_OF_CONDUCT.pt-BR.md). Reporte vulnerabilidades como o
[SECURITY.pt-BR.md](SECURITY.pt-BR.md) descreve, nunca numa issue pública.

Issues, commits e pull requests são escritos em inglês.

## Comece por uma issue

Toda mudança começa por uma issue, inclusive as correções pequenas. Abra um relato de bug ou um pedido de
funcionalidade pelos [formulários de issue](https://github.com/ATNexusLab/tightship/issues/new/choose) e
espere o mantenedor confirmar o escopo antes de escrever código. Um pull request sem issue é fechado com um
pedido para abrir uma.

Uma mudança no que o produto precisa fazer é uma mudança nos [requisitos](docs/requirements/README.md). Ela
passa por uma issue `requirement-change`, e o mantenedor decide antes de qualquer código mudar.

## Prepare o ambiente

Você precisa de:

- `rustup`. O toolchain está fixado em `rust-toolchain.toml`, e o `rustup` o instala na primeira chamada
  ao `cargo`.
- gitleaks 8.30.1 no `PATH`, para o `cargo xtask scan` e os testes.
- Node.js, para o `npx markdownlint-cli2`.
- `uv`, para o site de docs via `uvx`.
- Git for Windows, no Windows. Rode todos os comandos no Git Bash.

Depois crie o `.identity-terms` na raiz do seu clone. O git o ignora. Coloque uma linha por nome, endereço de
e-mail, nome de usuário e handle seus. O build falha se algum deles aparecer no que ele entrega, então a sua
identidade nunca vaza para o plugin. O build também exige que o arquivo exista.

## Faça a mudança

1. Faça um fork do repositório e crie a branch a partir de `develop`: `<type>/<issue>-<slug>`, por exemplo
   `fix/52-hook-exit-code`.
2. Mantenha cada peça no seu lugar:
   - `content/` guarda a única cópia de cada peça e não cita harness, nem caminho ou ferramenta de um. Uma
     referência a outra peça, a uma capacidade de harness ou a uma opção é um token `{{<kind>:<name>}}`.
   - `harnesses/<harness>/` é o único lugar onde um harness é citado.
   - `dist/` é gerado pelo `cargo xtask build`. Nunca o edite nem o commite.
3. Escreva o código em Rust e as docs em inglês. A prosa em `docs/` quebra em cerca de 105 colunas. Quando
   mudar o `README.md`, o `CONTRIBUTING.md`, o `SECURITY.md`, o `CODE_OF_CONDUCT.md` ou uma página de
   `docs/install/`, atualize a tradução `.pt-BR.md` no mesmo commit. Se você não lê português, diga isso no
   pull request, e o mantenedor atualiza a tradução.
4. Adicione um teste quando a mudança altera um comportamento que alguém observa, como lógica, uma regra ou
   uma correção de bug, e escreva o teste antes do código. Mudanças de estilo, texto, configuração ou docs
   não precisam de teste.
5. Atualize as docs que a mudança afeta no mesmo pull request.

## Rode os gates

Rode do mais barato ao mais caro. Cada um precisa sair com 0 e sem avisos.

| Gate | Comando |
| --- | --- |
| Lint de Markdown | `npx markdownlint-cli2@0.23.3` |
| Lint de Rust | `cargo clippy --workspace --all-targets -- -D warnings` |
| Scan de segredos e identidade | `cargo xtask scan` |
| Checagem de tipos | `cargo check --workspace --all-targets` |
| Formatação | `cargo fmt --all --check` |
| Build das docs | `uvx zensical==0.0.70 build` |
| Build do plugin | `cargo xtask build` |
| Testes | `cargo test --workspace` |

Para testar o seu build numa sessão, rode `claude --plugin-dir dist/claude-code`.

## Abra o pull request

- Os commits seguem os [Conventional Commits](https://www.conventionalcommits.org/): uma linha no
  imperativo, em inglês, como `fix: keep the session alive when the vault is missing`.
- O pull request mira `develop` e diz `Closes #<issue>` na descrição.
- Preencha o template: o que mudou, por quê e como você conferiu.

Num pull request vindo de um fork, o workflow de Rust falha no build do plugin: um fork não recebe os
segredos do repositório, então o runner não tem `.identity-terms`. Os seus gates locais são a checagem que
vale até o mantenedor enviar a sua branch para este repositório, onde a CI roda em macOS, Linux e Windows.

O mantenedor revisa cada pull request e faz o merge em `develop`. Os releases vão de `develop` para `main`.

## Licença

Ao contribuir, você concorda que a sua contribuição é licenciada sob a [Licença MIT](LICENSE).
