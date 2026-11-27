---
name: diverge-access
description: Who may do what on a Diverge daemon. Covers accounts (an (issuer, subject) identity the daemon admits), the daemon's administrators, roles and grants (every grant kind, every action, the make / over / tagging shapes, `within` and tags), creators, requests (a question put to accounts and tools, and the one answer it gets), the host's identities as the daemon lists them, and how to diagnose a `forbidden` answer or a connection the daemon closes. Load it before running any `diverge run accounts …`, `diverge run roles …`, `diverge run requests …` or `diverge run identities …` command, when creating an account or a role for a person, an agent host or another daemon, when writing or narrowing a grant, when tagging records so grants can reach them, when disconnecting an account, when asking an account or a tool a question and waiting for its answer, when listening for the questions asked of this account, or whenever a Diverge request answers `forbidden`.
---

# Diverge access: accounts, roles, grants

A Diverge daemon serves every request for exactly one requester, and decides it by that requester's grants.
A requester is an **account** (an identity the daemon admits, holding roles), an **administrator** (an
identity among the daemon's `admins`, holding every grant), or a tool's container (holding the grants its program
declared). A **role** is a named list of **grants**; an account may do what any grant of any role it holds
allows. Nothing denies: the union is the whole rule.

All commands here are `diverge run <family> <request> '<JSON body>'`. Each prints one compact JSON line per
answer. Run `<command> request-schema` or `<command> response-schema` for exact shapes (they dial nothing).
Every body may also carry `relay` (see the `diverge-daemons` skill).

## The model in six rules

1. **An identity is `(issuer, subject)`**, written `{"issuer":"https://identity.diverge.network","subject":"ronald.alice"}`.
   It is what a client's access token establishes in the handshake, once the client has proven it holds
   the key that token is bound to. The daemon keeps no list of
   trusted issuers: an identity from any issuer connects once a record (account, administrator, provider) names it.
2. **An account is one identity the daemon admits**, with a description, roles and tags. One account per
   identity; the identity is the account's name and never changes. An identity that is no account, no
   administrator and no provider is refused in the handshake: the connection closes with no frame.
3. **Administrators** are the identities in the daemon's `admins` setting (`diverge daemon admins get`)
   **that have no stored account**. An administrator is an account in every respect except that it is not
   stored: it is admitted wherever an account is (directly, and when another daemon connects as it), it is
   listed, tagged, disconnected and sent requests like one, and every judgment allows it everything without
   reading a grant. A fresh daemon has no account, so its first accounts are made by an administrator.
   `admins` is read once at the daemon's start (see `diverge-config` to change it).
   - **A stored account wins.** One identity is one account. If an identity in `admins` also has a stored
     account, it is that account and nothing more: it holds only what its roles grant, and the `admins`
     entry does nothing until the stored account is deleted.
   - An administrator cannot be created as an account (`accounts create` answers `"exists"`), and cannot be
     edited or deleted (`"administrator"`). To stop it being an administrator, take it out of `admins`.
4. **Admin status is local to one daemon.** Being an administrator of daemon A gives nothing on daemon B.
   A request relayed to B is served as whatever the presented identity is **on B**: a stored account there,
   or one of B's own administrators.
5. **A role holds grants; an account holds roles.** A role holds no roles. An account with no role may do
   nothing (it can still connect, and `info` needs no grant). A role's edit takes effect at each holder's
   next request.
6. **Every request is judged fresh**: the account's roles are read inside the request, so a role edited or
   an account's roles changed applies at once. Editing or deleting an account also closes its connections.

## Grants: the four shapes

A grant is one JSON object with one member, named for the kind: `{"agents": …}`. The value takes one of
these shapes, told apart by form. A value that is none of them does not decode (the request fails to
parse; it is not `forbidden`).

| Shape | Form | Meaning |
|---|---|---|
| **Make** | `{"agents":["create"]}` | A bare array of the kind's making actions. Holding it is the whole test. |
| **Over** | `{"agents":{"actions":["get","message"],"within":"any"}}` | Actions on things that exist, and how far they reach. |
| **Tagging** | `{"agents":{"actions":["tag","untag"],"within":"any","tags":["crew"]}}` | `tag`/`untag`, how far, and which tags (`"any"` or a list). |
| **The one thing** | `{"events":["list"]}` | For `postgres` and `events`: a bare array; nothing to narrow. |
| **Through a provider** | `{"images":{"actions":["build","push","list"],"within":{"identities":[…]}}}` | For `providers`, `providers_daemons`, `images`: `within` is `"any"` or a provider reach, by identity, by the provider record's tags, or both. `images` has no bare-array form. |

Mixing shapes is malformed: `tag` among `actions` without `tags`, or `create` inside an Over object, does
not decode. Write a making grant and an over grant as two separate grants.

### `within`

**One grant reaches all, exactly the records it names, or by tags. Never two of them.**

| Reach | `within` | Reaches |
|---|---|---|
| All | `"any"` | every record of the kind |
| Specific records | the kind's naming member, e.g. `{"ids":[…]}`, `{"volumes":[…]}` | exactly those records |
| Tags | `{"all_tags":[…]}`, `{"any_tags":[…]}` or both | records carrying every one of `all_tags` and any one of `any_tags` |

- **Naming records and tags in one grant is refused**, and so is naming records by two different members. The
  role create or edit, or the tool's registration, fails to decode.
- **"These records, or anything tagged so" is two grants.** Grants only add: a request is allowed when any one
  grant allows it, and no grant takes away what another gives.
- **In a role's grant, the other filter members narrow.** `creators`, `templates`, `active`, `deployed`, dates
  and the rest may stand beside either form, or alone, and all of them must hold. "Tagged `crew` and made from
  template X" is one grant: `{"all_tags":["crew"],"templates":["<id>"]}`.

```json
{"agents":{"actions":["get","list","message","logs"],"within":{"all_tags":["crew"]}}}
{"agents":{"actions":["message"],"within":{"agents":[{"template":"<agent template id>","index":3}]}}}
{"agents":{"actions":["delete"],"within":{"creators":[{"type":"client","identity":{"issuer":"https://identity.diverge.network","subject":"ronald.alice"}}]}}}
{"volumes":{"actions":["get","stat"],"within":{"volumes":[{"provider":{"issuer":"…","subject":"ronald.provider"},"name":"datasets"}]}}}
```

Which member names records differs per kind; the table and the narrowing members per kind are in
[references/grants.md](references/grants.md). For `providers`, `providers_daemons` and `images`, `within` is
`"any"`, `{"identities":[…]}`, or tags, and nothing narrows.

### The kinds

`agents`, `agents_templates`, `standalone_tools`, `standalone_tools_templates`, `connected_standalone_tools`,
`dependency_tools`, `providers`, `providers_daemons`, `daemons`, `accounts`, `roles`, `volumes`,
`postgres`, `postgres_compartments`, `events`, `images`, `identities`. There is no separate `logs`, `frontend`, `exec` or `terminal`
kind: those are *actions* inside the kinds that have them (`logs` over agents, tools and accounts;
`frontend` and `exec` over agents, standalone tools and dependency tools; a terminal is judged as `exec`).

**Every action of every kind, and which command needs it, is in
[references/grants.md](references/grants.md).** Read it before writing a role.

### Requests that need more than one grant

A request naming several things is allowed only when every one is:

- `accounts create` / `accounts edit` naming roles: `create` (or `edit`) over accounts **and** `grant` over
  each role named. An edit that changes the roles needs `grant` over every role in the resulting list, the
  ones kept included; an edit that leaves the roles as they were needs none.
- Attaching or detaching a tool and one agent: `attach`/`detach` over the tool **and** `edit` over the agent.
  With an agent template: `attach`/`detach` over the tool **and** `attach` over the template.
- `connected-standalone-tools create`: `create` over connected tools **and** `relay` over the remote
  daemon's record.
- Mounting a volume in a container's create or edit: the container's grant **and** `mount` over the volume.
  A FUSE mount of a volume on **another daemon** (the mount names `daemon`): the container's grant **and**
  `relay` over that daemon's record, and nothing over the volume here; the other daemon judges its own
  `fuse_serve` grant for the account the record names.
- Seeing a template's arguments (`"arguments": true` on `agents templates get|list`,
  `standalone-tools templates get|list`, `dependency-tools get|list`): the `get` or `list` grant **and**
  `arguments` over the same template. Arguments carry credentials, so give `arguments` narrowly.
