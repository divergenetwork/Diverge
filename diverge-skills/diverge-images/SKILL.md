---
name: diverge-images
description: Container images in Diverge — what an image reference is (`{digest, references:[{registry,name}]}`), where a provider gets an image from (its Store, its podman cache, a listed registry, the caller), how to build an image on a provider with `diverge run images build` and get its digest, the image's `/.diverge/DIVERGE.md` (frontmatter `type` of `agent`, `standalone_tool` or `dependency_tool`, `name`, `description`, `schema`, `memory`, `disk`, `secrets`, a tool's `mode`, a dependency tool's `scope`, `frontend`), the `diverge run images read` command (the one way to an image's `DIVERGE.md`), and a reference of every image in the repository — the six agent-loop images (python, lone, openrouter, codex, hermes, cc) and the example images (courier, taskboard, memory, alarm, chat, human) with each one's `arguments`, secrets and their modes (fixed/rotating), dependencies and a minimal template body. Load it when you need an image digest for a template, when a deploy fails with the image unavailable or "the provider pulls from no registry the image's references name", when you need to know what `arguments` or `secrets` an image takes, when you build or rebuild an image, or when you read an image's DIVERGE.md.
---

# Diverge images

An image is an OCI container image whose program speaks Diverge's inside-container contract: an agent
loop (it takes messages and answers them) or a tool (an MCP server). Diverge never names an image by
tag. A template names it by **manifest digest**, plus a list of **references** that say where the bytes
may be fetched. The daemon keeps no record of images; it reaches one through a provider or a remote daemon
that can supply it.

This skill covers: the image reference, where a provider finds an image, building one into the
provider's store, `/.diverge/DIVERGE.md`, the `images read` command, and what each image in the repository
takes. Templates, agents and tools themselves are in `diverge-agents` and `diverge-tools`; vault keys
for secrets are in `diverge-storage`.

## The image reference

Every place that names an image (an agent template, a tool template, a dependency entry,
`images read`, `images build`) uses the same object:

```json
{
  "digest": "sha256:9f2c…",
  "references": [
    { "registry": "docker.io", "name": "diverge-local/python" }
  ]
}
```

- `digest` — the manifest digest, `<algorithm>:<hex>`. **The digest is the image.** A provider runs
  the bytes the digest names and nothing else; the runtime hashes what it pulls.
- `references` — zero or more places the bytes may be fetched. `registry` is a bare host
  (`docker.io`, `ghcr.io`, `registry.example.com:5000`; no scheme, no path). `name` is the repository
  path there (`library/nginx`, `myorg/myimage`; no host, no tag). The image is
  `<registry>/<name>@<digest>` at that registry. `name` must be a valid lowercase repository path;
  nothing is normalized.
- References locate and do not identify. Two images with one digest are one image whatever each
  lists, and a template's id hashes the image **without** its references.

### The rule that trips everyone: references must name a registry the provider lists

A run names no platform. The provider runs the platform of the image that it can run
(`capabilities.containers.platforms`), and refuses an image that has none of them.

