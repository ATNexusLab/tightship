# Update

Claude Code finds a new version through the catalog you added from the release URL. Your options keep their
values across an update.

## Automatically

Auto-update is off by default for a catalog outside Claude Code's official one. To turn it on:

1. Run `/plugin` in a session.
2. Open **Marketplaces**, then `tightship`.
3. Choose **Enable auto-update**.

The session after an update loads the new version.

## By hand

```sh
claude plugin marketplace update tightship
claude plugin update tightship@tightship
```

The new version loads in the next session.