- A transfer: `transfer` over the source **and** `upload` over where it lands.
- Any request with `relay`: `relay` over the daemons record that is the first ID, on this daemon; the
  request itself is then judged on the remote daemon, as the account the record names.

### Lists

A `list` is judged record by record: it sends what the account's `list` grants reach (then narrowed by the
request's own filter) and nothing else. It is `forbidden` only when no grant holds `list` over the kind at
all. A caller whose grants reach nothing gets `"listed"` with no items, not `forbidden`.

One exception: a list whose body says `"arguments": true` is `forbidden` unless `arguments` grants reach
**every** template the list would show. Kept open with `--watch`, it does not show a later template outside
that reach. With `"arguments": false` the list is judged as any other and no item carries `arguments`.

## Tags, and what a creator is

- **Tags** are plain strings on a record, put on with the family's `tag` request and taken off with `untag`.
  They exist so grants (and list filters) can reach groups of records without naming them. Agents, both
  kinds of template, standalone / connected / dependency tools, providers, remote daemon records, accounts,
  roles, volumes and the host's identities all carry tags.
- A tagging grant limits *which* tags: `{"roles":{"actions":["tag"],"within":"any","tags":["team-a"]}}`
  lets the holder put only `team-a` on roles. Putting several tags in one request needs one grant that
  covers all of them.