Before the daemon asks a provider to run a container, it checks the provider's capabilities
(`images.registries`, visible inside a connected record's `status` in `diverge run providers list`,
under its `info`'s `capabilities`). A provider passes when
**some reference of the image names a registry that provider lists**. The default provider lists
`registry.diverge.network`, `docker.io`, `ghcr.io` and `quay.io`.

- An image with **no references** fails this test ("the image names no registry").
- An image whose references name only unlisted registries fails it ("the provider pulls from no
  registry the image's references name").
- When no connected provider passes, the daemon falls back to asking whether a provider's Store or a
  remote daemon holds the image. An image built with `diverge run images build` is in the Store and is
  found this way.

So an image is named one of two ways: by its digest alone, when it was built into the Store with
`diverge run images build`; or by its digest plus a reference naming a listed registry that really
serves it. An image that exists only in a provider's local cache, built or loaded behind the provider's
back, is not an image the daemon can read: `images read` does not hold it, so it has no readable
`DIVERGE.md`, and an agent or tool cannot be created from it.

## Where a provider gets an image

For a run, the provider looks in this order and takes the first that has the digest:

1. **The caller's Store.** A digest the calling daemon owns in the provider's image Store (one built
   or pushed on that provider through the provider protocol). Answered as source `store`.
2. **The provider's podman cache, then a listed registry.** If podman already holds an image with
   that digest under a name that does **not** start with `127.0.0.1:`, it is run as it is with nothing
   pulled. Otherwise every reference that names a registry in the provider's configuration is asked
   at once (`<registry>/<name>@<digest>`, anonymously: the provider holds no registry credential) and the
   first that serves it is pulled. References naming unlisted registries are ignored. Answered as
   source `{"registry":{"reference":"…"}}`.
3. **The caller.** The provider asks the daemon whether it holds the image; the daemon may supply it
   from another provider's Store or a remote daemon over OCI channels, and must then pass a nonce
   proof. Answered as source `client`; a failed proof fails the run.
4. Otherwise the image cannot be supplied and the start fails.

A registry is preferred to the caller, and the caller's Store to both. The provider tells the daemon
which source answered; no CLI answer reports it, so read it off the order above.

## Building an image

`diverge run images build` builds an image from a Containerfile on a provider, into the daemon's Store
there. This is the only way to build one: the provider's container runtime is the provider's own, and
an image put into it by any other means is not in the Store.

```bash
diverge run images build '{
  "provider": {"issuer": "http://127.0.0.1:14981", "subject": "provider"},
  "platforms": ["linux/amd64"],
  "containerfile": "FROM docker.io/library/alpine\nCOPY .diverge /.diverge\n",
  "context": {"volume": "my-source", "path": ["tool"]}
}' > build.log
# {"output":"[1/2] STEP 1/6: FROM …"}        one line per line the build says
# {"built":{"digest":"sha256:…","bytes":7675239}}
```

- **`provider`**: which provider builds and holds the image, by its identity (`diverge run providers list`).
- **`containerfile`**: the Containerfile's whole text, not a path.
- **`platforms`**: required, one or more, each `os/architecture` (`linux/amd64`, `linux/arm64`). Each
  must be a platform the provider builds for: `capabilities.images.store.build.platforms` in
  `diverge run providers list`. The default provider builds for its own machine's platform only.
- **`context`**: a directory of a volume on that same provider, as `{"volume":"<name>","path":[…]}`. The
  Containerfile's `COPY` and `ADD` read from it. Omit it for a Containerfile that copies nothing.
- **No description here.** An image's one-line description is its own: the `description` in its
  `DIVERGE.md` (below). No request gives an image one: a `description` in a body is ignored.
- **The last line is the result**:
  - `{"built":{"digest":…,"bytes":…}}`: the image is in the Store. The digest is what a template names.
  - `{"exists":{"digest":…}}`: the build produced a digest already in the Store; nothing was added.
  - `"insufficient_capacity"`: the Store is full. `"context_held"`: something has the context volume to
    itself. `"unsupported"`: the provider has no Store (its `images` are not enabled).
    `"platform_unsupported"`: the provider does not build for a platform named; nothing was built.
  - `"forbidden"`: no `build` grant over images reaching that provider, or no `mount` grant over the
    context volume. The grant is `{"images":{"actions":["build"],"within":"any"}}`, or narrowed to
    providers by identity or tag (see `diverge-access`).
    `"not_found"`: no such provider, or no such volume on it.
  - `{"error":…}`: the build failed; the `output` lines before it say why.
- A build cannot be cancelled. Stopping the command leaves it running on the provider.
- Each image that compiles Rust takes roughly ten minutes. Run builds one at a time, in the background,
  with output to a file.

### Getting source in front of a build

A build reads its context from a volume. For source on the provider's own host, list the folder as a
fixed, read-only volume of the provider, with the provider stopped, and start it again:

```bash
diverge provider stop
diverge provider volumes enable          # only if `diverge provider volumes enabled` prints false
diverge provider volumes fixed add --name my-source --path C:/path/to/source --bytes 34359738368 --mode read_only
diverge provider start
```

The provider needs a Store to build into, and one that builds. In `diverge run providers list`, a
connected provider's `capabilities.images.store` is an object when it keeps a Store, and that object's
`build` is there when it builds, naming the platforms it builds for:
`"images":{"store":{"build":{"platforms":["linux/amd64"]}},"registries":[…]}`. A provider with a Store
and no `build` takes pushes and refuses builds. If there is no `store`:

```bash
diverge provider stop
diverge provider images enable           # a Store of 32 GiB
diverge provider start
```

Otherwise create a volume and upload the source to it (`diverge-files`).

### Rules

- **A Containerfile sees only its context.** One that copies from several folders needs a context that
  holds them all: the Containerfiles in this repository (`diverge-agents/*/Containerfile`,
  `diverge-examples/*/*/Containerfile`) take the repository root as their context.
- **The Store is the daemon's.** An image built here runs for this daemon on that provider with nothing
  pulled. Another daemon does not see it.
- **Rebuild images whenever the inside-container contract changes** (the SDK's
  `container_proxy::inside`, the container proxy, or the loop crate). An image built against an older
  contract deploys but misbehaves against the current proxy (refused enqueues, a channel that finishes
  with no answer). The container proxy itself is never part of an image; the provider mounts it in.
- **A rebuild is a new digest.** Templates name digests, so a rebuilt image needs a new template.

Then use the digest in a template (see `diverge-agents` / `diverge-tools`). An image the daemon holds in
a Store is found by its digest, so it needs no `references`:

```json
{"type":"agent",
 "image":{"digest":"sha256:…"},
 "extra_memory":0,"extra_disk":0,
 "arguments":{"python":"[{\"type\":\"assistant_text_content\",\"text\":\"hello\"}]"},
 "secrets":[]}
```

## Listing the daemon's Stores: `diverge run images list`

```bash
diverge run images list                                    # every image the grant reaches, then "listed"
diverge run images list '{"providers":[{"issuer":"…","subject":"ronald.provider"}]}'
diverge run images list '{"digests":["sha256:…"]}' --jq '.added'
diverge run images list --watch                            # stays open: added / changed / removed
```

- Body members, all optional: `providers`, `digests`, `created_from`, `created_to` (RFC 3339), `count`,
  `relay`.
- One image, once: `diverge run images get '{"provider":{"issuer":"…","subject":"…"},"digest":"sha256:…"}'`
  answers `{"found":{…}}` with the same members as a list line, `"not_found"` (no such provider on record,
  or no such digest in its Store), `"forbidden"`, or `{"error":…}` (the provider is not connected now, has
  no Store, or its listing failed). It needs the images grant's `get` action.
- Each line is `{"added":{"provider":…,"digest":…,"bytes":…,"created":…,
  "manifest_platforms":[…],"platforms":[…],"references"?:[…]}}`, then `"listed"`. With `--watch`,
  `{"changed":…}` (a platform added, a reference recorded) and `{"removed":…}`
  follow as they happen. Other answers: `"forbidden"`, `{"error":…}`.
- **`manifest_platforms`** is every platform the image is for; **`platforms`** is the ones the daemon
  holds on that provider. The two equal, the image is whole there. Fewer held, push the missing ones
  (below). `references` is where the image was published from that provider, absent until it is.
- What is listed is what the **daemon** owns on each provider: images built with `images build` or pushed
  with `images push`. It is not the provider's pull cache, and not another daemon's Store.
- A provider that is not connected, that has no Store (its `images` are not enabled), or whose
  listing is not whole yet contributes nothing; it appears when it is.
- Grant: `list` over images, reaching the provider.

## Pushing an image that is already built: `diverge run images push`

```bash
diverge run images push '{
  "provider": {"issuer":"…","subject":"ronald.provider"},
  "platforms": ["linux/amd64","linux/arm64"],
  "image": {"digest":"sha256:…","references":[{"registry":"ghcr.io","name":"acme/worker"}]},
  "source": "daemon"
}'
# {"added":{"platforms":["linux/amd64","linux/arm64"],"bytes":123456789,"source":{"registry":{"reference":"ghcr.io/acme/worker@sha256:…"}}}}
# {"pushed":{"digest":"sha256:…","bytes":123456789,"source":{"registry":{"reference":"ghcr.io/acme/worker@sha256:…"}},"platforms":["linux/amd64","linux/arm64"]}}
```

Puts an image into the daemon's Store on a provider without building it.

**`platforms` is required**: one or more, each `os/architecture`. Those platforms of the image are what
the provider takes.

- Every platform named must be one the image has (its `manifest_platforms`). If one is not, the push ends
  `"platform_not_found"` and nothing is added: it is all of them or none.
- A provider holds a platform it cannot run. Pushing `linux/arm64` to an amd64 provider is fine.
- Pushing a digest the daemon already owns there adds the platforms it lacked and answers `{"pushed":…}`.
  This is how an image held in part becomes whole.

Where it comes from, in order:

1. The daemon already owns the digest there with every platform named: `{"exists":{"digest":…}}`, nothing
   moves.
2. A reference names a registry that provider lists, and the registry serves the digest: the provider
   fetches it. `source` is `{"registry":{"reference":…}}`.
3. Otherwise the provider asks for the manifest, the blobs and a proof, and the request's **`source`**
   answers.

**`source` is required.** It is one of:

| `source` | The bytes come from |
|---|---|
| `"daemon"` | Wherever the daemon finds the image. It asks every connected provider and every remote daemon it has a record of, all at once, which platforms each can supply. |
| `"client"` | The client that sent the request, which must hold the bytes |

Neither needs a grant beyond `push`.

**With `"daemon"`, the platforms can come from different places.** One provider holding `linux/amd64` and a
remote daemon holding `linux/arm64` together satisfy a push of both:

- The daemon looks first and moves nothing until every platform named has a source. A platform nobody
  holds ends the push `"unavailable"` with nothing added.
- Then each source's platforms are pushed at the same time, each from its own source.
- Each group that lands prints one `{"added":{"platforms":[…],"bytes":N,"source":…}}` line as it lands.
  Platforms the provider already held print `{"added":{"platforms":[…],"source":"store"}}`, with no `bytes`.
- If one group fails, the last line is that failure, but every `added` line printed before it is true:
  those platforms are held. Push again naming only the rest.

**The CLI refuses `"source":"client"`** before dialing anything (`not supported from the command line`): the
`diverge` program holds no image bytes. A client that has the bytes pushes through the SDK (`images.push`
with a store that answers the asks). In the answer, `pushed.source` is the *provider's* word for where it
got the image, so a push the daemon sourced still reads `"client"` there: the daemon was the provider's
client.

A client that does not answer, a daemon that finds no source for a platform named, or a source that stops
part way ends the push `"unavailable"`.

Last line, one of: `{"pushed":…}` (with `platforms`, every one named), `{"exists":…}`, `"insufficient_capacity"` (the Store is full),
`"unavailable"`, `"proof_failed"`, `"malformed"` (not an image the provider takes),
`"platform_not_found"` (a platform named is not in the image), `"unsupported"` (the provider has no Store), `"forbidden"` (no `push` grant reaching the provider), `"not_found"` (no such provider), `{"error":…}`. Before it, one
`{"blob":{"digest":…,"bytes":…}}` per blob the provider pulled from its caller, and one `{"added":…}` per
group of platforms that landed.
A push cannot be cancelled.

## Deleting an image, or some of its platforms: `diverge run images delete`

```bash
# the whole image
diverge run images delete '{"provider":{"issuer":"…","subject":"ronald.provider"},"digest":"sha256:…"}'
# "deleted"

# only some platforms; the rest stay, under the same digest
diverge run images delete '{"provider":{…},"digest":"sha256:…","platforms":["linux/arm64"]}'
# "deleted"
```

Takes an image out of the daemon's Store on a provider. It cannot be undone; build or push the image
again to have it back.

- **No `platforms`, the whole image goes.** With `platforms`, only those go.
- **The digest does not change** when some platforms stay. `images list` then shows `platforms` shorter
  than `manifest_platforms`, exactly as after a push of only some platforms.
- **All or nothing.** If one platform named is not held there, the answer is `"platform_not_held"` and
  nothing is deleted. Check `platforms` in `images list` first.
- **Naming every platform held deletes the image**, the same as naming none.
- **Nothing shared is lost.** A layer another platform, another image or another owner still uses stays on
  the provider. Only what nothing else uses is freed.
- **In use.** If a container is running the image, the whole image and the platform that provider runs
  cannot be deleted: `"in_use"`, nothing changed. Stop the container and ask again. A platform the provider
  does not run can still be deleted.
- Only the daemon's own image is deleted. Another daemon's image of the same digest on that provider is
  untouched.
- A template that names a deleted digest is not told. At its next run the image is found again from
  another source, or the run fails there.

One line, one of: `"deleted"`, `"in_use"`, `"platform_not_held"`, `"not_found"` (no such provider, or
the daemon owns no image of that digest there), `"unsupported"` (the provider has no Store),
`"forbidden"`, `{"error":…}` (the provider is not connected, among others).

Grant: `delete` over images, reaching the provider.

## Joining images into one: `diverge run images merge`

Two images built for two platforms are two different images, each with its own digest. A push cannot join
them: a push names one digest. `images merge` makes one new multi-platform image out of several, in the
daemon's Store on a provider, without sending it anywhere.

```bash
# An amd64 image built on one provider and an arm64 image built on another, joined on the first.
diverge run images merge '{
  "provider": {"issuer": "https://identity.diverge.network", "subject": "ronald.provider"},
  "images": [
    {"digest": "sha256:aaaa…"},
    {"digest": "sha256:bbbb…", "platforms": ["linux/arm64"]}
  ]
}' > merge.log
# {"blob":{"digest":"sha256:…","bytes":…}}          one line per blob pulled onto the provider
# {"added":{"platforms":["linux/arm64"]}}             one line per group of platforms that landed
# {"merged":{"digest":"sha256:cccc…","created":true,"bytes":…}}
```

- **`provider`** (required): where the merged image is made and kept.
- **`images`** (required, one or more): each a `digest`, and optionally the `platforms` taken of it. With
  `platforms` left out, every platform that image has is taken.
- **Where the images come from is not yours to say.** Whatever the provider's Store lacks is fetched from
  wherever the daemon finds it: every provider it is connected to, and every remote daemon it has a record
  of. An image already in that provider's Store is not fetched.
- **The last line is the result.**
  - `{"merged":{"digest":…,"created":…}}`: the image is in the Store; the digest is what a template names.
    `"created":false` means the daemon already owned that exact image there.
    The same images merged on any provider give the same digest.
  - `"unavailable"`: an image named, or a platform taken of it, was found on no source. Check the digest,
    that the provider or daemon holding it is connected, and that it holds that platform
    (`diverge run images list`).
  - `"platform_repeated"`: two entries take the same platform. One image holds a platform once: name
    `platforms` on the entries so they do not overlap.
  - `"platform_not_found"`: a platform named is not one that image has.
  - `"insufficient_capacity"`: the provider's Store is full. `"unsupported"`: it keeps no Store.
  - `"forbidden"`: no `merge` grant over images reaching that provider
    (`{"images":{"actions":["merge"],"within":"any"}}`). `"not_found"`: no such provider.
  - `"proof_failed"`, `"malformed"`, `{"error":…}`: a source did not prove it holds what it sent; bytes
    that are not an image; anything else.
- What was fetched before a failure stays in the Store.
- **One image with all its platforms** has nothing to join: it is brought in and reported as `merged` with
  its own digest. That is the same as a push.
- A merge cannot be cancelled.

## Publishing to a registry: `diverge run images publish`

```bash
# one image, as it is, with two tags
diverge run images publish '{
  "images": [{"digest": "sha256:…"}],
  "reference": {"registry": "ghcr.io", "name": "acme/worker"},
  "tags": ["1.4", "latest"],
  "credential": {"username": "…", "password": "…"}
}'

# two single-platform images, published as one multi-platform image
diverge run images publish '{
  "images": [
    {"digest": "sha256:aaa…", "platforms": ["linux/amd64"]},
    {"digest": "sha256:bbb…", "platforms": ["linux/arm64"]}
  ],
  "reference": {"registry": "ghcr.io", "name": "acme/worker"},
  "tags": ["1.4"],
  "credential": {"username": "…", "password": "…"}
}'
```

Sends one or more images to a registry as one image. The images do not have to be on any one provider,
or on any provider you name: the daemon gathers them.

| member | required | what it is |
|---|---|---|
| `images` | yes, one or more | each `{"digest":…}`, with optional `platforms`. Absent `platforms` means every platform that image's manifest names |
| `reference` | yes | `{"registry":…,"name":…}`: the registry host and the repository path. No tag here |
| `tags` | no | tags to put on it. None, and it is published by digest alone |
| `credential` | no | `{"username":…,"password":…}`; the password is whatever the registry takes: a password, a token. Absent, the registry is asked as nobody |
| `provider` | no | the provider to do it on. Absent, the daemon picks |

What the daemon does, in order:

1. **Works out the platforms.** A part with no `platforms` takes all its image has. A platform named by
   two parts ends the request `"platform_repeated"` before anything moves.
2. **Picks a provider** (unless you named one): connected, with a Store, that lists the target registry,
   and reached by your `publish` grant. Among those, the one already holding the most of what is needed.
3. **Brings in what that provider lacks**, from any provider or remote daemon that has it, every part at
   the same time. This is the same as `images push` with `"source":"daemon"`.
4. **Merges**, unless the request is exactly one image taking all its platforms. A merge makes a new
   image with a new digest; the same parts give the same digest every time.
5. **Publishes** it with the tags and the credential.

Lines printed, as they happen:

| line | meaning |
|---|---|
| `{"blob":{"digest":…,"bytes":N}}` | a blob pulled onto the provider |
| `{"added":{"platforms":[…],…}}` | a group of platforms landed on the provider |
| `{"merged":{"digest":…,"created":true,"bytes":N}}` | the parts are one image; `created` false means that merge was there already |
| `{"uploaded":{"digest":…,"bytes":N}}` | a blob the registry took |

Last line when it works:

```json
{"published":{"provider":{"issuer":"…","subject":"…"},"digest":"sha256:…","created":true,"reference":{"registry":"ghcr.io","name":"acme/worker"},"tags":["1.4"]}}
```

- **`provider`** is where it was done, and where the image now is in the daemon's Store.
- **`digest`** is what the registry holds: the image's own digest when one whole image was published,
  the merged digest otherwise. Use this digest in templates.
- **`created`** says whether this request made that image on the provider (`true`) or it was already in
  the daemon's Store there (`false`).

Other last lines: `"forbidden"`, `"not_found"` (the provider named is unknown), `"no_provider"` (nobody
reachable can do it), `"registry_unlisted"` and `"unsupported"` (the provider named does not use that
registry, or has no Store), `"unavailable"` (a platform has no source), `"platform_repeated"`,
`"platform_not_found"`, `"insufficient_capacity"`, `"proof_failed"`, `"malformed"` (also a bad repository
path or tag), `"unauthorized"` (the registry refused the credential), `"denied"` (the credential may not
write there), `{"error":…}`.

Things to know:

- **What was brought in stays** in the daemon's Store on that provider, after a success and after a
  failure. Remove it with `images delete` if you do not want it.
- **A failure part way does not undo** the `added` and `merged` lines before it. Running the same request
  again picks up from there: what is already held is not moved again.
- **The credential is not kept.** It is passed to the provider for that one publish. The daemon's record
  of the request has the password replaced.
- A publish cannot be cancelled.

Grant: `publish` over images, reaching the provider it is done on.

## `/.diverge/DIVERGE.md`: the image's own word on what it takes

Every image ships one file in the folder `.diverge` at the root of its filesystem, `/.diverge/DIVERGE.md`.
In a source tree the same folder sits beside the Containerfile (`.diverge/DIVERGE.md`), and the
Containerfile copies the folder whole: `COPY .diverge /.diverge`. It is how a caller learns
what `arguments` an image accepts, what kind of image it is, and what a container of it needs,
without knowing the image.

```text
---
{
  "type": "standalone_tool",
  "name": "chat",
  "description": "A chat between a person and the agents attached to it, with a live web page.",
  "schema": { "type": "object", "additionalProperties": false, "description": "…" },
  "memory": 268435456,
  "disk": 268435456,
  "mode": "eager",
  "frontend": true
}
---
# Chat

Prose: what the image runs, its tools, the secrets it declares, how to run it.
```

A dependency tool's frontmatter, and an agent's that states a secret:

```json
{
  "type": "dependency_tool",
  "name": "memory",
  "description": "A notebook of notes that outlasts an agent's conversation.",
  "schema": { "type": "object", "additionalProperties": false },
  "memory": 268435456,
  "disk": 268435456,
  "mode": "lazy",
  "scope": "instance"
}
```

```json
{
  "type": "agent",
  "name": "openrouter",
  "description": "An agent that runs a chat-completions loop against OpenRouter.",
  "schema": { "type": "object" },
  "memory": 536870912,
  "disk": 1073741824,
  "secrets": [
    { "name": "OPENROUTER_API_KEY", "description": "The OpenRouter API key every completion is asked with." }
  ],
  "chat": { "prompt_modalities": { "image": true, "audio": true, "file": true } }
}
```

An image is exactly one of three kinds:

- `"type": "agent"`: an agent. An agent template names it.
- `"type": "standalone_tool"`: a tool made by hand from a tool template. It is nobody's dependency: an
  agent that declares it as a dependency is refused.
- `"type": "dependency_tool"`: a tool an agent's program declares for itself. Nothing else makes one: a
  tool template of such an image is refused when a tool is created from it.

`"type": "tool"` is not a type.

The parse rules, in the order they are checked:

1. The first line is exactly `---` (a line ends at `\n`; a trailing space or a carriage return makes
   it not a fence). Otherwise there is no frontmatter.
2. The frontmatter is every line up to the next line that is exactly `---`. No such line is an
   unterminated frontmatter.
3. The frontmatter is **one JSON object**.
4. It has **`type`, required: `"agent"`, `"standalone_tool"` or `"dependency_tool"`**. Missing, or
   anything else (`"tool"` among them), is an error. The type decides which members are allowed.
5. All three types: `name` (string, required: see rule 6), `description` (string, required: one line, 1 to 132
   characters, no line break or other control character, no space at either end), `schema` (required),
   `memory` and `disk` (each required: a whole number of bytes), `secrets` (optional, none when absent:
   an array of objects, each exactly `{"name": "…", "description": "…"}`, both required),
   `frontend` (boolean, optional, `false` when absent). An agent's also: `chat` and `structured_decisions` (each
   `false` or an object, never `true`; optional, `false` when absent). A standalone tool's also: `mode`
   (required: `"lazy"` or `"eager"`; there is no default). A dependency tool's also: `mode`, the same,
   and `scope` (required: `"instance"`, `"template"` or `"global"`). **A tool's must not have `chat`
   or `structured_decisions` at all, not even as `false`.** An agent's must not have `mode` or `scope`,
   and a standalone tool's must not have `scope`. A missing member, **any member that is not
   one of its type's**, or a wrong type is an error.
6. **`name` is what a repository may be named, but that it may have capitals**: 1 to 128 bytes,
   letters `a` to `z` and `A` to `Z` and digits `0` to `9` in runs joined by a separator, where a
   separator is `.`, `_`, `__` or one or more `-`. It starts and ends with a letter or a digit. No
   space, nothing else: `my-agent`, `My-Agent`, `my_agent.v2` and `tool2` are names; `my agent`,
   `-agent`, `agent-`, `a..b` and `a___b` are not. Case is kept: `Agent` and `agent` are two names. A
   registry's repository name follows the same rule with no capital, so an image's name in lowercase
   can always be its repository's.
7. `schema` must be a JSON object: it is the **JSON Schema of the image's `arguments`**, not of the
   whole template.
8. Every secret's `name` is one line of plain text, 1 to 75 characters, with no space at either end,
   and no two secrets have the same name. A secret's `description` follows the rule of the image's own
   `description`: one line, 1 to 132 characters, no space at either end.
9. **An agent's must have at least one of `chat` and `structured_decisions` as an object.** Both
   `false`, both absent, or one `false` and one absent, is an error.
10. The body is everything after the closing fence, verbatim.

Written out, the members are in this order: `type`, `name`, `description`, `schema`, `memory`, `disk`,
`secrets`, `mode`, `scope`, `frontend`, `chat`, `structured_decisions`.

There is no `icons` member: an `icons` in the frontmatter is an unknown member, and an error.

An image whose document breaks a rule has no document as far as a provider is concerned: `--info`
answers `{"error":…}`, and the daemon refuses to create an agent or a tool from it. The daemon also
refuses an agent made from a tool's image, a standalone tool made from an image that is not a
`standalone_tool` image, and a dependency declared of an image that is not a `dependency_tool` image.

**Size limit: 1048576 bytes (2^20) per file.** It applies to `DIVERGE.md` and to each icon, each by
itself, measured as the file's length in the image. A provider reads none of a larger file and answers
that it is too large: `"too_large"` for an icon, and for the document, through `--info`, `{"error":…}`.

What each member means:

- `type` — `agent`, `standalone_tool` or `dependency_tool`: what the image's program is. It comes back in the parsed document, so a
  caller can tell the three apart without knowing the image.
- `name` — what the image calls itself. Nothing makes it unique.
- `description` — what the image is, in one line. It is the only description an image has: it travels
  inside the image to every provider and registry, and nothing outside the image can set or change it.
  To change it, change the file and build a new image. The body beneath the frontmatter is the long
  account of the same image.
- `schema` — what the template's `arguments` value may be. The daemon and the provider carry
  `arguments` to the program unread; the program validates them at its registration, so arguments
  that break the schema surface as a start error in the agent's or tool's log, not at
  `templates create`.
- `memory`, `disk` — whole numbers of bytes: what a container of the image needs. A template or a
  dependency declaration does not repeat them. It states `extra_memory` and `extra_disk`, what the
  container gets beyond them, and the container's limit is the sum.
- `secrets` — every secret the program could ever take, whatever its arguments, each by its `name` and
  a one-line `description` of what it is. A template or a dependency declaration names the ones its
  container needs: all of them or any subset, and none the image does not state. Absent means the
  image takes no secret.
- `mode` — a tool's image only. `lazy`: every tool made from the image runs while it is used. `eager`:
  it is kept running while anything holds it. No template and no declaration says another (see
  `diverge-tools`).
- `scope` — a dependency tool's image only. How far every tool made from the image, and its database,
  is shared: `instance` (one per agent), `template` (one per agent template) or `global` (one for all).
  No declaration says another.
- `frontend` — `true` when the program serves a web UI beneath `/frontend` (see the `frontend`
  commands in `diverge-agents` / `diverge-tools`). To look like the rest of Diverge in the person's
  theme, the page links `<link rel="stylesheet" href="/diverge/theme.css">` and
  `<script src="/diverge/theme.js"></script>` after its own styles, and names every colour, size,
  space and edge by the theme's roles (`--bg`, `--surface`, `--line`, `--line-strong`, `--text`,
  `--text-3`, `--accent`, `--on-accent`, `--ok`, `--bad`, `--font-ui`, `--text-sm`, `--control`,
  `--radius-md`… as in `diverge-theme/theme.css`). The viewer answers both on the frontend's own
  address, outside `/frontend`, and keeps the page's `data-theme` the person's. Elsewhere both are
  missing, so a page keeps fallback values of its own. Not taking them is allowed: the page is its
  maker's. The example `chat`, `human` and identity pages take them.
