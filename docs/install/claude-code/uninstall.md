# Uninstall

English · [Português](uninstall.pt-BR.md)

## 1. Put your status line back

Skip this step if you never turned tightship's status line on.

Claude Code runs no plugin code on uninstall, and the uninstall deletes the folder where tightship keeps your
previous status line. So turn the option off first, then open one session:

```sh
echo '{"status_line":"false"}' | claude plugin configure tightship@tightship --values-stdin
```

That session's start puts your previous `statusLine` back, or removes the key when you had none.

## 2. Remove the plugin

```sh
claude plugin uninstall tightship@tightship --prune -y
```

`--prune` also removes the companions the install added, and the plugin's data folder. A companion you
installed yourself stays. Claude Code deletes tightship's options from your user settings.

## 3. Remove the catalogs

```sh
claude plugin marketplace remove tightship
claude plugin marketplace remove ponytail
claude plugin marketplace remove i-have-adhd
claude plugin marketplace remove humanizer
```

Keep a companion's catalog if you still use that companion.

Your vault is never touched.