- Being able to tag is being able to move a record into another grant's reach. Hand out tagging grants
  with a narrow `tags` list.
- **A creator** is who made a record, carried on every list item as `creator`:
  `{"type":"client","identity":{…}}` for an account (or administrator) over an endpoint,
  `{"type":"agent","template":…,"index":…,"name":…}` for an agent, or `{"type":"tool",…}` for a tool that
  made it through the daemon. Only the direct maker is carried. Records are not owned: an account sees and
  acts on exactly what its grants reach, whoever made it. To give an account "its own things only", use
  `within: {"creators":[{"type":"client","identity":<that account>}]}`.

## Roles

```bash
# Create. `grants` may be empty (a role that allows nothing).
diverge run roles create '{"name":"operator","description":"runs the crew agents","grants":[
  {"agents":["create"]},
  {"agents":{"actions":["get","list","message","logs","edit","delete"],"within":{"all_tags":["crew"]}}},
  {"agents":{"actions":["tag","untag"],"within":"any","tags":["crew"]}},
  {"agents_templates":{"actions":["get","list"],"within":"any"}},
  {"events":["list"]},
  {"providers":{"actions":["read"],"within":"any"}}]}'
# -> "created" | "exists" | "forbidden" | {"error":…}

diverge run roles get  '{"name":"operator"}'          # -> {"found":{name, description, grants, accounts, tags, created, creator}}
diverge run roles list                                 # -> {"added":{…}} per role, then "listed"
diverge run roles list '{"accounts":[{"issuer":"https://identity.diverge.network","subject":"ronald.alice"}]}'   # roles alice holds
diverge run roles tag   '{"name":"operator","tags":["team-a"]}'    # -> "tagged"
diverge run roles untag '{"name":"operator","tags":["team-a"]}'    # -> "untagged"
diverge run roles delete '{"name":"operator"}'         # -> "deleted" | "not_found" | "in_use" | "forbidden"
```

`roles edit` takes a JSON Patch (RFC 6902) applied to the document `{"description":…,"grants":[…]}`, whole
or not at all. The name is not editable (a new name is a new role).

```bash
# Append one grant
diverge run roles edit '{"name":"operator","patch":[
  {"op":"add","path":"/grants/-","value":{"volumes":{"actions":["get","list"],"within":"any"}}}]}'
# Replace all grants
diverge run roles edit '{"name":"operator","patch":[{"op":"replace","path":"/grants","value":[{"events":["list"]}]}]}'
# Remove the third grant, only if it is what you think it is
diverge run roles edit '{"name":"operator","patch":[
  {"op":"test","path":"/grants/2","value":{"agents":{"actions":["tag","untag"],"within":"any","tags":["crew"]}}},
  {"op":"remove","path":"/grants/2"}]}'
# -> "edited" | "not_found" | "forbidden" | {"error":…} (a patch that does not apply, or a result that is not a role)
```