- **Icons are files, not members.** An image ships up to three, each optional, each by a fixed name in the
  same `.diverge` folder: `icon.svg` (an SVG), `icon-512.png` (a PNG of exactly 512×512), `icon-64.png`
  (a PNG of exactly 64×64). The file being there is the whole declaration. A provider answers each on
  its own channel of an `images::read` scope (`icon_svg`, `icon_512`, `icon_64`), and so does the daemon,
  which keeps each icon it has read by digest as it keeps the document.
  `diverge run images read '<body>' --icon svg|512|64 --out <path>` writes one to a file and prints
  `{"path":…,"bytes":N}`, or `"missing"`, `"too_large"` or `{"error":…}`.
- **An icon is checked before it is answered.** A PNG must decode and be exactly its name's size. An
  SVG must be UTF-8, well-formed XML with no `<!DOCTYPE`, with `svg` in the SVG namespace as its
  outermost element, no `script` element, and no attribute whose name begins with `on`. A file that
  fails is answered `{"error":{"kind":"icon",…}}`, never its bytes.
- `chat` — an agent's image only. `false`, or `{"prompt_modalities": {"image": …, "audio": …, "file": …}}`:
  what a message to the agent may carry beside text, all three booleans required. Text is always
  taken and is not listed. `file` is an embedded resource that is neither an image nor audio.
