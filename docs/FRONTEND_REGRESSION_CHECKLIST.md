# Frontend Regression Checklist

- Refresh the app and confirm the shell appears immediately while wallet balances and backend routing show explicit loading states instead of fake defaults.
- Open Settings before bootstrap finishes and confirm the modal stays disabled/loading until real backend config arrives.
- Save Settings after bootstrap and confirm the saved config persists, preset values rehydrate, and no fallback config is posted.
- Switch wallets repeatedly and confirm the selected wallet changes instantly from cached data while balances refresh in the background without jumping back to an older selection.
- Trigger a quote by typing quickly in dev buy fields and confirm stale responses never overwrite the latest input.
- Open the image library, type quickly in search, and confirm results update without flicker or stale result jumps.
- Upload an image and confirm the multipart upload succeeds, the image appears in the library, and the editor opens with the uploaded image selected.
- Open the reports terminal, change sort, click between reports quickly, and confirm stale report responses do not overwrite the latest active entry.
- Open the sniper modal, confirm same-time / on-submit / block-offset trigger rows render correctly, and verify retry is only shown for same-time rows.
- Refresh with sniper and automatic dev-sell settings already enabled and confirm both rehydrate with the correct visible button/panel state.
- Launch with same-time sniper enabled and confirm the inline same-time safeguard notice only appears when sniper fees exceed launch fees.
- Open the popout view with output and reports open and closed, and confirm workspace sizing follows the current visibility state without stacked or glitched layout.
- Run Build, Simulate, and Deploy and confirm the action buttons unblock as soon as the main request finishes while wallet/report refreshes continue in the background.
- Reload after CSS or JS changes and confirm updated assets load without requiring a hard refresh.
