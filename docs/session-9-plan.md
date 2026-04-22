# Session 9 Plan

Last updated: 2026-04-22

## Objective

Fix broken encryption, clean up UX issues, and add key new features (checkmarks,
message delete, chat/contact split, contact requests, invite links, group chats).
No coding starts until this plan is signed off.

---

## Current State Assessment

### What's Working
- Both iPhones connect to relay at `boot1.voryn.bitstack.website:4001`
- Messages arrive on both devices
- Rust crypto bridge is live on iOS (Ed25519, encrypt/decrypt functions)
- 10 screens, dark theme, passcode lock all functional

### Critical Bug: Messages Are NOT Encrypted

`ChatScreen.handleSend` has two send calls:

```ts
// Call 1 — encrypts via Rust, but sends via dead libp2p path (not connected)
const messageId = await VorynBridge.sendMessage(contactPubkeyHex, text);

// Call 2 — sends RAW PLAINTEXT over the WebSocket relay
NetworkService.sendToPeer(contactPubkeyHex, text, messageId);
```

`VorynBridge.sendMessage` does encrypt but then calls `sendRawToPeer` which uses the old
libp2p node (removed in Session 8). `NetworkService.sendToPeer` sends the unencrypted
plaintext string directly. The relay is routing plaintext.

There is also an **unresolved merge conflict** in `ChatScreen.tsx` lines 17–20
(duplicate `NetworkService` import left over from a branch merge).

### Also Not Encrypted: Incoming Path
`NetworkService.storeIncoming` calls `VorynBridge.receiveMessage(from, payload, msgId)`
passing the raw WebSocket payload as `plaintext`. No decryption step exists.

---

## Tasks (Checklist Order)

Tasks are ordered: critical bugs first, then UX polish, then new features.

---

### TASK 1 — Fix merge conflict in ChatScreen.tsx
- [ ] Remove duplicate `import * as NetworkService` conflict markers (lines 17–20)
- [ ] Verify file compiles clean after fix

**Why:** The file has `<<<<<<< HEAD` / `=======` / `>>>>>>>` markers. It won't
build cleanly and obscures intent.

---

### TASK 2 — Fix Encryption End-to-End
**Goal:** All messages sent over the relay must be encrypted. Relay sees only
opaque hex blobs. Sender's plaintext never leaves the device unencrypted.

#### Send path fix
1. In `ChatScreen.handleSend`, remove the second `NetworkService.sendToPeer(contactPubkeyHex, text, ...)` call
2. In `VorynBridge.sendMessage`, replace `sendRawToPeer` (libp2p, dead) with
   `NetworkService.sendToPeer(recipientPubkeyHex, encrypted.envelopeHex, messageId)`
3. If Rust bridge unavailable (no encryption), **do not send** — mark status `failed`
   with reason "Encryption unavailable". Do not silently send plaintext as fallback.

#### Receive path fix
1. In `NetworkService.storeIncoming`, before calling `VorynBridge.receiveMessage`,
   attempt to decrypt via `VorynBridge.decryptMessage(payload, identity.secretKeySeedHex)`
2. If decryption succeeds, store the plaintext
3. If decryption fails (legacy plaintext message from before this fix), store with
   a `[unencrypted]` prefix or discard — decision needed (see open question below)

#### Open question for sign-off
> Existing messages stored in AsyncStorage on both devices are plaintext (stored as-is).
> Do we keep them as-is (they'll display fine since they're already decoded),
> or wipe message history clean on this session's build?
> **Recommendation: wipe on both devices during this session's install since
> the devices are test-only and we want a clean encrypted baseline.**

#### Verification checklist
- [ ] Rust bridge `encryptMessage` / `decryptMessage` called in send/receive path
- [ ] `NetworkService.sendToPeer` receives hex-encoded ciphertext, not plaintext
- [ ] Message arrives on other device decrypted correctly
- [ ] Relay `/status` shows peer count but cannot read message content
- [ ] Message marked `failed` (not silently sent as plaintext) if bridge unavailable

---

### TASK 3 — Fix Double Splash Screen
**What's happening:** iOS shows the native LaunchScreen (storyboard) first, then
React Native boots and immediately renders the custom `SplashScreen` component.
User sees two splash screens back to back.

**Fix options:**
- Option A (simple): Remove or make the native LaunchScreen instant (blank/black frame
  only) so only the custom RN splash is visible. On iOS, set the LaunchScreen to a
  plain black background matching the app — the native flash becomes invisible.