- `structured_decisions` — an agent's image only. `false`, or
  `{"prompt_modalities": {…}, "instruction_modalities": {…}, "choice_modalities": {…}}`, each the same
  three booleans: what the decision's `content`, its questions' `instructions`, and its choices (option
  descriptions and score levels) may carry beside text.
- An agent's image declares at least one of the two as an object, or its document is invalid; a
  tool's image must not state either.
- **The document is required.** `agents create`, `standalone-tools create` and an agent's dependency
  declaration each read the image's document and refuse an image that ships none, that nobody in
  reach holds, or whose document does not parse. An agent is also refused when its image's document
  is a tool's, a standalone tool when its image's document is not a `standalone_tool`'s, and a
  dependency tool when its image's document is not a `dependency_tool`'s. A template or declaration
  that names a secret the document does not state is refused the same way. A template is not checked: `templates create` takes any
  image.
- **Both are enforced** on `agents message`: `chat` for a request of type `chat`, and
  `structured_decisions` for one of type `structured_decisions` (see `diverge-agents`). A request for a mode the image says
  `false` to is refused by the daemon and never reaches the container.
- **What a program is handed.** Its `POST /run` body is tagged: `{"type":"chat","messages":[…]}` or
  `{"type":"structured_decisions","key":…,"content":[…],"questions":[…]}`. A program that declares
  structured decisions answers that run by echoing the decision as one `user_structured_decisions` chunk,
  saying its answers in one `structured_decisions_response` chunk, and ending the stream. Its answers are
  probabilities only — `{"noul":n}`, `{"probabilities":{"<option value>":n,…}}` for a choice,
  `{"probabilities":[n,…]}` for a score, one per level in order. It names no `choice` and states no
  `score`: a `score` member makes the chunk malformed, and the daemon computes both for the log. `POST /enqueue`
  is chat only: the proxy never enqueues a decision, and hands a program nothing while a decision's run is
  open.