Rules: `roles delete` answers `in_use` while any account holds the role (take it off the accounts first).
The daemon stores grants exactly as given and checks nothing beyond decoding them, so a grant naming a tag
or a name nothing has is accepted and simply reaches nothing. List filter members for roles: `names`,
`accounts`, `creators`, `all_tags`, `any_tags`, `created_from`, `created_to`, plus `count`. `roles list`
takes `--watch`.

## Accounts

```bash
# Create: the identity is admitted from now on. `roles` may be empty.
diverge run accounts create '{"identity":{"issuer":"https://identity.diverge.network","subject":"ronald.alice"},
  "description":"crew operator","roles":["operator"]}'
# -> "created" | "exists" | "no_role" | "forbidden" | {"error":…}

diverge run accounts get '{"account":{"issuer":"https://identity.diverge.network","subject":"ronald.alice"}}'
# -> {"found":{"identity":{…},"description":…,"roles":[…],"connected":false,"last_connected":…,"tags":[…],
#              "created":…,"creator":{…}}}   (an administrator also carries "admin":true)

diverge run accounts list                                  # every account the grants reach, then the admins
diverge run accounts list '{"roles":["operator"],"connected":true}'

# Edit: a JSON Patch over {"description":…,"roles":[…]}. The identity is not editable.
diverge run accounts edit '{"account":{"issuer":"https://identity.diverge.network","subject":"ronald.alice"},
  "patch":[{"op":"add","path":"/roles/-","value":"auditor"}]}'
# -> "edited" | "not_found" | "no_role" | "forbidden" | {"error":…}

diverge run accounts tag   '{"account":{"issuer":"…","subject":"ronald.alice"},"tags":["team-a"]}'   # -> "tagged"
diverge run accounts untag '{"account":{"issuer":"…","subject":"ronald.alice"},"tags":["team-a"]}'   # -> "untagged"

diverge run accounts disconnect '{"account":{"issuer":"…","subject":"ronald.alice"}}'   # -> {"disconnected":2}
diverge run accounts delete     '{"account":{"issuer":"…","subject":"ronald.alice"}}'   # -> "deleted" | "not_found" | "forbidden"
```

What to know:

- **Creating an account does not create the identity.** The subject must exist at its issuer and sign in
  there to get a token (`diverge-config`: `diverge identities login`).
  The issuer string must match the token's `iss` exactly. The daemon needs no entry for that
  issuer of the daemon's own identity, and need not be signed in there: the account record alone admits the identity.
- To use the new account from the CLI, add it to that daemon's accounts
  (`diverge cli daemons <INDEX> accounts add …`, skill `diverge-config`) and pass `--account <name>`.
- `no_role` (a role the daemon does not have) is answered before `forbidden` (no `grant` over a role named).
- **`accounts edit` closes every connection the account holds**, so it reconnects as it now stands;
  `accounts delete` closes them too. What a deleted account created stays, keeping its creator.
- **`accounts disconnect`** closes every connection a client has as the account (direct ones, and one
  another daemon made as it through a provider), answers the count (`0` if none) without waiting, and
  changes nothing else: the account may reconnect at once. An account may disconnect itself. It also works
  on an administrator. Use it to force a reconnect or to cut a session off; to keep
  someone out, delete the account or take its roles away.
- **Administrators in lists.** `accounts list` and `get` show each administrator with `"admin":true`, its
  tags, and no roles, `created` or `creator`. `accounts tag` and `untag` work on it as on any account.
  `edit` and `delete` answer `"administrator"`: it is not stored, so there is nothing to change or remove.
  A grant's `within` reaches an administrator through `identities`, `connected` and its tags, never through
  a filter on roles, creators or times. An identity in `admins` that has a stored account shows as that
  account, with `"admin":false`.
- List filter members for accounts: `identities`, `roles`, `connected`, `creators`, `all_tags`, `any_tags`,
  `created_from`, `created_to`, plus `count`. `accounts list` takes `--watch` (an account's `connected`
  flipping arrives as `changed`).

## Requests (`diverge run requests …`)

A request is a question one party puts to others, and the one answer it gets. The question and the answer
are content blocks, exactly as a chat message's `content` is (`[{"type":"text","text":…}]`): conversation,
not data. The daemon reads neither.

