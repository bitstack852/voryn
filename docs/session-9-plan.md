# Voryn — Session 9 Plan

**Last updated:** 2026-04-22  
**Branch:** `claude/assess-main-branch-gCIii`  
**Status:** Tasks 1–4 done — working on Task 5

---

## Current Snapshot

| Layer | State |
|-------|-------|
| iOS build | Builds and runs on iPhone-NST + Acumen-XR |
| Android build | APK builds — no device to test, shelved |
| Relay | Live at `ws://boot1.voryn.bitstack.website:4001/ws` |
| Messages | E2E encrypted — relay sees opaque ciphertext only |
| Rust bridge | Connected on iOS — Ed25519, encrypt/decrypt working |
| Screens | 10 screens implemented, dark theme, passcode lock |

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

### TASK 1 — Fix double splash screen ✅
**Files:** `apps/mobile/ios/Voryn/LaunchScreen.storyboard`

Native LaunchScreen replaced with plain `#050608` black frame — invisible transition into the custom RN SplashScreen.

- [x] Set LaunchScreen background to `#050608`, remove all labels/images

---

### TASK 2 — Fix encryption end-to-end ✅
**Files:** `ChatScreen.tsx`, `VorynBridge.ts`, `NetworkService.ts`, `App.tsx`

- [x] `sendMessage` encrypts via Rust, sends ciphertext hex over relay — no plaintext ever leaves device
- [x] `receiveMessage` decrypts before storing — discards on failure, no fallback
- [x] No-bridge path marks messages `failed`, does not send plaintext
- [x] One-time migration wipes legacy plaintext messages on first boot (`@voryn/migrated_v2` flag)
- [x] Relay ACK wired — updates message status to `delivered`
- [x] `updateMessageStatus(messageId, status)` added to `VorynBridge`

---

### TASK 3 — Message checkmarks ✅
**Files:** `ChatScreen.tsx`, `NetworkService.ts`

- [x] `StatusTick` component replaces Unicode emoji: dim ✓ pending, grey ✓ sent, grey ✓✓ delivered, red ✗ failed
- [x] `onAck` API added to `NetworkService` — fires with `message_id` from relay ACK
- [x] `ChatScreen` subscribes to `onAck`, updates status to `delivered` in storage

---

### TASK 4 — Message delete ✅
**Files:** `ChatScreen.tsx`, `VorynBridge.ts`

Local delete only — removes from this device's AsyncStorage. No "delete for everyone".

- [x] `deleteMessage(messageId)` added to `VorynBridge`
- [x] Long-press on any bubble → confirmation alert → delete
- [x] Works for both sent and received messages

---

### TASK 5 — Separate Chats tab and Contacts tab
**Files:** `RootNavigator.tsx`, `ContactsScreen.tsx`, new `ChatsScreen.tsx`, `VorynBridge.ts`

**Current:** Single `ContactsScreen` acts as both contact list and chat entry point.  
**Goal:** Bottom tab navigator with two distinct tabs.

#### Chats tab (new `ChatsScreen.tsx`)
- Conversations ordered by most recent message (newest first)
- Each row: avatar, display name, last message preview (~40 chars), timestamp
- Only contacts with at least one message exchanged
- Unread badge per conversation
- Tap → `ChatScreen`

#### Contacts tab (existing `ContactsScreen.tsx`, trimmed)
- Full contact list (address book)
- Tap → `ChatScreen`, long-press → `ContactDetail`
- Badge on tab icon when pending incoming contact requests exist (wired in Task 6)
- Settings via top-right header icon instead of inline bottom bar

#### Navigation change
```
Before: Stack → ContactsScreen (inline bottom bar)
After:  Stack → TabNavigator
                  ├── ChatsScreen   (tab 1)
                  └── ContactsScreen (tab 2)
```

#### Work
- [ ] Add `getConversations()` to `VorynBridge` — contacts with ≥1 message, sorted by last message timestamp, includes `lastMessageText` + `unreadCount`
- [ ] Add `markConversationRead(conversationId)` to `VorynBridge`
- [ ] Create `ChatsScreen.tsx`
- [ ] Add `createBottomTabNavigator` to `RootNavigator`
- [ ] Remove inline bottom bar from `ContactsScreen`
- [ ] Settings accessible from top-right header icon in `ContactsScreen`
- [ ] Unread badge on Chats tab icon
- [ ] Pending requests badge placeholder on Contacts tab icon (filled in Task 6)

---

### TASK 6 — Contact request approval
**Files:** `VorynBridge.ts`, `NetworkService.ts`, `AddContactScreen.tsx`, new `PendingRequestsScreen.tsx`, `ContactsScreen.tsx`