- **Echo what you take.** A program says a request back as it takes it up: a chat message as its user
  parts, a decision as its `user_structured_decisions` chunk, each under the `key` it was handed. The echo
  is how the daemon logs the request with its sender, and how a sender watching (`"watch":true`) finds its
  request in the log. A request accepted and never echoed is answered `"dropped"` to a watcher.
- The body states, in prose, the tools the image serves and the mode (`fixed` or `rotating`) its
  program takes each secret in. The frontmatter's `secrets` lists every secret the image states. Read
  both before writing a template.

The images in this repository generate their schema from their Rust argument types; running the
image's program with `--schema` prints the same schema as the frontmatter carries.

## `diverge run images read`: can the daemon supply an image, and one ask of it

When: you have a digest and need the image's `DIVERGE.md` (its type, its argument schema, the memory and disk it needs, the secrets it states, a tool's mode and scope),
or you need to know whether any source the daemon reaches can supply the image at all. It is the one way
to an image's document. Its manifests, blobs and possession proof are on the same command and are rarely
needed by hand: they are how providers move images between each other.

```bash
# the document: the argument schema, or the prose
diverge run images read --info '{"image":{"digest":"sha256:…"}}'
diverge run images read --info --jq '.info? // empty | .schema' '{"image":{…}}'
diverge run images read --info --jq '.info? // empty | .body'   '{"image":{…}}'

# only the question: can it be supplied?
diverge run images read '{"image":{"digest":"sha256:…","references":[{"registry":"docker.io","name":"library/alpine"}]}}'

# which of these platforms can be supplied, and is the set whole?
diverge run images read '{"image":{"digest":"sha256:…"},"platforms":["linux/amd64","linux/arm64"]}'
# {"held":{"platforms":["linux/amd64"]}}
# {"held":{"platforms":["linux/arm64"]}}
# "settled"

# the image's parts
diverge run images read --manifest sha256:… '{"image":{…}}'
diverge run images read --blob sha256:… --out layer.bin '{"image":{…}}'
diverge run images read --prove-nonce <64 hex chars> [--platform sha256:…] '{"image":{…}}'
```