```bash
# Ask. Prints `created`, a `delivered` for each listener sent it, then `responded`, and ends.
diverge run requests create '{"request":[{"type":"text","text":"Deploy to production?"}],"to":"all"}'
#   {"created":{"id":"7c0e…"}}
#   {"delivered":{"to":{"type":"client","identity":{"issuer":"…","subject":"ronald.alice"}}}}
#   {"responded":{"response":[{"type":"text","text":"yes"}],"from":{"type":"client","identity":{"issuer":"…","subject":"ronald.alice"}}}}

# Ask only some. Every member is optional.
diverge run requests create '{"request":[{"type":"text","text":"ready?"}],"to":{"only":{
  "accounts":[{"issuer":"…","subject":"ronald.alice"}],
  "account_tags":["oncall"],
  "standalone_tools":[{"name":"reviewer"}],
  "standalone_tool_tags":[],
  "dependency_tools":[{"template":"<dependency template id>","owner":"global"}],
  "dependency_tool_tags":[]}}}'

# Listen. Prints each request made of this account; answers are read from standard input, one JSON object a line.
diverge run requests listen
#   {"request":{"id":"7c0e…","request":[{"type":"text","text":"Deploy to production?"}],
#               "from":{"type":"client","identity":{"issuer":"…","subject":"ronald.bob"}}}}
# type on standard input:   {"id":"7c0e…","response":[{"type":"text","text":"yes"}]}
#   {"delivered":{"id":"7c0e…"}}
```

- **One answer.** The first listener whose answer the daemon takes has answered the request. Its line is
  `{"delivered":{"id":…}}`; the creator's `responded` names it. Every other listener that was sent the
  request gets `{"withdrawn":{"id":…}}`, and an answer from one of them is `{"not_delivered":{"id":…}}`.
  The listener that answered gets no `withdrawn` for its own answer.
- **A request lives exactly as long as the `create` command.** Ending the command (or its connection
  dropping) withdraws the request from everyone it was sent to. The daemon stores no request and sets no
  time limit: `create` waits until somebody answers. Use `--timeout` to bound the wait.
- **Late listeners are sent it.** A party that starts listening while a request is pending is sent it then,
  and the creator gets its `delivered` then. Each listener is sent a request once, and the creator is told
  of each delivery once.
- **One `listen` per account or tool**, across all its connections. A second answers `"already_open"`.
- **`listen` never ends by itself.** Do not run it in the foreground of a task that must return: run it in
  the background and write lines to its standard input. Standard input ending does not end the listen. A
  line that is not `{"id":…,"response":[…]}` ends the command with an error.
- **Ignore a `withdrawn` for an ID you never saw.**
- **Who may be asked.** Listening needs no grant. Creating needs the `request` action, which exists in
  three kinds, each with its own `within` and its own tags: `accounts`, `standalone_tools`,
  `dependency_tools`. A tag in `account_tags` names accounts only; a tool carrying the same tag is not
  addressed by it.
  - A request goes to **everything it names that you may ask**, and silently skips the rest: an account
    or tool the grant for its kind does not reach, the unreached members of a tag, a name that names
    nothing. `"to":"all"` is the same with nothing narrowed. The `delivered` lines say who was sent it.
  - It is refused only when it would go to nobody: `"no_recipient"` when no existing account or tool is
    both named and reachable (also `{"only":{}}`, and `"all"` with nothing reachable).
  - Recipients that exist but are not listening do not refuse it: the request is created and waits.
  - No `request` grant of any of the three kinds: `"forbidden"`.
