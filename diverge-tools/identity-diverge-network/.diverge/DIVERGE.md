---
{
  "type": "tool",
  "name": "identity-diverge-network",
  "description": "Signs identities in at identity.diverge.network and publishes images to Diverge's registry, as an account.",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments an identity.diverge.network tool is made with.\n\nNone: the tool is the same for every caller, and the accounts it\nkeeps are told apart by the template the tool was made from. `{}` is\nthe one value; any member is refused, so a misspelled setting is\nnever silently ignored.",
    "type": "object",
    "additionalProperties": false
  },
  "frontend": true
}
---
# identity.diverge.network

Signs identities in on the daemon, at Diverge's own issuer,
`https://identity.diverge.network`, with no person involved, and
publishes images to Diverge's registry, `registry.diverge.network`, as
an account.

**This tool needs permission to log in to new identities.** It signs
identities in on the daemon's host, so the daemon must grant it the
`login` over identities: see [Permissions](#permissions). Without it,
every `login` is refused. To publish it needs `publish` over images as
well.

Every subject at that issuer is `<username>.<name>`: the owner's
username, and a name the owner chose. An **account** is `<issuer>.<id>`:
`github` or `google`, and the owner's user id there. An account sets
its username at that issuer once, and it never changes; an account
that has set none has no subject and no namespace, and `login` and
`publish` as it are refused, saying so. The tool keeps accounts — the GitHub or
Google sign-ins themselves — in its database, and uses one to approve
a login the daemon begins. So an agent that may call this tool can
put any subject of any account the tool holds onto the daemon's host,
and can publish images into any of those accounts' namespaces at the
registry.

## The tools

For agents, over MCP:

| tool | parameters | does |
|---|---|---|
| `list_accounts` | none | the accounts the tool holds, each as `{"username","account","issuer","id"}` |
| `login` | `username`, `subject` | signs the subject `<username>.<subject>` in on the daemon's host, as that account, and waits for it |
| `publish` | `username`, `repository`, `images`, and optionally `tags`, `provider` | publishes the images to `registry.diverge.network/<username>/<repository>` as that account, and waits for it |

- `username` names the account: the one it set at
  `identity.diverge.network`, as `list_accounts` gives it.
- An account listed with `"username": null` has set none. No tool can
  name it, so it can sign nothing in and publish nothing until a person
  sets its username at the page. No tool call sets one: it is
  permanent, and choosing it is a person's decision.
- `account`, `issuer` and `id` in the list say which GitHub or Google
  sign-in it is, and are given to no tool.
- `subject` is the name alone, the second part: 1 to 64 of `a-z`, `0-9`,
  `-` and `_`. The whole subject is the account's username, a period,
  and it.
- A subject never signed in before and one that already is are the
  same call: a login under a new name is how a subject begins.
- `login` answers `{"signed_in": {"issuer": "https://identity.diverge.network", "subject": "<username>.<subject>"}}`.
  The identity is then on the daemon's host. Which identity the daemon
  presents is still its own configuration's.
- No tool answers a token, and no tool adds an account.

### Publishing

`publish` is the daemon's `images publish` with the account filled in.
It takes no username, no password and no registry:

- **The registry is fixed**: `registry.diverge.network`.
- **The namespace is the account's username**: the image goes to
  `<username>/<repository>`, the one namespace that account may
  write. `repository` is one name with no `/`.
- **The credential is the account's own**, supplied by the tool. It is
  handed to the daemon for that one publish and appears in no answer.
- `images` is one or more, each `{"digest": "sha256:…"}` with optional
  `platforms`. One image taking all its platforms is published as it
  is. Anything else is merged into one new image first, and the answer
  then carries `merged` with the new digest.
- `tags` are put on it. With none it is published by digest alone.
- `provider` picks where it is done. Absent, the daemon picks a
  provider that uses the registry.
- The images need not be on any one provider: the daemon gathers them
  from wherever they are.
- `publish` answers
  `{"published": {"provider", "digest", "created", "reference", "tags"}}`.
  `provider` is where it was done. `created` is whether the publish
  made that image on the provider, or it was there already.
- A provider must list `registry.diverge.network` among the registries
  it uses, or no provider can do the publish.
- The provider the publish is done on is handed the account's token
  for the length of the publish. Publish only on providers trusted
  with it.

## The page

For people, in the tool's frontend. It is where an account gets into
the tool, which no tool call can do:

- **Add an account.** Choose GitHub or Google. The page shows a code,
  with a button that copies it,
  and a link to the issuer's own page; sign in there and enter the
  code, and the account appears on the page by itself.
- **See the accounts** the tool holds, each with its username, and the
  name its issuer has for it now: the GitHub login, or the Google
  email. The issuer's name is asked each time the page loads and is
  kept nowhere.
- **Set a username** for an account that has none. The page says
  whether the one typed is free, and asks once more before setting it:
  a username is permanent. It can never be changed or given up, and no
  two accounts have the same one.
- **Remove an account.** The tool forgets it. What it already signed in
  on a daemon stays signed in.

The accounts are shared: anyone the daemon lets open the page can add
or remove them and set a username, and any agent attached to the tool can log in as any of
them. The page answers no token.

## Running it

A tool container, standalone or a dependency.

- **Arguments:** none; `{}`.
- **Secrets:** none. The accounts are kept in the tool's database, not
  in the vault.
- **Database:** the tool's own, through the proxy. One table,
  `identity_accounts`, each row an account, its issuer's tokens and
  its username once known, under the template the tool was made from.
- **Network:** the registry is reached by the provider, not by the
  tool. The tool's own is outbound HTTPS to `identity.diverge.network`, and to
  GitHub (`github.com`, `api.github.com`) and Google
  (`oauth2.googleapis.com`, `openidconnect.googleapis.com`) to sign an
  account in, to renew its token, and to read its name.
- **Frontend:** yes; the page above.

### Permissions

**The tool needs permission to log in to new identities**, and to log
in again ones already on the host, and to publish images. It declares
all three itself, at its registration, and the daemon must grant them
for `login` and `publish` to work:

```json
[
  { "identities": ["login"] },
  { "identities": { "actions": ["login"], "within": "any" } },
  { "images": { "actions": ["publish"], "within": "any" } }
]
```

| grant | what it allows | without it |
|---|---|---|
| `{"identities": ["login"]}` | signing in an identity that is not on the daemon's host yet: a subject's first login there | a login of a new subject answers that it is forbidden |
| `{"identities": {"actions": ["login"], "within": "any"}}` | signing in again one that is already there | a login of a subject already on the host answers that it is forbidden |
| `{"images": {"actions": ["publish"], "within": "any"}}` | publishing images from any provider | every `publish` answers that nothing was published, the daemon granting it none |

Nothing else: the tool lists no identity, tags none, builds, pushes,
lists and deletes no image, and asks the daemon for nothing but the
login and the publish. The page needs no grant of the
tool's own; who may open it is the daemon's `frontend` grant over the
tool.