Body: `image` (required), `platforms` (optional: the platforms wanted, each `os/architecture`), `ignore` (optional list of `{issuer, subject}` identities of remote daemons
not to ask), `relay` (optional). The body is required here; there is no `{}` default that makes sense.
Run `diverge run images read request-schema` for the exact shape. Flags, at most one ask per invocation:

| flag | does |
|---|---|
| `--info` | prints the image's `DIVERGE.md`, parsed |
| `--manifest <DIGEST>` | prints the manifest under that digest the image reaches, with its media type |
| `--blob <DIGEST>` with `--out <PATH>` | writes that blob to the file (truncated first); `--blob` and `--out` require each other |
| `--prove-nonce <HEX>` | prints the proof over the image under that 64-hex-character nonce |
| `--platform <DIGEST>` | only with `--prove-nonce`: which manifest of an image index is meant |

Lines printed: first the daemon's answer; after a `{"held":…}`, any further `{"held":…}` lines and then
`"settled"`; then (only when an ask was given) the ask's answer. So `--info` prints `{"held":…}`,
`"settled"` and the document; the `--jq` filters above skip all but the last.

**Reading the `held` lines:**

- Each `{"held":{"platforms":[…]}}` is what one source added: platforms you asked for that no earlier line
  carried. No platform appears twice, and none you did not ask for.