#### Flow
1. Alice enters Bob's key → taps **Send Request**
2. App stores contact as `pending_sent`, sends encrypted `contact_request` over relay
3. Request queued client-side with sent/delivered status — retried on reconnect until ACKed
4. Bob receives → stored as `pending_received`, badge on Contacts tab
5. Bob opens Pending Requests → Approve or Deny
6. Approve → Bob stores Alice as `approved`, sends encrypted `contact_accepted` to Alice
7. Deny → sends `contact_denied`, removes pending entry (not a permanent block)
8. Alice receives `contact_accepted` → marks `approved`; `contact_denied` → dismissible notice

#### Contact status field
```ts
status: 'approved' | 'pending_sent' | 'pending_received' | 'denied'
```

#### Protocol message types (all encrypted)
```json
{ "type": "contact_request", "from": "<pubkey>", "display_name": "...", "intro": "..." }
{ "type": "contact_accepted", "from": "<pubkey>" }
{ "type": "contact_denied", "from": "<pubkey>" }
```

#### Work
- [ ] Add `status` field to `Contact` type
- [ ] `addContact` creates `pending_sent` entry + sends encrypted `contact_request` via relay
- [ ] Pending outbox queue in `VorynBridge` — retry on relay reconnect
- [ ] `NetworkService` detects `contact_request` / `contact_accepted` / `contact_denied` type after decrypt, routes to separate handler
- [ ] Add `onContactRequest(handler)` API to `NetworkService`
- [ ] `contact_request` received → store `pending_received`
- [ ] `contact_accepted` received → update to `approved`
- [ ] `contact_denied` received → mark `denied`, show dismissible notice
- [ ] Create `PendingRequestsScreen.tsx` — list with Approve / Deny buttons
- [ ] `AddContactScreen` — rename to "Send Request", show "Request sent" confirmation
- [ ] Wire pending requests badge on Contacts tab icon
- [ ] `ContactsScreen` only shows `approved` contacts

---

### TASK 7 — Invite links
**Files:** `ShareKeyScreen.tsx`, `RootNavigator.tsx`, `App.tsx`, `Info.plist`, new `InviteAcceptScreen.tsx`

#### Link format
```
voryn://invite?from=<sender_pubkey_hex>&t=<random_8byte_hex>
```

#### Flow
1. Alice: Share Key screen → "Generate Invite Link" → share sheet
2. Bob taps link → app opens → `InviteAcceptScreen`
3. Screen shows inviter's key (first 12 chars) + Accept / Decline
4. Accept → Bob sends `contact_request` to Alice (Task 6 flow)
5. Decline → dismiss, no action

#### Work
- [ ] Register `voryn://` URL scheme in `Info.plist`
- [ ] Handle `Linking.getInitialURL()` + `Linking.addEventListener` in `App.tsx`
- [ ] Parse `voryn://invite?from=...&t=...` → navigate to `InviteAcceptScreen`
- [ ] Add "Generate Invite Link" button to `ShareKeyScreen`
- [ ] Create `InviteAcceptScreen.tsx`
- [ ] Accept triggers `addContact(fromPubkey)` → Task 6 request flow
- [ ] Store generated tokens in `@voryn/invite_tokens` for future revocation

---

### TASK 8 — Group chats *(deferred — separate session)*

Rust code exists in `voryn-protocol` (Shamir, group ledger, key management). UI and wire-up are the work. Encryption uses shared group key via Rust Shamir (decided).

#### When we get here
- [ ] `Group` type: `{ groupId, name, members: pubkey[], adminPubkey, createdAt }`
- [ ] Group storage in AsyncStorage
- [ ] `ChatsScreen` shows group conversations alongside 1-to-1
- [ ] Group creation screen (name + add members from contacts)
- [ ] Wire Rust group key generation + distribution to JS bridge
- [ ] Group message send/receive via group key
- [ ] Group chat UI — same as 1-to-1, member count in header
- [ ] Member add/remove (admin only)

---

## Progress

| Task | Status |
|------|--------|
| TASK 1 — Double splash | ✅ Done |
| TASK 2 — Fix encryption | ✅ Done |
| TASK 3 — Message checkmarks | ✅ Done |
| TASK 4 — Message delete | ✅ Done |
| TASK 5 — Chats / Contacts split | ✅ Done |
| TASK 6 — Contact request approval | ✅ Done |
| TASK 7 — Invite links | ✅ Done |
| TASK 8 — Group chats | 🔵 Deferred |
