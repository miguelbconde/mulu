# Isolated testing with a separate Windows account (decision D19)

Test versions that read files are tried in a separate, standard (non-administrator) Windows account that cannot see the tester's personal files, against a copy of the library. This replaces Windows Sandbox, which fails to start on some Windows 11 computers (see `results.md`). Windows Sandbox remains a good option for testers on whose computers it works.

What it protects: a standard account cannot read other users' profile folders (Documents, Desktop, Downloads, cloud folders), so a faulty or compromised test version can reach only the copy. What it does not protect: the rest of Windows is shared, so this is a separation of accounts, not a disposable machine.

## One-time setup

1. **Create the account.** Settings → Accounts → Other users → Add account → "I don't have this person's sign-in information" → "Add a user without a Microsoft account". Name: `Mulu Test`. A password is optional. Keep the account type as **Standard user**.
2. **Copy the library to a shared place.** From your own account, copy your library folder (or part of it) to `C:\Users\Public\Documents\Mulu test library`. Public Documents is the only folder both accounts can read. Copy, never move: the originals stay where they are.
3. **Sign in to the test account once.** Start → your account picture → `Mulu Test`. The first sign-in takes a few minutes. Decline any offer to sign in to OneDrive or a Microsoft account.

## For each test version

1. Sign in to `Mulu Test`.
2. Download and install the test version there, following that version's test script.
3. When Mulu asks for the folder with your books, choose `Public Documents\Mulu test library`.
4. When done, uninstall Mulu from that account and sign out.