- `"settled"` means the daemon has stopped looking. Add up the `held` lines: if every platform you named is
  there, the image can be supplied whole; if some are missing, nobody the daemon reaches holds them.
- The daemon stops the moment every platform is found, without waiting for slower sources.
- With no `platforms` in the body, any platform will do: one `{"held":…}` listing everything the first
  source holds, then `"settled"`.

| line | meaning |
|---|---|
| `{"held":{"platforms":[…]}}` | a source can supply these platforms; the scope is served through every source found |
| `"settled"` | the daemon looks no further; no `held` line follows |
| `"not_found"` | no source the daemon asked holds any platform wanted |
| `"forbidden"` | the account holds neither `read` over any provider nor `relay` over any remote daemon record |
| `{"error":…}` | a failure; after a `held`, the source could not read the document, or it is not one |
| `{"info":{"type":"agent","name":…,"description":"…","schema":{…},"memory":N,"disk":N,"secrets":[{"name":…,"description":…}],"frontend":true,"chat":{…},"body":"…"}}` | the document, parsed (`type` is `agent`, `standalone_tool` or `dependency_tool`; `secrets` absent when none; `mode` in a tool's, `scope` in a dependency tool's; `frontend`, `chat` and `structured_decisions` each absent when false, the last two never in a tool's) |
| `"missing"` | the image ships no `/.diverge/DIVERGE.md` |
| `{"media_type":"…","body":"…"}` | the manifest asked for |
| `{"path":"…","bytes":N}` | the blob asked for, written |
| `{"proof":"<64 hex>"}` | the proof asked for |
| `"not_held"` | no source answered the ask: a digest the image does not reach, or a proof none would give |

Rules:

- **The daemon holds no image itself.** It opens the question on every connected provider that keeps a
  Store or lists a registry, and on every remote daemon it records, all at once. Each source that adds a
  platform wanted is kept; the rest are cancelled as soon as every platform is found.
- **A provider holds an image by the rules it deploys one by**: the image is in the asking daemon's
  Store there, or a registry the provider lists serves it under one of the image's references. An image
  that is only in a provider's cache is not held, and has no readable document.
- **Only sources the caller's grants reach are asked**: a provider where a `providers` grant holds `read`
  over it, a remote daemon where a `daemons` grant holds `relay` over its record. See `diverge-access`.
  A remote daemon is asked as the account this daemon holds there, which needs its own grants on that
  daemon too.
- **The document is recorded.** Once the daemon has read a document for a digest it keeps it (a digest
  never changes what it ships) and answers `--info` from its record, asking the source nothing. Only a
  document is recorded: a `"missing"` and an `{"error":…}` are asked again. The image must still be
  held somewhere for the scope to open.
- **No container is asked.** A document is read out of the image, never from a running container of it.
- **`agents create`, `standalone-tools create` and a dependency's deployment read the document
  themselves**, from the record or from any source the daemon reaches, whatever the caller's grants,
  and refuse an image with none. The first create of a new image can take as long as a provider needs
  to pull it.
- A remote daemon in `ignore`, or the daemon itself, is never asked, so rings of daemons end. Leave
  `ignore` out.
- The CLI cancels the scope after the one ask; each invocation is a fresh question.

Grant: `read` over providers and `relay` over daemons, for example
`{"providers":{"actions":["read"],"within":"any"}}`. There is no grant over images for it, and no `read`
over daemons: `relay` is the one grant to use a remote daemon.

## The images in this repository

Each image's directory holds its `Containerfile` and its `DIVERGE.md`. Build any of them as shown
above with `-f <dir>/Containerfile .`.

| image dir | `name` | kind | required `arguments` | secrets the image states |
|---|---|---|---|---|
| `diverge-agents/python` | `python` | agent, no model | `python` | none |
| `diverge-agents/lone` | `lone` | agent, local LoNE bundle (macOS) | `bundle` | none |
| `diverge-agents/openrouter` | `openrouter` | agent | `model` | one: `OPENROUTER_API_KEY` (fixed) |
| `diverge-agents/codex` | `codex` | agent | `model`, `auth` | four, of which the program takes one, by `auth.method`: `OPENAI_API_KEY`, `CODEX_ACCESS_TOKEN`, `OPENAI_IDENTITY_TOKEN` (fixed) or `CODEX_AUTH` (rotating) |
| `diverge-agents/hermes` | `hermes` | agent | `provider`, `model` | 51, one per provider credential and toolset credential; which the program takes is selected by the provider and the toolsets chosen |
| `diverge-agents/cc` | `cc` | agent (Claude Code) | `model` | none |
| `diverge-examples/tools/courier` | `courier` | `dependency_tool`: eager, `global` | `{}` | none |
| `diverge-examples/tools/taskboard` | `taskboard` | `dependency_tool`: eager, `template` | `{}` | none |
| `diverge-examples/tools/memory` | `memory` | `dependency_tool`: lazy, `instance` | `{}` | none |
| `diverge-examples/tools/alarm` | `alarm` | `dependency_tool`: eager, `global` | `{}` | none |
| `diverge-examples/tools/chat` | `chat` | `standalone_tool`: eager, has a frontend | `{}` | none |
| `diverge-tools/identity-diverge-network` | `identity-diverge-network` | `standalone_tool`: lazy, has a frontend | `{}` | none |
| `diverge-examples/agents/human` | `human` | agent, no model, has a frontend | `{}` | none |

Each agent image states `"memory":536870912,"disk":1073741824`. Each tool image states
`"memory":268435456,"disk":268435456`.

Full per-image detail — every argument, every secret with its mode and value shape, dependencies and
a minimal working template body:

- [references/agent-loop-images.md](references/agent-loop-images.md) — python, lone,
  openrouter, codex, cc, and the shared `dependencies` member.
- [references/hermes.md](references/hermes.md) — hermes: every provider id and its secrets, every
  toolset.
- [references/example-images.md](references/example-images.md) — courier, taskboard, memory, alarm,
  chat, human.

### How secrets work for an image (summary)

Four parties, four statements:

1. The **image's document** states every secret its program could ever take, whatever its arguments,
   each as `{"name": "…", "description": "…"}` in the frontmatter's `secrets`.
2. The **template** lists secret **names** (`"secrets":["OPENROUTER_API_KEY"]`): all of those the image
   states, or any subset of them, and none the image does not state. The member is required; `[]` is none.
3. The **agent or tool** made from it binds each name its template lists, and no other, to a **vault key**
   (`"secrets":{"OPENROUTER_API_KEY":"my-or-key"}` on its create).
4. The **program**, at its registration, declares which of those names it takes and each one's
   **mode**: `fixed` (a value read at every activation, never locked) or `rotating` (one value locked
   to the container while it is active and written back when it goes inactive).

A program may take fewer names than the template lists, never more: a name the program declares
that the template does not is a start error. Secrets are never arguments and never environment the
caller sets. The vault side (keys, slots, locks, `use` grant) is in `diverge-storage`.

### How dependencies work for an image (summary)

Every agent-loop image accepts a `dependencies` argument: a list of dependency tool templates the
daemon deploys and serves to the agent as MCP servers. Each entry is
`{"image":{…},"extra_memory":N,"extra_disk":N,"arguments":{…},"secrets":[…]}`, and its image must be a `dependency_tool` image. It states no `mode` and no `scope`: the tool image's own document does. A dependency's secrets are names the tool image states, resolved
through the **agent's** binding, so every name a dependency lists must also be a name the agent's
own record binds. Details and lifecycle are in `diverge-tools`; the entry shape is in
[references/agent-loop-images.md](references/agent-loop-images.md).

## Troubleshooting

| symptom | cause and fix |
|---|---|
| Start error "the provider pulls from no registry the image's references name" or "the image names no registry" | The image body has no `references` entry naming a registry the provider lists. Add one (`docker.io` for the default provider). |
| Start error saying the image is unavailable | The digest is not in the daemon's Store on that provider and no listed registry serves it. Build it with `diverge run images build` on that provider and use the digest it prints. |
| The image deploys but its program misbehaves against the proxy | The image predates the current inside-container contract. Rebuild it from current source. |
| Start error naming a secret the template does not list | The program's arguments select a secret (see the per-image reference) that the template's `secrets` omits. Add the name to the template (a new template id) and bind it on the create. |
| `images read` answers `"not_found"` for a local image | No connected provider has the image in the daemon's Store or serves it from a listed registry, and no remote daemon holds it. An image only in a provider's cache does not count. Build it on a connected provider with `diverge run images build`. |
| A rebuilt image has the same tag but templates still run the old one | Templates name digests. A rebuild is a new digest: make a new template with it. |

## Not verified

- `images list`, `images get` and `images push` are from the SDK, the daemon's handlers and the CLI; none
  has been run live. A push whose bytes a client supplies has no caller in this repository; a push fed
  from a provider or a daemon is the daemon's own piping, and has not been run either.
- The provider protocol also has `images::stat` over the caller's Store, which the daemon does not
  expose: the Containerfile an image was built from cannot be read back from the CLI.
- Running a container from a Store image whose template names no `references` is from the daemon's
  code (it asks who holds the digest when no registry matches), not run live.
- macOS and Linux build commands are from code and the setup report, not run live.
- `diverge-agents/cc` declares no secret and says Claude Code logs in by itself; how a login
  reaches the container (for example a mount of its credential directory) was not established.
- The `memory` / `disk` values the images state are starting points, not measured minimums.

## When this task is done

Load the `diverge` skill again before starting the next Diverge task: it is the index, and it says which
skill the next task needs.
