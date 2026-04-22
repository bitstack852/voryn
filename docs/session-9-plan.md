# Voryn — Session 9 Plan

**Last updated:** 2026-04-22  
**Branch:** `claude/assess-main-branch-gCIii`  
**Status:** Planning complete — ready to code

---

## Current Snapshot

| Layer | State |
|-------|-------|
| iOS build | Builds and runs on iPhone-NST + Acumen-XR |
| Android build | APK builds — no device to test, shelved |
| Relay | Live at `ws://boot1.voryn.bitstack.website:4001/ws` |
| Messages | Arriving on both devices — **NOT encrypted (see Bug 1)** |
| Rust bridge | Connected on iOS — Ed25519, encrypt/decrypt functions working |
| Screens | 10 screens implemented, dark theme, passcode lock |

---

## Known Bugs (fix before features)

### Bug 1 — Messages sent as plaintext ❌ CRITICAL
`ChatScreen.handleSend` makes two calls:
```ts
// Encrypts, but sends via dead libp2p path (not connected) — does nothing useful
await VorynBridge.sendMessage(contactPubkeyHex, text);

// Sends RAW PLAINTEXT over WebSocket relay ← the bug
NetworkService.sendToPeer(contactPubkeyHex, text, messageId);
```
Receive side also stores payload directly without decrypting.  
**Every message in transit is readable by anyone watching the relay.**

### Bug 2 — Merge conflict in ChatScreen.tsx ❌
Lines 17–20 have unresolved `<<<<<<< HEAD` / `=======` / `>>>>>>>` markers
leaving a duplicate `import * as NetworkService` in the file.

---

## Signed-Off Decisions

| # | Question | Decision |
|---|----------|----------|
| 1 | Existing plaintext message history | **Wipe** on this session's install |
| 2 | Denied contact requests | **Dismiss** — not a permanent block |
| 3 | Group message encryption | **Shared group key** via Rust Shamir infrastructure |
| 4 | Tick states | **Sent (✓ grey) + Delivered (✓✓ grey)** — no blue read ticks yet |
| 5 | Offline contact requests | **Client-side queue** — store with sent/delivered status, retry on reconnect |

---

## Task Checklist

---

### TASK 1 — Fix merge conflict in ChatScreen.tsx
**Files:** `apps/mobile/src/screens/ChatScreen.tsx`

- [ ] Remove conflict markers (lines 17–20), keep single `import * as NetworkService`
- [ ] Verify file compiles

---

### TASK 2 — Fix encryption end-to-end
**Files:** `ChatScreen.tsx`, `VorynBridge.ts`, `NetworkService.ts`

#### Send path
- [ ] Remove `NetworkService.sendToPeer(contactPubkeyHex, text, ...)` from `ChatScreen.handleSend`
- [ ] In `VorynBridge.sendMessage`: after encrypting, call `NetworkService.sendToPeer(recipient, encrypted.envelopeHex, messageId)` instead of dead `sendRawToPeer`
- [ ] If Rust bridge unavailable → mark message `failed`, do **not** send plaintext fallback

#### Receive path
- [ ] In `NetworkService.storeIncoming`: call `VorynBridge.decryptMessage(payload, secretKey)` before storing
- [ ] On successful decrypt → store plaintext via `VorynBridge.receiveMessage`
- [ ] On decrypt failure → discard silently (legacy plaintext, pre-fix messages)

#### Wipe existing message history
- [ ] Add one-time migration in `VorynBridge` on app start: if a `@voryn/migrated_v2` key is absent, clear `@voryn/messages` and write the flag

#### Verification
- [ ] Send message Phone A → relay shows opaque hex, not plaintext
- [ ] Message arrives on Phone B decrypted correctly
- [ ] Message marked `failed` (not sent) when bridge unavailable

---

### TASK 3 — Fix double splash screen
**Files:** `apps/mobile/ios/Voryn/LaunchScreen.storyboard`

**Cause:** iOS shows native LaunchScreen, then RN boots and immediately renders
the custom `SplashScreen` component — two splashes back-to-back.

**Fix:** Make the native LaunchScreen a plain black frame (`#050608`) matching the
app background so it's invisible. The custom RN `SplashScreen` handles all branding.

- [ ] Edit `LaunchScreen.storyboard` — set background to `#050608`, remove all labels/images
- [ ] Build and confirm only one splash visible on device

---

### TASK 4 — Message checkmarks
**Files:** `ChatScreen.tsx`, `NetworkService.ts`, `VorynBridge.ts`

#### Tick states
| Status | Display | Trigger |
|--------|---------|---------|
| `pending` | dim single tick | Message saved locally, not yet sent |
| `sent` | grey `✓` | Relay received and routed the message |
| `delivered` | grey `✓✓` | Relay ACK received with matching `message_id` |
| `failed` | red `✗` | Send or encrypt failed |

