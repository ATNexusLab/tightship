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

`--prune` also removes the plugin's data folder. Claude Code deletes tightship's options from your user settings.

## 3. Remove the catalog

```sh
claude plugin marketplace remove tightship
```

Your vault is never touched.