- Option B (proper): Use `react-native-bootsplash` to hold the native splash until
  RN is ready, then hand off directly. More code, cleaner result.

**Recommendation: Option A** — edit `LaunchScreen.storyboard` to be a solid black
background with no logo/text. The RN `SplashScreen` handles the branded experience.

- [ ] Edit `apps/mobile/ios/Voryn/LaunchScreen.storyboard` background to `#050608` (match app bg)
- [ ] Remove any image/label from the storyboard
- [ ] Build and verify only one splash is visible on device

---

### TASK 4 — Message Checkmarks (styled)
**Current state:** `statusIcon` returns `⏳` `✓` `✓✓` as plain Unicode text.

**Goal:** Signal/WhatsApp-style tick icons, styled properly.

| Status | Display | Meaning |
|--------|---------|---------|
| `pending` | clock icon or dim single tick | Composing / not yet sent to relay |
| `sent` | single grey tick | Relay received it |
| `delivered` | double grey tick | Recipient's device confirmed receipt |
| `read` | double blue tick | Recipient opened the conversation *(phase 2)* |

**Implementation:**
- Replace Unicode emoji with styled `Text` or small SVG/image ticks
- Ticks sit bottom-right of bubble, consistent sizing (11–12pt)
- `pending`: dim single tick or hourglass
- `sent`: `✓` grey
- `delivered`: `✓✓` grey
- Keep `failed` state visible (red `!` or red `✗`) with retry tap

**Delivery confirmation (relay ACK):**
The relay already sends `{ type: 'ack' }` back to sender. `NetworkService.handleServerMessage`
currently ignores it. Wire the ACK to update message status from `sent` → `delivered`.
The relay ACK includes `message_id` — use that to find and update the stored message.

- [ ] Style tick icons in `ChatScreen` (replace emoji with styled component)
- [ ] Handle relay `ack` in `NetworkService` — fire a callback with the `message_id`
- [ ] Add `onAck` listener API to `NetworkService`
- [ ] `ChatScreen` subscribes to `onAck`, calls `VorynBridge.updateMessageStatus(messageId, 'delivered')`
- [ ] Add `updateMessageStatus` to `VorynBridge`
- [ ] `failed` messages show red indicator and are tappable to retry

---

### TASK 5 — Message Delete
**Goal:** Long-press a message bubble → action sheet → Delete option.