#### Work
- [ ] Replace Unicode emoji in `statusIcon()` with styled `Text` component ticks (11pt, consistent sizing)
- [ ] `failed` state shows red `✗`, tap to retry
- [ ] Wire relay `ack` in `NetworkService.handleServerMessage` (currently ignored):
  - Parse `message_id` from ACK
  - Fire `onAck(messageId)` callback to registered listeners
- [ ] Add `onAck(handler)` / `offAck` API to `NetworkService`
- [ ] `ChatScreen` subscribes to `onAck`, calls `VorynBridge.updateMessageStatus(messageId, 'delivered')`
- [ ] Add `updateMessageStatus(messageId, status)` to `VorynBridge`

---

### TASK 5 — Message delete
**Files:** `ChatScreen.tsx`, `VorynBridge.ts`

**Scope:** Local delete only (removes from this device's AsyncStorage). No "delete for everyone" yet.

- [ ] Add `deleteMessage(messageId: string)` to `VorynBridge` — filters message out of stored array
- [ ] Long-press handler on message bubble in `ChatScreen`
- [ ] `Alert.alert` confirmation: "Delete Message" / "Cancel"
- [ ] Message list reloads immediately after delete
- [ ] Works for both sent and received messages

---

### TASK 6 — Split Chats tab and Contacts tab
**Files:** `RootNavigator.tsx`, `ContactsScreen.tsx`, new `ChatsScreen.tsx`

**Current:** Single `ContactsScreen` acts as both contact list and chat entry point.  
**Goal:** Bottom tab navigator with two distinct tabs.

#### Chats tab (new `ChatsScreen.tsx`)
- Conversations ordered by most recent message (newest first)
- Each row: avatar, display name, last message preview (truncated ~40 chars), timestamp
- Only shows contacts with at least one message exchanged
- Unread badge (count of unread incoming messages per conversation)
- Tap row → `ChatScreen`

#### Contacts tab (existing `ContactsScreen.tsx`, trimmed)
- Full contact list (address book)
- Tap → `ChatScreen`, long-press → `ContactDetail`
- Badge on tab icon when there are pending incoming contact requests (Task 7)
- Settings reachable via top-right icon (gear) instead of bottom bar button

#### Navigation structure change
```
Before: Stack → ContactsScreen (with inline bottom bar)
After:  Stack → TabNavigator
                  ├── ChatsScreen
                  └── ContactsScreen
```

#### New VorynBridge functions
- [ ] `getConversations()` — returns contacts with ≥1 message, sorted by last message timestamp, with `lastMessage` text + `unreadCount`
- [ ] `markConversationRead(conversationId)` — sets all incoming messages in convo to `status: 'read'` (used when opening a chat)

#### Work
- [ ] Create `ChatsScreen.tsx`
- [ ] Add `getConversations()` and `markConversationRead()` to `VorynBridge`
- [ ] Add bottom tab navigator to `RootNavigator` (React Navigation `createBottomTabNavigator`)
- [ ] Remove inline bottom bar from `ContactsScreen`
- [ ] Move Settings access to top-right header icon in `ContactsScreen`
- [ ] Unread badge on Chats tab icon
- [ ] Pending requests badge on Contacts tab icon (wired in Task 7)

---

### TASK 7 — Contact request approval
**Files:** `VorynBridge.ts`, `NetworkService.ts`, `AddContactScreen.tsx`, new `PendingRequestsScreen.tsx`, `ContactsScreen.tsx`

#### Flow
1. Alice enters Bob's key → taps **Send Request** (not "Add Contact")
2. App stores contact as `status: 'pending_sent'`, sends encrypted `contact_request` payload over relay
3. Request stored with `sent`/`delivered` status — retried on reconnect if undelivered (per Decision 5)
4. Bob's app receives `contact_request` → stores as `status: 'pending_received'`, shows badge on Contacts tab
5. Bob opens **Pending Requests** → sees Alice's key + intro message (if any) → **Approve** or **Deny**
6. Approve → Bob stores Alice as `status: 'approved'`, sends encrypted `contact_accepted` to Alice
7. Deny → sends encrypted `contact_denied` to Alice, removes pending entry
8. Alice receives `contact_accepted` → marks contact `approved`; receives `contact_denied` → marks `denied` (dismissible)

#### Contact status field
```ts
status: 'approved' | 'pending_sent' | 'pending_received' | 'denied'
```

#### Protocol message types (all encrypted, sent over relay alongside chat messages)
```json
{ "type": "contact_request", "from": "<pubkey>", "display_name": "...", "intro": "..." }
{ "type": "contact_accepted", "from": "<pubkey>" }
{ "type": "contact_denied", "from": "<pubkey>" }
```

#### Work
- [ ] Add `status` field to `Contact` type in `VorynBridge`
- [ ] Update `addContact` → creates `pending_sent` entry + sends encrypted `contact_request` via relay
- [ ] Add `pendingOutbox` queue to `VorynBridge` — retry unsent requests on relay reconnect
- [ ] `NetworkService` routes incoming encrypted payloads: detect `contact_request` / `contact_accepted` / `contact_denied` type after decrypt, fire separate handler (not `onMessage`)
- [ ] Add `onContactRequest(handler)` API to `NetworkService`
- [ ] Handle `contact_request` → store `pending_received` contact
- [ ] Handle `contact_accepted` → update contact to `approved`
- [ ] Handle `contact_denied` → mark `denied`, show dismissible notice to sender
- [ ] Create `PendingRequestsScreen.tsx` — list of `pending_received` contacts with Approve/Deny buttons
- [ ] Update `AddContactScreen` — rename button to "Send Request", show "Request sent" confirmation
- [ ] Wire pending requests badge on Contacts tab icon
- [ ] `ContactsScreen` only shows `approved` contacts in main list

---

### TASK 8 — Invite links
**Files:** `ShareKeyScreen.tsx`, `RootNavigator.tsx`, `App.tsx`, `Info.plist`, new `InviteAcceptScreen.tsx`

#### Link format
```
voryn://invite?from=<sender_pubkey_hex_64chars>&t=<random_8byte_hex>
```
- `from` — inviter's full public key
- `t` — random token (makes links non-guessable; revocation deferred)

#### Flow
1. Alice: **Share Key** screen → "Generate Invite Link" → app generates `t`, formats link, opens share sheet
2. Bob taps link → app opens → `InviteAcceptScreen` renders
3. Screen shows: "Invitation from [first 12 chars of key]…" + Accept / Decline
4. Accept → triggers Task 7 contact request (Bob sends `contact_request` to Alice's pubkey)
5. Decline → dismiss, no action

#### Work
- [ ] Register `voryn://` URL scheme in `apps/mobile/ios/Voryn/Info.plist`
- [ ] Handle `Linking.getInitialURL()` + `Linking.addEventListener` in `RootNavigator` or `App.tsx`
- [ ] Parse `voryn://invite?from=...&t=...` → navigate to `InviteAcceptScreen` with params
- [ ] Add "Generate Invite Link" button to `ShareKeyScreen`
- [ ] Create `InviteAcceptScreen.tsx`
- [ ] On Accept: call `addContact(fromPubkey)` which triggers Task 7 request flow
- [ ] Store generated `t` tokens in AsyncStorage (`@voryn/invite_tokens`) for future revocation

---

### TASK 9 — Group chats *(deferred — separate session)*
**Rust code exists** in `voryn-protocol` (Shamir, group ledger, key management). UI and wire-up are the work.

#### Architecture (decided: shared group key via Rust Shamir)
- Group key generated by admin, distributed to members encrypted individually
- Members decrypt group key with their private key, store it
- Messages encrypted with group key, sent to all members via relay
- Key rotation on member removal

#### When we get here
- [ ] Design `Group` type: `{ groupId, name, members: pubkey[], adminPubkey, createdAt }`
- [ ] Group storage in AsyncStorage alongside contacts
- [ ] `ChatsScreen` shows group conversations in same list as 1-to-1
- [ ] Group creation screen (name + add members from contacts)
- [ ] Wire Rust group key generation + distribution to JS bridge
- [ ] Group message send/receive using group key
- [ ] Group chat UI (same as 1-to-1, shows member count in header)
- [ ] Member management: add/remove (admin only)

---

## Implementation Order

```
Bug fixes first, then features in dependency order:

1. TASK 1  Fix merge conflict          ~5 min
2. TASK 2  Fix encryption              ~3 hrs  ← CRITICAL
3. TASK 3  Fix double splash           ~30 min
4. TASK 4  Message checkmarks          ~2 hrs
5. TASK 5  Message delete              ~1 hr
6. TASK 6  Chats / Contacts split      ~3 hrs  ← do before Task 7 (nav depends on it)
7. TASK 7  Contact request approval    ~5 hrs
8. TASK 8  Invite links                ~2 hrs
9. TASK 9  Group chats                 deferred
```

**Estimated total (Tasks 1–8): ~17 hrs / 1–2 sessions**

---

## Progress

| Task | Status |
|------|--------|
| TASK 1 — Merge conflict | ⬜ Not started |
| TASK 2 — Fix encryption | ⬜ Not started |
| TASK 3 — Double splash | ⬜ Not started |
| TASK 4 — Checkmarks | ⬜ Not started |
| TASK 5 — Message delete | ⬜ Not started |
| TASK 6 — Chats / Contacts split | ⬜ Not started |
| TASK 7 — Contact request approval | ⬜ Not started |
| TASK 8 — Invite links | ⬜ Not started |
| TASK 9 — Group chats | 🔵 Deferred |