- **Tools.** A standalone tool and a dependency tool can each listen and create, as itself, with the
  grants its image declares. A connected standalone tool (another daemon's) does neither.
- An administrator is addressed like any account: by its identity in `accounts`, or by a tag it carries.
- `from` and `to` name a party as a creator is named: `{"type":"client","identity":{…}}` for an account,
  `{"type":"tool","record":{…}}` or `{"type":"tool","dependency":{…}}` for a tool.

## The host's identities (`diverge run identities …`)

These are the OIDC identities signed in **on the daemon's host**, as
opposed to accounts, which are identities the daemon *admits*. The daemon presents exactly one of them
(its `identity` setting) on every connection it accepts and to every provider it dials, and uses others as "our account" on remote daemons (`diverge-daemons`). No
request ever answers a token.

```bash
diverge run identities list                         # -> {"listed":[{"issuer":…,"subject":…,"tags":[…]}, …]}
diverge run identities list '{"issuers":["https://identity.diverge.network"],"any_tags":["peer"]}'
diverge run identities get   '{"identity":{"issuer":"…","subject":"ronald.daemon-a"}}'                  # {"found":{…}} | "not_found"
diverge run identities tag   '{"identity":{"issuer":"…","subject":"ronald.daemon-a"},"tags":["peer"]}'   # "tagged" | "not_found"
diverge run identities untag '{"identity":{"issuer":"…","subject":"ronald.daemon-a"},"tags":["peer"]}'   # "untagged"

# Sign an identity in on the daemon's host. At Diverge's own issuer the command approves the code
# itself and waits; the account whose username is ronald must be signed in under the CLI's root first.
diverge run identities login --issuer https://identity.diverge.network --subject ronald.daemon-a --client-id diverge
# -> {"signed_in":{"issuer":…,"subject":…}}
# -> or, refused: "held" | "forbidden" | {"error":…}, or an error naming the account to sign in

# At any other issuer the command prints the code and ends; the wait goes on in the background.
diverge run identities login --issuer https://auth.example.com --subject alice --client-id my-client
# -> {"pending":{"user_code":…,"verification_uri":…,"expires_in":…}}   then the command ends
# Approve the code at the issuer, then check that it worked:
diverge run identities list
```

- `identities list` is a snapshot: one `listed` answer holding the array, then the end (no `--watch`).
- `login` flags stand in for body members (`--issuer`, `--subject`, `--client-id`, `--scope`, `--resource`);
  a JSON body with the same members works too. The device code is the only login there is: no password,
  and no browser is opened.
- **At `https://identity.diverge.network`, exactly**, the login needs no person. The subject is
  `<username>.<name>`, and the account with that username (a GitHub or Google sign-in) must already be signed
  in under the CLI's own root with `diverge identity.diverge.network accounts login <issuer>`. The command
  approves the code as that account, waits, and prints `signed_in`. See `diverge-config`.
- **At any other issuer** the command ends once `pending` is printed. A background process of the CLI keeps
  the connection the login lives on until the code is approved, denied or expires, and prints nothing. The
  outcome is read from `identities list`: the identity is there once the login was approved as the subject
  named.
- A login replaces an identity already on the host. `held` means another login or a reader holds the
  identity's lock; nothing was asked of the issuer.
- Signing an identity in does not make the daemon present it: which identity the daemon presents
  stays the daemon's `identity` setting.
- Grants: `{"identities":["login"]}` for an identity not yet on the host;
  `{"identities":{"actions":["get","list","login"],"within":…}}` for those there (filter members `issuers`,
  `subjects`, `all_tags`, `any_tags`); the tagging shape for `tag`/`untag`.
- This is not `diverge identities login` (no `run`), which signs in on the host the CLI runs on. See
  `diverge-config`.

## Worked example: a narrow account for one team

```bash
ISS=https://identity.diverge.network
# 1. As an administrator: a role that reaches only what carries the tag "team-a".
diverge run roles create '{"name":"team-a","grants":[
  {"agents":["create"]},
  {"agents":{"actions":["get","list","message","logs","filetree"],"within":{"all_tags":["team-a"]}}},
  {"agents":{"actions":["get","list","edit","delete"],"within":{"creators":[{"type":"client","identity":{"issuer":"'$ISS'","subject":"ronald.alice"}}]}}},
  {"agents":{"actions":["tag"],"within":{"creators":[{"type":"client","identity":{"issuer":"'$ISS'","subject":"ronald.alice"}}]},"tags":["team-a"]}},
  {"agents_templates":{"actions":["get","list"],"within":"any"}},
  {"accounts":{"actions":["logs"],"within":{"identities":[{"issuer":"'$ISS'","subject":"ronald.alice"}]}}}]}'
# 2. The account.
diverge run accounts create '{"identity":{"issuer":"'$ISS'","subject":"ronald.alice"},"roles":["team-a"]}'
# 3. Tag the existing agents the team should see.
diverge run agents tag '{"agent":{"name":"echo"},"tags":["team-a"]}'
# 4. Check as alice (after adding her to the daemon's accounts: diverge cli daemons <INDEX> accounts add …).
diverge run --account alice agents list
```

## Diagnosing a refusal

There are three different failures. Tell them apart first.

| What you see | What it is |
|---|---|
| The command fails before any answer: the daemon "closed the connection with no answer" / rejected | **Not admitted.** The identity is no account and no administrator of this daemon, or its issuer is not in the daemon's `identities`, or the token could not be had (not signed in on this root). Fix: `accounts create` (as an administrator), check the identity of the CLI account you dialled as (`diverge cli daemons <INDEX> accounts get`), `diverge identities list`. |
| The answer is `"forbidden"` | **Admitted, not allowed.** No grant of the account allows this action over everything the request names. |
| The answer is `"not_found"`, `"no_role"`, `"in_use"`, `{"error":…}` | The request was allowed and came to something else. Not an access problem. |

For `"forbidden"`, work through this list:

1. **Who was I?** The CLI dials as that daemon's default account (`diverge cli daemons <INDEX> default get`) unless `--account` says
   otherwise. `diverge run accounts get '{"account":{…}}'` shows the account's `roles` and whether `admin`.
2. **What do the roles grant?** `diverge run roles list '{"accounts":[<identity>]}' --jq '.added | {name, grants}'`.
3. **Is the action held at all?** Look the command up in [references/grants.md](references/grants.md): the
   kind and the action it needs. A making action (`create`, `add`, `insert`, `login`) needs the bare-array
   grant; an Over grant listing `create` does not exist.
4. **Does `within` reach the record?** Get the record as an administrator and test the filter by hand:
   every member given must pass (tags, names, creators, `active`, …). A `within` with a tag the record does
   not carry reaches nothing. `forbidden` is also what a record that exists but is out of reach answers;
   with no grant of the action at all, `forbidden` comes before the record is even looked up.
5. **Does the request name a second thing?** See "Requests that need more than one grant": roles named in
   an account, the agent or template of an attach, the daemons record of a connected tool, a volume
   mounted, a template's `arguments` asked for, a transfer's destination, the first `relay` ID.
6. **Tagging:** the grant's `tags` must cover every tag in the request, or be `"any"`.
7. **Relayed?** With `relay`, `forbidden` can come from this daemon (no `relay` grant reaching the record)
   or from the remote, which judges the request as the account the record names, by *its* roles there. If
   that identity is one of the remote's administrators (and has no stored account there), it is allowed
   everything there.
8. **Is the requester a tool?** A tool's container holds only what its image declared at registration, and
   is no account; an agent's container asks the daemon nothing. See `diverge-tools`.
9. **Read the command log.** The daemon keeps every request a requester put, with whether it was let
   through. As an administrator (or with `logs` over accounts):

   ```bash
   diverge run logs '{"of":{"type":"account","issuer":"https://identity.diverge.network","subject":"ronald.alice"}}' \
     --jq '.item.account | select(.allowed == false) | {logs_index, created, path, request}'
   ```

   `allowed:false` is exactly a `forbidden`; `path` is the request (`agents/message`), `request` its body.
   An administrator's own log is `{"type":"admin","issuer":…,"subject":…}` and only administrators read it.
   See `diverge-logs`.

Fixes, in order of preference: tag the record so an existing grant reaches it; add a narrowly scoped grant
to the role with `roles edit`; add a role to the account with `accounts edit` (this disconnects it).
Changes apply at the next request; nothing needs restarting.

## Not verified

- The whole access model is read from the SDK and daemon source and from the CLI's schemas. The read-only
  requests (`accounts list|get`, `roles list`, `identities list`, `logs`) were run against a live daemon as
  an administrator; no account, role or tag was created, edited or deleted while writing this, and no
  request was run as a non-administrator account.
- The exact error text of an unadmitted connection is the CLI's own and may differ by version.
- The SDK's module documentation mentions an `assign` action over accounts; no such action exists in the
  grant types on this branch, so it is not listed.

## When this task is done

Load the `diverge` skill again before starting the next Diverge task: it is the index, and it says which
skill the next task needs.