**Scope:**
- Delete is local only (removes from this device's AsyncStorage)
- No protocol-level "delete for everyone" in this phase
- Delete applies to both sent and received messages

**Implementation:**
- Long-press on message bubble → `Alert.alert` with "Delete Message" / "Cancel"
- `VorynBridge.deleteMessage(messageId)` removes from stored messages
- UI refreshes immediately

- [ ] Add `deleteMessage(messageId)` to `VorynBridge`
- [ ] Long-press handler on message bubble in `ChatScreen`
- [ ] Confirmation alert before delete
- [ ] Message list refreshes after delete

---

### TASK 6 — Separate Contacts Tab and Chats Tab
**Current state:** `ContactsScreen` is both the contact list and the entry point to chats.
Tapping a contact opens chat. There is no "recent conversations" view.

**Goal:** Two distinct tabs at the bottom of the main screen.

#### Chats tab (new)
- Lists **active conversations** (contacts you've sent or received at least one message with)
- Ordered by most recent message timestamp (newest first)
- Each row shows: avatar, display name, last message preview (truncated), timestamp
- Unread badge count on conversations with unread messages
- Tapping opens `ChatScreen`

#### Contacts tab (existing, refined)
- Lists **all contacts** (the address book)
- Tapping a contact opens chat (same as today)
- Long-press opens `ContactDetail`
- New button: **Pending Requests** badge if there are unapproved contact requests (Task 7)

#### Navigation
- Bottom tab navigator with two tabs: **Chats** and **Contacts**
- Replace the current `ContactsScreen` bottom bar (Share Key / Scan QR / Settings)
  with proper tab navigation
- Settings accessible from within Contacts tab (top-right icon or list item)

**Data changes needed in `VorynBridge`:**
- `getConversations()` — returns list of contacts who have at least one stored message,
  with last message text, timestamp, and unread count
- `getUnreadCount(conversationId)` — count of messages where `isMine === false`
  and `status !== 'read'` (use `delivered` as proxy for now)

- [ ] Create `ChatsScreen.tsx` with conversation list
- [ ] Add `getConversations()` to `VorynBridge`
- [ ] Replace bottom bar in main screens with bottom tab navigator (React Navigation Tab)
- [ ] Update `RootNavigator` with tab structure
- [ ] Contacts tab retains existing contact list behaviour
- [ ] Chats tab shows last message + timestamp per contact
- [ ] Unread badge on Chats tab icon

---

### TASK 7 — Contact Request Approval
**Goal:** When Alice adds Bob's key, Bob receives a request and must approve before
they can exchange messages.

#### Flow
1. Alice types Bob's public key → taps "Send Request"
2. App sends a `contact_request` message over relay to Bob's pubkey, containing:
   - Alice's public key
   - Alice's chosen display name for herself (optional)
   - A short intro message (optional, max 200 chars)
3. Bob's device receives the request, shows a notification/badge on the Contacts tab
4. Bob navigates to "Pending Requests" section
5. Bob can: **Approve** (adds Alice to contacts, sends `contact_accepted` back to Alice)
   or **Deny** (sends `contact_denied`, Alice sees declined status)
6. Alice's side: pending contact shows as "Request sent — awaiting approval"

#### Data model additions
- `Contact.status: 'approved' | 'pending_sent' | 'pending_received' | 'denied'`
- `pendingRequests` stored separately or filtered by status from contacts list

#### Protocol additions (relay message types)
- `contact_request` — `{ type, from, display_name?, intro_message? }` (encrypted with recipient's pubkey)
- `contact_accepted` — `{ type, from }` (encrypted)
- `contact_denied` — `{ type, from }` (encrypted)

These are encrypted protocol messages, not chat messages — they go over the same relay
but use a distinct `type` field before the inner payload is decrypted.

**Open question for sign-off:**
> Should denied contacts be permanently blocked (never receive requests from them again)
> or just dismissed (they can request again)?
> **Recommendation: dismissed only for now. Blocking is a separate feature.**

- [ ] Update `Contact` type with `status` field
- [ ] Update `addContact` to create a `pending_sent` contact and send request over relay
- [ ] `NetworkService` routes `contact_request` payloads to a new handler (not `onMessage`)
- [ ] Add `PendingRequestsScreen.tsx` (or section within Contacts tab)
- [ ] Approve/Deny buttons on each pending request
- [ ] Send `contact_accepted` / `contact_denied` encrypted relay message on decision
- [ ] Handle incoming `contact_accepted` — mark contact as `approved`
- [ ] Badge on Contacts tab for pending incoming requests
- [ ] `AddContactScreen` updated to show "Request sent" confirmation, not immediate add

---

### TASK 8 — Invite Links
**Goal:** Alice generates a link she can share anywhere (iMessage, email, etc.).
When Bob taps the link, the app opens and walks Bob through adding Alice.

#### Link format
```
voryn://invite?from=<alice_pubkey_hex>&t=<random_8byte_token_hex>
```

- `from` — Alice's full public key hex (64 chars)
- `t` — random 8-byte token (prevents scanning for pubkeys, makes links one-use capable)

#### Flow
1. Alice opens **Share Key** screen → "Generate Invite Link" button
2. App generates `t` token, stores `{ token, created_at }` locally
3. Link is formatted and shared via iOS share sheet
4. Bob taps link → app opens (or installs first) → deep link fires
5. App shows "You've been invited by [first 8 chars of Alice's key]" with Accept/Decline
6. Accept → triggers Task 7 contact request flow (Bob sends `contact_request` to Alice)
7. Alice receives the contact request and approves Bob (Task 7 approval flow)

#### Technical
- Register URL scheme `voryn://` in `Info.plist` (iOS) and `AndroidManifest.xml`
- Handle `Linking.getInitialURL()` + `Linking.addEventListener('url', ...)` in `App.tsx`
- Parse `voryn://invite?from=...&t=...` and navigate to an `InviteAcceptScreen`
- The `t` token is informational for now (rate limiting / revocation deferred)

- [ ] Register `voryn://` URL scheme in iOS `Info.plist`
- [ ] Add invite link generation to `ShareKeyScreen` (or new `InviteScreen`)
- [ ] Handle deep link in `App.tsx` / `RootNavigator`
- [ ] Create `InviteAcceptScreen.tsx` — show inviter's key, Accept / Decline
- [ ] Accept triggers contact request to inviter (Task 7 flow)
- [ ] Store generated tokens locally for future revocation capability

---

### TASK 9 — Group Chats
**Goal:** Create a group, add members, send messages to the group.

**Note:** This is the largest task. Rust code exists (`voryn-protocol` group module with
Shamir's Secret Sharing, group ledger, key management). The UI and wire-up are the work.

**This task is deferred to after Tasks 1–8 are complete.** It is listed here for
planning completeness so architecture decisions in Tasks 6–7 don't block it later.

#### What group chats require
- A `Group` data model: `{ groupId, name, members: string[], adminPubkey, createdAt }`
- Groups stored in AsyncStorage alongside contacts
- `ChatsScreen` shows both 1-to-1 and group conversations in the same list
- Group messages relayed: sender encrypts for each member individually (or use group key)
- Group creation flow: pick name, add members from contacts, send `group_invite` to each
- Member addition/removal by admin
- Group chat UI identical to 1-to-1 chat but shows member avatars

#### Architecture decision (sign-off needed)
> Two encryption options for groups:
> A) **Per-member encryption** — sender encrypts the message once per member (same as 1-to-1 DH).
>    Simple, uses existing `encryptMessage`. Scales poorly for large groups (N encryptions per message).
> B) **Shared group key** — single symmetric key shared via X3DH / Shamir's, sender encrypts once,
>    relay broadcasts to all members. Uses the existing Rust group key management code.
>    More complex but correct architecture for groups.
> **Recommendation: Option B using Rust group key infrastructure — it's already built.**

- [ ] Design group data model and storage schema
- [ ] Group creation screen (Task 9a)
- [ ] Group invite protocol messages (Task 9b)
- [ ] Wire Rust group key management to JS (Task 9c)
- [ ] Group message send/receive path (Task 9d)
- [ ] Group chat UI (Task 9e)
- [ ] Group member management (add/remove) (Task 9f)

---

## Implementation Order

```
1. TASK 1 — Fix merge conflict          (5 min)
2. TASK 2 — Fix encryption              (2–3 hrs) ← CRITICAL, do first
3. TASK 3 — Fix double splash           (30 min)
4. TASK 4 — Message checkmarks          (2 hrs)
5. TASK 5 — Message delete              (1 hr)
6. TASK 6 — Chats / Contacts split      (3–4 hrs) ← structural, do before requests
7. TASK 7 — Contact request approval    (4–5 hrs)
8. TASK 8 — Invite links                (2–3 hrs)
9. TASK 9 — Group chats                 (deferred — separate session)
```

**Estimated total (Tasks 1–8): 1–2 focused sessions**

---

## Decisions (Signed Off)

1. **Existing message history** — ✅ WIPE on this session's install (clean encrypted baseline)

2. **Denied contact requests** — ✅ DISMISSED only (no permanent block; they can request again)

3. **Group encryption** — ✅ SHARED GROUP KEY using existing Rust Shamir/group-key infrastructure

4. **Read receipts** — ✅ BOTH: `sent` (single grey tick) + `delivered` (double grey tick). No blue read ticks yet.

5. **Offline contact requests** — ✅ CLIENT-SIDE QUEUE: store the request on the sender's device with `sent`/`delivered` status, same as messages. Retry sending on reconnect until the relay ACKs delivery. No relay store-and-forward needed.

---

## Files That Will Change

| File | Tasks |
|------|-------|
| `apps/mobile/src/screens/ChatScreen.tsx` | 1, 2, 4, 5 |
| `apps/mobile/src/services/VorynBridge.ts` | 2, 4, 5, 6, 7 |
| `apps/mobile/src/services/NetworkService.ts` | 2, 4, 7, 8 |
| `apps/mobile/src/navigation/RootNavigator.tsx` | 6, 8 |
| `apps/mobile/src/screens/ContactsScreen.tsx` | 6, 7 |
| `apps/mobile/ios/Voryn/LaunchScreen.storyboard` | 3 |
| `apps/mobile/src/screens/ShareKeyScreen.tsx` | 8 |
| `apps/mobile/src/screens/AddContactScreen.tsx` | 7 |
| NEW: `apps/mobile/src/screens/ChatsScreen.tsx` | 6 |
| NEW: `apps/mobile/src/screens/PendingRequestsScreen.tsx` | 7 |
| NEW: `apps/mobile/src/screens/InviteAcceptScreen.tsx` | 8 |
