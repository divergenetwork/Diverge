---
{
  "type": "agent",
  "name": "hermes",
  "chat": {
    "prompt_modalities": {
      "image": false,
      "audio": false,
      "file": false
    }
  },
  "structured_decisions": false,
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Agent",
    "description": "An agent running against Hermes (Nous Research's agent harness).\n\nEvery field here is HONORABLE: it names a mechanism the container\nactually has â€” a `config.yaml` key its harness writes before\nHermes starts, or a per-request field on the run it drives â€” and\nnothing else made the cut. What Hermes cannot be told through its\nwire is not here: no skill preloading (skills arrive as mounts\nunder `/root/.hermes/external-skills/`, one directory per skill\nholding its `SKILL.md`, and Hermes indexes them itself â€” that\npath, not its own `skills/`, which it writes into), no\ntemperature (Hermes has no\nmain-agent sampling key), no thinking-token budget (the\n[`effort`](Self::effort) ladder is the whole reasoning\nvocabulary).\n\n# Secrets\n\nNo credential is a field. The provider and the toolsets SELECT\nsecrets by name â€” each structure's `SECRET:` paragraph says which,\nand in what mode â€” and the agent's template must declare every\nname selected. [`declaration`](Self::declaration) is that set, as\nthe registration answers it; the proxy feeds the values before\nevery loop, and collects the rotating ones after.",
    "type": "object",
    "properties": {
      "provider": {
        "description": "The inference source â€” Hermes is multi-provider by design,\nthis picks which, and the pick selects the secrets its auth\nneeds. See [`Provider`].",
        "$ref": "#/$defs/Provider"
      },
      "model": {
        "description": "The model to run, in the provider's own naming.",
        "type": "string"
      },
      "system_prompt": {
        "description": "The system prompt, applied per request.",
        "type": [
          "string",
          "null"
        ]
      },
      "effort": {
        "description": "Reasoning effort. Absent = the provider's own default;\n[`none`](Effort::None) = explicitly disabled â€” Hermes's own\ndisable sentinel, which is why absence and `none` are\ndifferent facts. See [`Effort`].",
        "anyOf": [
          {
            "$ref": "#/$defs/Effort"
          },
          {
            "type": "null"
          }
        ]
      },
      "toolsets": {
        "description": "Hermes's LOCAL capabilities, one switch each: an\n`Option<bool>` where a tool takes nothing but at most a\nsecret, that tool's own argument structure where it takes\nmore â€” absent is off either way, and the whole structure\nabsent is everything off. (The caller's MCP tools ride beside\nthese regardless, and the tool containers the agent asks for\nare [`dependencies`](Self::dependencies).) See [`Toolsets`].",
        "$ref": "#/$defs/Toolsets",
        "default": {}
      },
      "dependencies": {
        "description": "The tool containers this agent depends on, each a dependency\ntool template the caller deploys and serves to it as an MCP\nserver, exactly as the SDK's [`Template`] states one: that\ntype's JSON Schema is what `--schema` prints for this member,\nin `/.diverge/DIVERGE.md`, and this list, unchanged, is what the\nregistration answers â€” one schema, from the one type, at both.\nAbsent is none.",
        "type": "array",
        "items": {
          "$ref": "#/$defs/Template"
        }
      }
    },
    "required": [
      "provider",
      "model"
    ],
    "$defs": {
      "Provider": {
        "description": "The inference source.\n\nUntagged, discriminated by each variant's own `provider` constant\nâ€” the same discipline the [`Agent`](super::super::super::Agent)\nunion uses for `upstream`. One variant per provider because the\nargument sets genuinely differ: most are the discriminator alone,\ntheir one key a secret the variant declares by name, but bedrock\nnames its auth mode (which fixes its name set), vertex its\nproject and region, azure-foundry and custom carry their\nendpoints â€” and each secret applies through a different mechanism\nthe variant's own docs spell out.\n\nCLOSED at the pin, deliberately: user plugins can extend Hermes's\nregistry, but the container runs stock Hermes, so the bundled set\nis the whole vocabulary â€” minus `copilot-acp`, whose auth is not\nHermes's to hold (see [the module](super)). Rotating-OAuth\nproviders declare their state as `rotating` secrets rather than\n`fixed` ones; the module doc says what that means.\n\nHermes's own `auto` â€” provider detection from whatever\ncredentials happen to be present â€” is deliberately NOT here, and\nunder auth-as-secrets it is not even coherent: the feed carries\nexactly one provider's secrets, so there is nothing to detect.",
        "anyOf": [
          {
            "description": "See [`actual::Provider`](super::actual::Provider).",
            "$ref": "#/$defs/Provider2"
          },
          {
            "description": "See [`ai_gateway::Provider`](super::ai_gateway::Provider).",
            "$ref": "#/$defs/Provider3"
          },
          {
            "description": "See [`alibaba::Provider`](super::alibaba::Provider).",
            "$ref": "#/$defs/Provider4"
          },
          {
            "description": "See [`alibaba_coding_plan::Provider`](super::alibaba_coding_plan::Provider).",
            "$ref": "#/$defs/Provider5"
          },
          {
            "description": "See [`anthropic::Provider`](super::anthropic::Provider).",
            "$ref": "#/$defs/Provider6"
          },
          {
            "description": "See [`arcee::Provider`](super::arcee::Provider).",
            "$ref": "#/$defs/Provider7"
          },
          {
            "description": "See [`azure_foundry::Provider`](super::azure_foundry::Provider).",
            "$ref": "#/$defs/Provider8"
          },
          {
            "description": "See [`bedrock::Provider`](super::bedrock::Provider).",
            "$ref": "#/$defs/Provider9"
          },
          {
            "description": "See [`commandcode::Provider`](super::commandcode::Provider).",
            "$ref": "#/$defs/Provider10"
          },
          {
            "description": "See [`commandcode_anthropic::Provider`](super::commandcode_anthropic::Provider).",
            "$ref": "#/$defs/Provider11"
          },
          {
            "description": "See [`copilot::Provider`](super::copilot::Provider).",
            "$ref": "#/$defs/Provider12"
          },
          {
            "description": "See [`custom::Provider`](super::custom::Provider).",
            "$ref": "#/$defs/Provider13"
          },
          {
            "description": "See [`deepinfra::Provider`](super::deepinfra::Provider).",
            "$ref": "#/$defs/Provider14"
          },
          {
            "description": "See [`deepseek::Provider`](super::deepseek::Provider).",
            "$ref": "#/$defs/Provider15"
          },
          {
            "description": "See [`fireworks::Provider`](super::fireworks::Provider).",
            "$ref": "#/$defs/Provider16"
          },
          {
            "description": "See [`gemini::Provider`](super::gemini::Provider).",
            "$ref": "#/$defs/Provider17"
          },
          {
            "description": "See [`gmi::Provider`](super::gmi::Provider).",
            "$ref": "#/$defs/Provider18"
          },
          {
            "description": "See [`huggingface::Provider`](super::huggingface::Provider).",
            "$ref": "#/$defs/Provider19"
          },
          {
            "description": "See [`kilocode::Provider`](super::kilocode::Provider).",
            "$ref": "#/$defs/Provider20"
          },
          {
            "description": "See [`kimi_coding::Provider`](super::kimi_coding::Provider).",
            "$ref": "#/$defs/Provider21"
          },
          {
            "description": "See [`kimi_coding_cn::Provider`](super::kimi_coding_cn::Provider).",
            "$ref": "#/$defs/Provider22"
          },
          {
            "description": "See [`meta_ai::Provider`](super::meta_ai::Provider).",
            "$ref": "#/$defs/Provider23"
          },
          {
            "description": "See [`minimax::Provider`](super::minimax::Provider).",
            "$ref": "#/$defs/Provider24"
          },
          {
            "description": "See [`minimax_cn::Provider`](super::minimax_cn::Provider).",
            "$ref": "#/$defs/Provider25"
          },
          {
            "description": "See [`minimax_oauth::Provider`](super::minimax_oauth::Provider).",
            "$ref": "#/$defs/Provider26"
          },
          {
            "description": "See [`nebius_token_factory::Provider`](super::nebius_token_factory::Provider).",
            "$ref": "#/$defs/Provider27"
          },
          {
            "description": "See [`nous::Provider`](super::nous::Provider).",
            "$ref": "#/$defs/Provider28"
          },
          {
            "description": "See [`novita::Provider`](super::novita::Provider).",
            "$ref": "#/$defs/Provider29"
          },
          {
            "description": "See [`nvidia::Provider`](super::nvidia::Provider).",
            "$ref": "#/$defs/Provider30"
          },
          {
            "description": "See [`ollama_cloud::Provider`](super::ollama_cloud::Provider).",
            "$ref": "#/$defs/Provider31"
          },
          {
            "description": "See [`openai_codex::Provider`](super::openai_codex::Provider).",
            "$ref": "#/$defs/Provider32"
          },
          {
            "description": "See [`opencode_free::Provider`](super::opencode_free::Provider).",
            "$ref": "#/$defs/Provider33"
          },
          {
            "description": "See [`opencode_go::Provider`](super::opencode_go::Provider).",
            "$ref": "#/$defs/Provider34"
          },
          {
            "description": "See [`opencode_zen::Provider`](super::opencode_zen::Provider).",
            "$ref": "#/$defs/Provider35"
          },
          {
            "description": "See [`openrouter::Provider`](super::openrouter::Provider).",
            "$ref": "#/$defs/Provider36"
          },
          {
            "description": "See [`qwen_oauth::Provider`](super::qwen_oauth::Provider).",
            "$ref": "#/$defs/Provider37"
          },
          {
            "description": "See [`router::Provider`](super::router::Provider).",
            "$ref": "#/$defs/Provider38"
          },
          {
            "description": "See [`stepfun::Provider`](super::stepfun::Provider).",
            "$ref": "#/$defs/Provider39"
          },
          {
            "description": "See [`upstage::Provider`](super::upstage::Provider).",
            "$ref": "#/$defs/Provider40"
          },
          {
            "description": "See [`vertex::Provider`](super::vertex::Provider).",
            "$ref": "#/$defs/Provider41"
          },
          {
            "description": "See [`xai::Provider`](super::xai::Provider).",
            "$ref": "#/$defs/Provider42"
          },
          {
            "description": "See [`xiaomi::Provider`](super::xiaomi::Provider).",
            "$ref": "#/$defs/Provider43"
          },
          {
            "description": "See [`zai::Provider`](super::zai::Provider).",
            "$ref": "#/$defs/Provider44"
          }
        ]
      },
      "Provider2": {
        "description": "Actual Computer.\n\nAPPLICATION: the harness sets `ACTUAL_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `ACTUAL_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `actual`.",
            "$ref": "#/$defs/Actual"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Actual": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "actual"
        ]
      },
      "Provider3": {
        "description": "Vercel AI Gateway.\n\nAPPLICATION: the harness sets `AI_GATEWAY_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `AI_GATEWAY_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `ai-gateway`.",
            "$ref": "#/$defs/AiGateway"
          }
        },
        "required": [
          "provider"
        ]
      },
      "AiGateway": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "ai-gateway"
        ]
      },
      "Provider4": {
        "description": "Alibaba Cloud (DashScope).\n\nAPPLICATION: the harness sets `DASHSCOPE_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `DASHSCOPE_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `alibaba`.",
            "$ref": "#/$defs/Alibaba"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Alibaba": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "alibaba"
        ]
      },
      "Provider5": {
        "description": "Alibaba Cloud's dedicated coding tier.\n\nAPPLICATION: the harness sets `ALIBABA_CODING_PLAN_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n(`DASHSCOPE_API_KEY` is an accepted fallback in Hermes; the\nspecific var is the one set.)\n\nSECRET: `ALIBABA_CODING_PLAN_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `alibaba-coding-plan`.",
            "$ref": "#/$defs/AlibabaCodingPlan"
          }
        },
        "required": [
          "provider"
        ]
      },
      "AlibabaCodingPlan": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "alibaba-coding-plan"
        ]
      },
      "Provider6": {
        "description": "Anthropic, over the Messages API.\n\nAPPLICATION: the harness sets `ANTHROPIC_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `ANTHROPIC_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `anthropic`.",
            "$ref": "#/$defs/Anthropic"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Anthropic": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "anthropic"
        ]
      },
      "Provider7": {
        "description": "Arcee AI.\n\nAPPLICATION: the harness sets `ARCEEAI_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `ARCEEAI_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `arcee`.",
            "$ref": "#/$defs/Arcee"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Arcee": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "arcee"
        ]
      },
      "Provider8": {
        "description": "Microsoft Azure Foundry â€” a caller-supplied deployment, so the\nendpoint is as much the credential as the key is.\n\nAPPLICATION: the harness sets `AZURE_FOUNDRY_API_KEY` to the\nsecret's value and `AZURE_FOUNDRY_BASE_URL` to\n[`base_url`](Self::base_url) in the gateway's process environment\nbefore Hermes starts. Both are required: Hermes raises a hard\nauth error on a missing base URL. The Entra-ID mode is\ndeliberately absent â€” ambient Azure identity is not an argument.\n\nSECRET: `AZURE_FOUNDRY_API_KEY`, fixed â€” the API key, under that\nname.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `azure-foundry`.",
            "$ref": "#/$defs/AzureFoundry"
          },
          "base_url": {
            "description": "The deployment's endpoint, applied as\n`AZURE_FOUNDRY_BASE_URL`.",
            "type": "string"
          }
        },
        "required": [
          "provider",
          "base_url"
        ]
      },
      "AzureFoundry": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "azure-foundry"
        ]
      },
      "Provider9": {
        "description": "AWS Bedrock â€” AWS credentials, not a provider key.\n\nAPPLICATION: the harness sets the AWS SDK's own env vars in the\ngateway's process environment before Hermes starts, each to its\nsecret's value â€” see [`Auth`] for which â€” plus `AWS_REGION` to\n[`region`](Self::region) when present. These must be REAL\nprocess env: the botocore chain underneath never reads Hermes's\ndotenv.\n\nSECRET: the set [`auth`](Self::auth) fixes, every one fixed â€”\n`AWS_BEARER_TOKEN_BEDROCK` for `bearer-token`;\n`AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` for `access-key`;\nthose two and `AWS_SESSION_TOKEN` for `session-token`.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `bedrock`.",
            "$ref": "#/$defs/Bedrock"
          },
          "auth": {
            "description": "Which credential shape the secrets carry. See [`Auth`].",
            "$ref": "#/$defs/Auth"
          },
          "region": {
            "description": "The region, applied as `AWS_REGION`; absent = Hermes's own\ndefault (`us-east-1`).",
            "type": [
              "string",
              "null"
            ]
          }
        },
        "required": [
          "provider",
          "auth"
        ]
      },
      "Bedrock": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "bedrock"
        ]
      },
      "Auth": {
        "description": "One of AWS's static credential shapes, by name â€” which fixes the\nsecrets the provider declares. Kebab-case on the wire.\n\nThe rest of the botocore chain (profiles, container/IMDS roles,\nweb identity) is deliberately absent: those are facts about a\nmachine, and a request has no machine.",
        "oneOf": [
          {
            "description": "A Bedrock API key â€” the first credential the chain checks.\nDeclares `AWS_BEARER_TOKEN_BEDROCK`.",
            "type": "string",
            "const": "bearer-token"
          },
          {
            "description": "Static SigV4 credentials. Declares `AWS_ACCESS_KEY_ID` and\n`AWS_SECRET_ACCESS_KEY`.",
            "type": "string",
            "const": "access-key"
          },
          {
            "description": "Temporary SigV4 credentials. Declares `AWS_ACCESS_KEY_ID`,\n`AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN`.",
            "type": "string",
            "const": "session-token"
          }
        ]
      },
      "Provider10": {
        "description": "CommandCode.\n\nAPPLICATION: the harness sets `COMMANDCODE_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `COMMANDCODE_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `commandcode`.",
            "$ref": "#/$defs/Commandcode"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Commandcode": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "commandcode"
        ]
      },
      "Provider11": {
        "description": "CommandCode's Anthropic-dialect route.\n\nAPPLICATION: the harness sets `COMMANDCODE_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n(The same key as [`commandcode`](super::commandcode::Provider)\nâ€” one credential, two wires.)\n\nSECRET: `COMMANDCODE_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `commandcode-anthropic`.",
            "$ref": "#/$defs/CommandcodeAnthropic"
          }
        },
        "required": [
          "provider"
        ]
      },
      "CommandcodeAnthropic": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "commandcode-anthropic"
        ]
      },
      "Provider12": {
        "description": "GitHub Copilot (token exchange).\n\nAPPLICATION: the harness sets `COPILOT_GITHUB_TOKEN` to the\nsecret's value in the gateway's process environment before\nHermes starts; Hermes performs the GitHub-token â†’\nCopilot-api-token exchange on its own, and setting the var also\nsuppresses its `gh auth token` fallback.\n\nSECRET: `COPILOT_GITHUB_TOKEN`, fixed â€” the GitHub token the\nexchange starts from. Must be an OAuth or fine-grained token\n(`gho_`/`ghu_`/`github_pat_`) â€” Hermes rejects classic `ghp_`\nPATs.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `copilot`.",
            "$ref": "#/$defs/Copilot"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Copilot": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "copilot"
        ]
      },
      "Provider13": {
        "description": "Any OpenAI-compatible endpoint the caller names â€” the local\nruntimes' home (ollama, vllm, llama.cpp all alias onto it in\nHermes itself).\n\nAPPLICATION: the harness sets `CUSTOM_BASE_URL` to\n[`base_url`](Self::base_url) in the gateway's process environment\nbefore Hermes starts, and writes the same URL as `model.base_url`\nin the config it already owns. When [`api_key`](Self::api_key)\nis `true` the secret's value is written beside it as\n`model.api_key` â€” the config, not the environment, because\n`CUSTOM_API_KEY` is not a variable Hermes reads: the key is read\nfrom the config only, and only next to a configured URL. With no\nkey, Hermes supplies its own `no-key-required` placeholder â€” the\nkeyless case is the endpoint's business, not a protocol gap.\n\nSECRET: `CUSTOM_API_KEY`, fixed â€” the API key the endpoint\nexpects; declared only when [`api_key`](Self::api_key) is `true`,\nand none otherwise.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `custom`.",
            "$ref": "#/$defs/Custom"
          },
          "base_url": {
            "description": "The endpoint, applied as `CUSTOM_BASE_URL` and `model.base_url`.",
            "type": "string"
          },
          "api_key": {
            "description": "Whether the endpoint expects a key: `true` declares the\n`CUSTOM_API_KEY` secret, whose fed value becomes\n`model.api_key`; absent or `false` is the keyless endpoint.",
            "type": "boolean"
          }
        },
        "required": [
          "provider",
          "base_url"
        ]
      },
      "Custom": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "custom"
        ]
      },
      "Provider14": {
        "description": "DeepInfra.\n\nAPPLICATION: the harness sets `DEEPINFRA_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `DEEPINFRA_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `deepinfra`.",
            "$ref": "#/$defs/Deepinfra"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Deepinfra": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "deepinfra"
        ]
      },
      "Provider15": {
        "description": "DeepSeek.\n\nAPPLICATION: the harness sets `DEEPSEEK_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `DEEPSEEK_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `deepseek`.",
            "$ref": "#/$defs/Deepseek"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Deepseek": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "deepseek"
        ]
      },
      "Provider16": {
        "description": "Fireworks AI.\n\nAPPLICATION: the harness sets `FIREWORKS_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `FIREWORKS_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `fireworks`.",
            "$ref": "#/$defs/Fireworks"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Fireworks": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "fireworks"
        ]
      },
      "Provider17": {
        "description": "Google Gemini (AI Studio).\n\nAPPLICATION: the harness sets `GEMINI_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\nThe harness must NOT set `GOOGLE_API_KEY`, which Hermes\nchecks first.\n\nSECRET: `GEMINI_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `gemini`.",
            "$ref": "#/$defs/Gemini"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Gemini": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "gemini"
        ]
      },
      "Provider18": {
        "description": "GMI Cloud.\n\nAPPLICATION: the harness sets `GMI_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `GMI_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `gmi`.",
            "$ref": "#/$defs/Gmi"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Gmi": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "gmi"
        ]
      },
      "Provider19": {
        "description": "HuggingFace's inference router.\n\nAPPLICATION: the harness sets `HF_TOKEN` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `HF_TOKEN`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `huggingface`.",
            "$ref": "#/$defs/Huggingface"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Huggingface": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "huggingface"
        ]
      },
      "Provider20": {
        "description": "Kilo Code's gateway.\n\nAPPLICATION: the harness sets `KILOCODE_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `KILOCODE_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `kilocode`.",
            "$ref": "#/$defs/Kilocode"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Kilocode": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "kilocode"
        ]
      },
      "Provider21": {
        "description": "Moonshot's Kimi coding tier.\n\nAPPLICATION: the harness sets `KIMI_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `KIMI_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `kimi-coding`.",
            "$ref": "#/$defs/KimiCoding"
          }
        },
        "required": [
          "provider"
        ]
      },
      "KimiCoding": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "kimi-coding"
        ]
      },
      "Provider22": {
        "description": "Moonshot's Kimi coding tier, China endpoint.\n\nAPPLICATION: the harness sets `KIMI_CN_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `KIMI_CN_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `kimi-coding-cn`.",
            "$ref": "#/$defs/KimiCodingCn"
          }
        },
        "required": [
          "provider"
        ]
      },
      "KimiCodingCn": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "kimi-coding-cn"
        ]
      },
      "Provider23": {
        "description": "Meta's Model API (Muse Spark).\n\nAPPLICATION: the harness sets `MODEL_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `MODEL_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `meta-ai`.",
            "$ref": "#/$defs/MetaAi"
          }
        },
        "required": [
          "provider"
        ]
      },
      "MetaAi": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "meta-ai"
        ]
      },
      "Provider24": {
        "description": "MiniMax.\n\nAPPLICATION: the harness sets `MINIMAX_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `MINIMAX_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `minimax`.",
            "$ref": "#/$defs/Minimax"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Minimax": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "minimax"
        ]
      },
      "Provider25": {
        "description": "MiniMax, China endpoint.\n\nAPPLICATION: the harness sets `MINIMAX_CN_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `MINIMAX_CN_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `minimax-cn`.",
            "$ref": "#/$defs/MinimaxCn"
          }
        },
        "required": [
          "provider"
        ]
      },
      "MinimaxCn": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "minimax-cn"
        ]
      },
      "Provider26": {
        "description": "MiniMax over OAuth. Single-use rotating refresh tokens, so the\ncredential is a ROTATING secret: the caller holds its current\nstate, the run rotates it, and the rotated state is collected\nback instead of silently burning their login. (Keyed MiniMax is\n[`minimax`](super::minimax::Provider) /\n[`minimax_cn`](super::minimax_cn::Provider).)\n\nAPPLICATION: the harness writes the fed document as the\n`providers.minimax-oauth` entry of `$HERMES_HOME/auth.json`\nbefore Hermes starts â€” provider selection rides the config the\nharness already owns, never the file's `active_provider`. On\ntoken refresh Hermes rewrites the entry in place; after the\ngateway exits the entry is read back as the run left it and\nhanded to the proxy's collect. Nothing about it is an argument:\nchoosing this provider is the whole ask.\n\nSECRET: `MINIMAX_OAUTH`, rotating â€” the OAuth state document, a\nJSON object, exactly as Hermes keeps the entry.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `minimax-oauth`.",
            "$ref": "#/$defs/MinimaxOauth"
          }
        },
        "required": [
          "provider"
        ]
      },
      "MinimaxOauth": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "minimax-oauth"
        ]
      },
      "Provider27": {
        "description": "Nebius Token Factory.\n\nAPPLICATION: the harness sets `NEBIUS_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `NEBIUS_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `nebius-token-factory`.",
            "$ref": "#/$defs/NebiusTokenFactory"
          }
        },
        "required": [
          "provider"
        ]
      },
      "NebiusTokenFactory": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "nebius-token-factory"
        ]
      },
      "Provider28": {
        "description": "Nous Research â€” the Portal, Hermes's home team. OAuth with\nsingle-use rotating refresh tokens, so the credential is a\nROTATING secret: the caller holds its current state, the run\nrotates it, and the rotated state is collected back instead of\nsilently burning their login.\n\nAPPLICATION: the harness writes the fed document as the\n`providers.nous` entry of `$HERMES_HOME/auth.json` before Hermes\nstarts â€” provider selection rides the config the harness already\nowns, never the file's `active_provider`, which Hermes treats as\na last-resort fallback. On token refresh Hermes rewrites the\nentry in place; after the gateway exits the entry is read back as\nthe run left it and handed to the proxy's collect. Nothing about\nit is an argument: choosing this provider is the whole ask.\n\nSECRET: `NOUS_OAUTH`, rotating â€” the OAuth state document, a JSON\nobject, exactly as Hermes keeps the entry.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `nous`.",
            "$ref": "#/$defs/Nous"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Nous": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "nous"
        ]
      },
      "Provider29": {
        "description": "NovitaAI.\n\nAPPLICATION: the harness sets `NOVITA_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `NOVITA_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `novita`.",
            "$ref": "#/$defs/Novita"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Novita": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "novita"
        ]
      },
      "Provider30": {
        "description": "NVIDIA NIM.\n\nAPPLICATION: the harness sets `NVIDIA_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `NVIDIA_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `nvidia`.",
            "$ref": "#/$defs/Nvidia"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Nvidia": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "nvidia"
        ]
      },
      "Provider31": {
        "description": "Ollama Cloud (the hosted service, not a local ollama â€” that is\n[`custom`](super::custom::Provider)).\n\nAPPLICATION: the harness sets `OLLAMA_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `OLLAMA_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `ollama-cloud`.",
            "$ref": "#/$defs/OllamaCloud"
          }
        },
        "required": [
          "provider"
        ]
      },
      "OllamaCloud": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "ollama-cloud"
        ]
      },
      "Provider32": {
        "description": "The ChatGPT Codex backend, over external OAuth. Single-use\nrotating refresh tokens, so the credential is a ROTATING secret:\nthe caller holds its current state, the run rotates it, and the\nrotated state is collected back instead of silently burning\ntheir login.\n\nAPPLICATION: the harness writes the fed document as the\n`providers.openai-codex` entry of `$HERMES_HOME/auth.json` before\nHermes starts â€” provider selection rides the config the harness\nalready owns, never the file's `active_provider`. On token\nrefresh Hermes rewrites the entry in place; after the gateway\nexits the entry is read back as the run left it and handed to the\nproxy's collect. Nothing about it is an argument: choosing this\nprovider is the whole ask.\n\nSECRET: `OPENAI_CODEX_OAUTH`, rotating â€” the OAuth state\ndocument, a JSON object, exactly as Hermes keeps the entry.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `openai-codex`.",
            "$ref": "#/$defs/OpenaiCodex"
          }
        },
        "required": [
          "provider"
        ]
      },
      "OpenaiCodex": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "openai-codex"
        ]
      },
      "Provider33": {
        "description": "OpenCode's free tier â€” keyless by design.\n\nAPPLICATION: nothing. Hermes short-circuits this provider to an\nanonymous placeholder before any credential check, so the marker\nis the whole argument.\n\nSECRET: none.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `opencode-free`.",
            "$ref": "#/$defs/OpencodeFree"
          }
        },
        "required": [
          "provider"
        ]
      },
      "OpencodeFree": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "opencode-free"
        ]
      },
      "Provider34": {
        "description": "OpenCode Go.\n\nAPPLICATION: the harness sets `OPENCODE_GO_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `OPENCODE_GO_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `opencode-go`.",
            "$ref": "#/$defs/OpencodeGo"
          }
        },
        "required": [
          "provider"
        ]
      },
      "OpencodeGo": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "opencode-go"
        ]
      },
      "Provider35": {
        "description": "OpenCode Zen.\n\nAPPLICATION: the harness sets `OPENCODE_ZEN_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `OPENCODE_ZEN_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `opencode-zen`.",
            "$ref": "#/$defs/OpencodeZen"
          }
        },
        "required": [
          "provider"
        ]
      },
      "OpencodeZen": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "opencode-zen"
        ]
      },
      "Provider36": {
        "description": "OpenRouter.\n\nAPPLICATION: the harness sets `OPENROUTER_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\nHermes discards keys that do not start with `sk-or-`.\n\nSECRET: `OPENROUTER_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `openrouter`.",
            "$ref": "#/$defs/Openrouter"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Openrouter": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "openrouter"
        ]
      },
      "Provider37": {
        "description": "Qwen's portal, over OAuth. Single-use rotating refresh tokens,\nso the credential is a ROTATING secret: the caller holds its\ncurrent state, the run rotates it, and the rotated state is\ncollected back instead of silently burning their login.\n\nAPPLICATION: unlike the other OAuth providers this state is not\nHermes's own â€” it is the Qwen CLI's token file. The harness\nwrites the fed document to `~/.qwen/oauth_creds.json` at the REAL\nhome directory (Hermes hardcodes that path; it ignores\n`$HERMES_HOME`) before Hermes starts. On token refresh Hermes\nrewrites the creds file; after the gateway exits the file is read\nback as the run left it and handed to the proxy's collect.\nNothing about it is an argument: choosing this provider is the\nwhole ask.\n\nSECRET: `QWEN_OAUTH`, rotating â€” the CLI's token document, a JSON\nobject, exactly as the file holds it.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `qwen-oauth`.",
            "$ref": "#/$defs/QwenOauth"
          }
        },
        "required": [
          "provider"
        ]
      },
      "QwenOauth": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "qwen-oauth"
        ]
      },
      "Provider38": {
        "description": "Ramp Router.\n\nAPPLICATION: the harness sets `RAMP_ROUTER_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `RAMP_ROUTER_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `router`.",
            "$ref": "#/$defs/Router"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Router": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "router"
        ]
      },
      "Provider39": {
        "description": "StepFun's step plan.\n\nAPPLICATION: the harness sets `STEPFUN_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `STEPFUN_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `stepfun`.",
            "$ref": "#/$defs/Stepfun"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Stepfun": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "stepfun"
        ]
      },
      "Provider40": {
        "description": "Upstage Solar.\n\nAPPLICATION: the harness sets `UPSTAGE_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `UPSTAGE_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `upstage`.",
            "$ref": "#/$defs/Upstage"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Upstage": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "upstage"
        ]
      },
      "Provider41": {
        "description": "GCP Vertex AI â€” a service account, not an API key.\n\nAPPLICATION: the harness writes the secret's value to a file\nunder the Hermes home and sets `VERTEX_CREDENTIALS_PATH` to it in\nthe gateway's process environment before Hermes starts; access\ntokens are minted from it per call. `VERTEX_PROJECT_ID` and\n`VERTEX_REGION` are set from the optional fields when present.\nAmbient application-default credentials are deliberately absent\nâ€” those are facts about a machine, and a request has no machine.\n\nSECRET: `VERTEX_SERVICE_ACCOUNT_JSON`, fixed â€” the service-account key\ndocument, as a JSON STRING holding the document's text verbatim\n(not the document as an object): the string is what lands in the\nfile.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `vertex`.",
            "$ref": "#/$defs/Vertex"
          },
          "project_id": {
            "description": "The project, applied as `VERTEX_PROJECT_ID`; absent = the\nproject named inside the service-account document.",
            "type": [
              "string",
              "null"
            ]
          },
          "region": {
            "description": "The region, applied as `VERTEX_REGION`; absent = Hermes's own\ndefault (`global`).",
            "type": [
              "string",
              "null"
            ]
          }
        },
        "required": [
          "provider"
        ]
      },
      "Vertex": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "vertex"
        ]
      },
      "Provider42": {
        "description": "xAI.\n\nAPPLICATION: the harness sets `XAI_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `XAI_API_KEY`, fixed â€” the API key, under that name. The\n`x_search` toolset declares the same name; declared from both\nplaces it is one secret.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `xai`.",
            "$ref": "#/$defs/Xai"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Xai": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "xai"
        ]
      },
      "Provider43": {
        "description": "Xiaomi MiMo.\n\nAPPLICATION: the harness sets `XIAOMI_API_KEY` to the\nsecret's value in the gateway's process environment before\nHermes starts.\n\nSECRET: `XIAOMI_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `xiaomi`.",
            "$ref": "#/$defs/Xiaomi"
          }
        },
        "required": [
          "provider"
        ]
      },
      "Xiaomi": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "xiaomi"
        ]
      },
      "Provider44": {
        "description": "Z.AI (GLM / Zhipu).\n\nAPPLICATION: the harness sets `GLM_API_KEY` to the secret's value\nin the gateway's process environment before Hermes starts, and\n`GLM_BASE_URL` to [`base_url`](Self::base_url) when present.\n\nThe optional endpoint is the one such override in the vocabulary,\nand it earns its place: without a pinned base URL, Hermes's first\nuse fires a live probe across four candidate endpoints and caches\nthe winner to disk â€” a network round-trip at start, guessing at a\nfact the caller knows (which of Z.AI's global and CN surfaces\ntheir key belongs to).\n\nSECRET: `GLM_API_KEY`, fixed â€” the API key, under that name.",
        "type": "object",
        "properties": {
          "provider": {
            "description": "The discriminator. Always `zai`.",
            "$ref": "#/$defs/Zai"
          },
          "base_url": {
            "description": "The endpoint the key belongs to, applied as `GLM_BASE_URL`;\nabsent = Hermes's own endpoint probe.",
            "type": [
              "string",
              "null"
            ]
          }
        },
        "required": [
          "provider"
        ]
      },
      "Zai": {
        "description": "[`Provider`]'s discriminator.\n\nOne variant, and the reason the [union](super::Provider) can be\nuntagged: no other provider's arguments can produce this value.",
        "type": "string",
        "enum": [
          "zai"
        ]
      },
      "Effort": {
        "description": "Hermes's reasoning ladder, whole:\n`none < minimal < low < medium < high < xhigh < max < ultra`.\n\nDeliberately NOT the five-tier `Effort` the claude_code and\ncodex agents share: a different upstream owns a different\nvocabulary â€” [`none`](Self::None) and [`ultra`](Self::Ultra)\nexist only here, `none` being Hermes's explicit\nreasoning-disabled sentinel (which is why the agent's field is\nan `Option`: absent means the provider's default, `none` means\noff, and those are different facts). Hermes clamps to each\nwire's supported subset itself, always to the nearest WEAKER\nlevel â€” a caller never has to know which rungs a given model\nexposes.",
        "oneOf": [
          {
            "description": "Reasoning explicitly disabled.",
            "type": "string",
            "const": "none"
          },
          {
            "description": "Least.",
            "type": "string",
            "const": "minimal"
          },
          {
            "description": "Less.",
            "type": "string",
            "const": "low"
          },
          {
            "description": "Balanced.",
            "type": "string",
            "const": "medium"
          },
          {
            "description": "More.",
            "type": "string",
            "const": "high"
          },
          {
            "description": "Above high, below max.",
            "type": "string",
            "const": "xhigh"
          },
          {
            "description": "As much as the model will spend.",
            "type": "string",
            "const": "max"
          },
          {
            "description": "Above even max, where a wire supports it.",
            "type": "string",
            "const": "ultra"
          }
        ]
      },
      "Toolsets": {
        "description": "Per-toolset switches over Hermes's configurable checklist â€” the\nlist its own configuration wizard manages, closed at the pin,\nand narrowed to what a REQUEST can actually make work.\n\nTwo field shapes, by whether the toolset takes arguments. A\ntoolset with nothing to configure is an `Option<bool>` where\nABSENT IS OFF â€” the same as `false` â€” so nothing is on by\nomission and Hermes's own API-server defaults are never\nconsulted. A toolset with arguments is an `Option` of its own\nstructure â€” absent is off, present is the switch thrown on WITH\nthat tool's endpoint facts, each documented in its own file with\nthe mechanism the harness applies. A toolset whose only need is\na credential is a plain switch too: `true` declares the secret.\nEnabling a tool without its prerequisites is not a protocol\nerror â€” the tools simply fail as themselves when used.\n\nThese are the agent's LOCAL capabilities: what Hermes itself can\ndo beside the conversation. The caller's MCP tools are not in\nthis vocabulary and never need to be â€” each configured MCP\nserver becomes its own toolset in Hermes automatically, and the\ncontainer always wires the caller's in. The SKILLS toolset is\nnot here either, on purpose: whether the agent can browse skills\nfollows from whether the request mounted any, not from a switch\nthat could contradict that fact.\n\nWhat is deliberately ABSENT, and why, is [the module](super)'s\nto say.\n\nSECRET: `FAL_KEY`, fixed, when [`image_gen`](Self::image_gen) or\n[`video_gen`](Self::video_gen) is `true` (one secret for both);\n`XAI_API_KEY`, fixed, when [`x_search`](Self::x_search) is `true`\n(one secret with the xai provider's); and whatever the structured\ntoolsets declare, each in its own file.",
        "type": "object",
        "properties": {
          "web": {
            "description": "Web search and page extraction. See [`web`](super::web).",
            "anyOf": [
              {
                "$ref": "#/$defs/Toolset"
              },
              {
                "type": "null"
              }
            ]
          },
          "browser": {
            "description": "Browser automation. See [`browser`](super::browser).",
            "anyOf": [
              {
                "$ref": "#/$defs/Toolset2"
              },
              {
                "type": "null"
              }
            ]
          },
          "terminal": {
            "description": "Terminal and process control. Nothing to configure â€” the\ncontainer is the sandbox, and the local backend reads no\ncredentials.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "file": {
            "description": "File operations: read, write, patch, search. Nothing to\nconfigure.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "code_execution": {
            "description": "Code execution. Nothing of its own to configure â€” scripts\ncall other tools over RPC, and each called tool's own\narguments apply transitively.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "vision": {
            "description": "Image analysis. Nothing to configure â€” analysis rides the\nrun's own inference provider.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "video": {
            "description": "Video analysis. Nothing to configure â€” it rides the run's\nown inference provider, exactly as vision does.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "image_gen": {
            "description": "Image generation over FAL, the built-in path (Hermes's other\nimage providers â€” openai, krea, deepinfra, xai â€” are\ndeliberately absent at this pin). `true` declares `FAL_KEY`,\napplied in the gateway's process environment, and pins\n`image_gen.provider: fal`; generation has no keyless path, so\nthe switch and the secret are one ask.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "video_gen": {
            "description": "Video generation over FAL â€” the same deliberate one-provider\nscope as image generation. `true` declares `FAL_KEY` (the same\nsecret) and pins `video_gen.provider: fal`.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "x_search": {
            "description": "X (Twitter) search over the xAI API. `true` declares\n`XAI_API_KEY`, applied in the gateway's process environment.\nThe key path ONLY, deliberately: Hermes's own dispatch prefers\nthe key, and the xAI subscription OAuth bearer answers this\ntool in a degraded no-citation mode â€” the rotating path is a\nworse product here, not a missing feature.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "tts": {
            "description": "Text-to-speech. See [`tts`](super::tts).",
            "anyOf": [
              {
                "$ref": "#/$defs/Toolset3"
              },
              {
                "type": "null"
              }
            ]
          },
          "todo": {
            "description": "Task planning (todo). Nothing to configure â€” the store is\nin-memory and the run's own.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "session_search": {
            "description": "Searching past conversations. Nothing to configure â€” the\nlocal session database, credential-free. The database rides\nthe continuation whole, so the searchable past is the\nlineage's, not just this run's.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "memory": {
            "description": "Hermes's own memory: the two capped files, `MEMORY.md` (the\nagent's notes) and `USER.md` (the user profile), injected into\nthe system prompt at session start and edited by the `memory`\ntool. Nothing to configure â€” and the files ride the\ncontinuation, so what the tool writes is what the next run\nstarts with. (External memory-provider plugins are a separate\nconfig switch the harness leaves off.)",
            "type": [
              "boolean",
              "null"
            ]
          },
          "homeassistant": {
            "description": "Home Assistant control. See\n[`homeassistant`](super::homeassistant).",
            "anyOf": [
              {
                "$ref": "#/$defs/Toolset4"
              },
              {
                "type": "null"
              }
            ]
          },
          "spotify": {
            "description": "Spotify control. See [`spotify`](super::spotify).",
            "anyOf": [
              {
                "$ref": "#/$defs/Toolset5"
              },
              {
                "type": "null"
              }
            ]
          }
        }
      },
      "Toolset": {
        "description": "Web search and page extraction, with its arguments. The field\nbeing absent from [`Toolsets`](super::Toolsets) is Hermes's own\ndefault for this toolset; present is the switch thrown on.\n\nTwo roles, two slots: [`search`](Self::search) is what answers\nqueries, [`extract`](Self::extract) is what turns a page into\ntext â€” different tools, different backends, independently\nkeyed. Each slot names ONE backend, and naming a keyed one\ndeclares its secret; a slot left absent runs on Hermes's keyless\nfree-tier ring, which needs nothing â€” the keys buy quality and\nquota, not existence. Every combination is coherent, the empty\nobject included.\n\nSECRET: the named search backend's, fixed â€” `TAVILY_API_KEY`,\n`EXA_API_KEY`, `PARALLEL_API_KEY`, `KEENABLE_API_KEY` or\n`BRAVE_SEARCH_API_KEY` for [`Keyed`]'s five, none for SearXNG;\nand `FIRECRAWL_API_KEY`, fixed, for [`Extract::Firecrawl`].",
        "type": "object",
        "properties": {
          "search": {
            "description": "The search backend, one of [`Search`]'s; absent = the\nkeyless ring.",
            "anyOf": [
              {
                "$ref": "#/$defs/Search"
              },
              {
                "type": "null"
              }
            ]
          },
          "extract": {
            "description": "The extraction backend, one of [`Extract`]'s; absent = the\nkeyless ring.",
            "anyOf": [
              {
                "$ref": "#/$defs/Extract"
              },
              {
                "type": "null"
              }
            ]
          }
        }
      },
      "Search": {
        "description": "One search backend. Untagged â€” a keyed backend is its name as a\nJSON string, a self-hosted SearXNG is an object carrying its\naddress â€” and a string and an object never collide, so no marker\nis needed.\n\nOne backend, deliberately: Hermes can pool several keys as\nfallbacks for each other, but a vocabulary where extra keys\nquietly change behavior is the environment-dependent meaning\nthis module keeps rejecting. The caller names the backend; the\nkeyless ring remains the fallback story. The harness applies the\nkeyed backend's secret in the gateway's process environment, or\nSearXNG's address, and pins `web.search_backend` to the named\nprovider in the config it owns.",
        "anyOf": [
          {
            "description": "A keyed backend, by name; its key is the secret [`Keyed`]\nnames.",
            "$ref": "#/$defs/Keyed"
          },
          {
            "description": "A self-hosted SearXNG instance, applied as `SEARXNG_URL` â€”\nthe one backend whose argument is an address, and no secret.",
            "type": "object",
            "properties": {
              "searxng_url": {
                "description": "The instance.",
                "type": "string"
              }
            },
            "required": [
              "searxng_url"
            ]
          }
        ]
      },
      "Keyed": {
        "description": "The keyed search backends, by Hermes's own plugin names.\nKebab-case on the wire.\n\nSECRET: one per variant, fixed, applied in the gateway's process\nenvironment under its own name â€” as each variant says.",
        "oneOf": [
          {
            "description": "Tavily. Declares `TAVILY_API_KEY`.",
            "type": "string",
            "const": "tavily"
          },
          {
            "description": "Exa. Declares `EXA_API_KEY`.",
            "type": "string",
            "const": "exa"
          },
          {
            "description": "Parallel. Declares `PARALLEL_API_KEY`.",
            "type": "string",
            "const": "parallel"
          },
          {
            "description": "Keenable. Declares `KEENABLE_API_KEY`.",
            "type": "string",
            "const": "keenable"
          },
          {
            "description": "Brave, on its free plan. Declares `BRAVE_SEARCH_API_KEY`.",
            "type": "string",
            "const": "brave-free"
          }
        ]
      },
      "Extract": {
        "description": "One extraction backend, by Hermes's own plugin name â€”\nsingle-variant today because Hermes has one keyed extractor at\nthis pin. Kebab-case on the wire. The harness applies the secret\nand pins `web.extract_backend`.\n\nSECRET: `FIRECRAWL_API_KEY`, fixed, for [`Firecrawl`](Self::Firecrawl).",
        "oneOf": [
          {
            "description": "Firecrawl. Declares `FIRECRAWL_API_KEY`.",
            "type": "string",
            "const": "firecrawl"
          }
        ]
      },
      "Toolset2": {
        "description": "Browser automation, with its arguments. The field being absent\nfrom [`Toolsets`](super::Toolsets) is Hermes's own default for\nthis toolset; present is the switch thrown on.\n\nThe one slot is optional because the default backend is the\ncontainer's own headless Chromium, which needs nothing â€” the\nslot points the toolset at a browser somewhere else instead,\nand the empty object is the local browser, coherently.\n\nSECRET: `BROWSERBASE_API_KEY` and `BROWSERBASE_PROJECT_ID`, both\nfixed, when the remote is Browserbase; none for a raw CDP\nendpoint, and none for the local browser.",
        "type": "object",
        "properties": {
          "remote": {
            "description": "The remote backend, one of [`Remote`]'s; absent = the local\nheadless Chromium.",
            "anyOf": [
              {
                "$ref": "#/$defs/Remote"
              },
              {
                "type": "null"
              }
            ]
          }
        }
      },
      "Remote": {
        "description": "One remote browser backend. Untagged â€” Browserbase is its name\nas a JSON string, a raw endpoint is an object carrying its\naddress â€” and a string and an object never collide, so no marker\nis needed. One slot for both because they are the same role:\nBrowserbase IS a CDP provider, and a request naming a raw\nendpoint and a managed one would be contradicting itself.",
        "anyOf": [
          {
            "description": "Browserbase, by name. Declares `BROWSERBASE_API_KEY` and\n`BROWSERBASE_PROJECT_ID` â€” the pair is one credential, both\nsecrets, applied in the gateway's process environment â€” and\npins `browser.cloud_provider: browserbase`.",
            "$ref": "#/$defs/Browserbase"
          },
          {
            "description": "Any browser reachable over CDP, applied as\n`BROWSER_CDP_URL`. No secret.",
            "type": "object",
            "properties": {
              "cdp_url": {
                "description": "The endpoint.",
                "type": "string"
              }
            },
            "required": [
              "cdp_url"
            ]
          }
        ]
      },
      "Browserbase": {
        "description": "[`Remote::Browserbase`]'s name: the one string the slot takes.",
        "type": "string",
        "enum": [
          "browserbase"
        ]
      },
      "Toolset3": {
        "description": "Text-to-speech, with its arguments. The field being absent from\n[`Toolsets`](super::Toolsets) is Hermes's own default for this\ntoolset; present is the switch thrown on.\n\nThe ElevenLabs switch is OPTIONAL by the keyless-floor rule:\nHermes's default voice is edge-tts, which needs nothing â€” the\nempty object is speech on the free voice, coherently. The key is\nthe upgrade, not the existence.\n\nSECRET: `ELEVENLABS_API_KEY`, fixed, when\n[`elevenlabs`](Self::elevenlabs) is `true`; none otherwise.",
        "type": "object",
        "properties": {
          "elevenlabs": {
            "description": "ElevenLabs as the voice: `true` declares `ELEVENLABS_API_KEY`,\napplied in the gateway's process environment, and the harness\nselects the provider (`tts.provider: elevenlabs` in the config\nit owns). Absent or `false` is the free voice. Hermes's other\nkeyed voices are deliberately absent at this pin.",
            "type": "boolean"
          }
        }
      },
      "Toolset4": {
        "description": "Home Assistant control, with its arguments. The field being\nabsent from [`Toolsets`](super::Toolsets) is Hermes's own\ndefault for this toolset; present is the switch thrown on.\n\nThe url is REQUIRED here even though Hermes treats it as\noptional: its fallback is `homeassistant.local`, an mDNS name\nthat is dead inside a container â€” an instance the caller does\nnot name is an instance this run cannot reach.\n\nSECRET: `HASS_TOKEN`, fixed â€” the long-lived access token, static\nby Home Assistant's own design, never rotated; applied in the\ngateway's process environment.",
        "type": "object",
        "properties": {
          "url": {
            "description": "The instance, applied as `HASS_URL` in the gateway's\nprocess environment.",
            "type": "string"
          }
        },
        "required": [
          "url"
        ]
      },
      "Toolset5": {
        "description": "Spotify control, with its arguments â€” back in the vocabulary,\nbecause its only blocker was rotation and rotation is what a\nrotating secret is for: Hermes refreshes the PKCE state on\neffectively every Spotify tool call and rewrites the store each\ntime, so the login is fed in before the loop and its rotated\nform is collected after. The field being absent from\n[`Toolsets`](super::Toolsets) is Hermes's own default for this\ntoolset; present is the switch thrown on.\n\nAPPLICATION: the harness sets `HERMES_SPOTIFY_CLIENT_ID` to\n[`client_id`](Self::client_id) in the gateway's process\nenvironment, and writes the fed login document as the\n`providers.spotify` entry of `$HERMES_HOME/auth.json` (Hermes\nkeeps this entry inert for provider selection by its own\ndesign); after the gateway exits the entry is read back as the\nrun left it and handed to the proxy's collect.\n\nSECRET: `SPOTIFY_OAUTH`, rotating â€” the login document, a JSON\nobject, exactly as Hermes keeps the entry.",
        "type": "object",
        "properties": {
          "client_id": {
            "description": "The caller's own Spotify app (PKCE â€” there is no secret,\nand Hermes ships no default app). Static and public, so a\nplain argument.",
            "type": "string"
          }
        },
        "required": [
          "client_id"
        ]
      },
      "Template": {
        "description": "One tool container an agent's program depends on: everything the\ncaller needs to deploy it, as fields and not as words. A tool\ncontainer depends on nothing.\n\nA [`Container`](crate::shared::containers::request::Container) is an\nimage, two limits, mounts and arguments. The image, the limits and\nthe arguments are here, member for member; beside them the secrets\nthe image needs, by name alone — which of them the tool's program\ntakes, and in what mode, is its own answer at registration, and\ntheir values are the caller's, each resolved through the vault key\nthe agent that spawns the tool binds the same name to, fed at every\nactivation. The mounts are not the caller's volumes — a volume is\nnamed in the caller's own listing, and a tool that named one would\nbe dictating the caller's storage — but, for a lazy tool of the\n`instance` scope alone,\nthe AGENT's own files and directories, each a path in the agent\ncontainer served live at a path in the tool container: the caller\nserves the agent container, a `containers::fuse_serve` of the path,\nand mounts the tool, a `fuse::mount` answered from it.\n[`mode`](Self::mode) says which of the two modes the dependency is —\na lazy tool, run while it is used, or an eager one the caller\nkeeps, each with a scope — see [`Mode`]. What the tool may ask\nof the caller is not here: the tool's own program declares it at its\nregistration, every time it starts, as a standalone tool's does —\nsee the `permissions` of the\n[`register`](crate::container_proxy::inside::register) answer.\n\nThere are no instructions. What a template cannot say — the provider\nthe tool runs on, the volumes it is given — is\nthe caller's, and the caller needs no prose to choose it. The\ndaemon's own tool templates are another thing: a record the daemon\nkeeps, named by a hash, made by a request; this is what an image\ndeclares and a caller deploys.\n\nThere is no name. A dependency is named by its ID: the lowercase\nhexadecimal SHA-256 of its [`canonical`](crate::shared::canonical)\nbytes — this template as compact JSON, absent members omitted, the\nimage's `references` absent too, every object key sorted at every\ndepth, the `arguments` and the `secrets` included, the names sorted\nbytewise, the `mode` and\nits members among the keys — sixty-four characters, which the caller\ncomputes over [`hashed`](Self::hashed). Two declarations with one id\nare one dependency, and a list that declares one twice is refused;\nthe same template deployed for the same agent, on whatever provider,\nis the same dependency, and the same template declared under\nthe same scope key, by whatever agent, is the same tool. The\ncaller labels the tool's MCP tools by the id when it merges lists,\nand the program knows which tool serves each by the keys under\n`_meta`, see [`shared::mcp`](crate::shared::mcp).",
        "type": "object",
        "properties": {
          "image": {
            "description": "The image: a digest and its references, as a\n`containers::tools::run` request names one.",
            "$ref": "#/$defs/Image"
          },
          "memory": {
            "description": "The memory the tool container needs, in bytes.",
            "type": "integer",
            "format": "uint64",
            "minimum": 0
          },
          "disk": {
            "description": "The bytes the tool container may write, in bytes.",
            "type": "integer",
            "format": "uint64",
            "minimum": 0
          },
          "arguments": {
            "description": "The arguments the tool container is registered with — the\n`arguments` of the run request that makes it, verbatim."
          },
          "secrets": {
            "description": "The secrets the image needs, by name, in the order the image\nstates them: names only. Which of them the tool's program takes\nand in what mode is the program's answer at its registration —\nsee [`secrets`](crate::shared::containers::secrets) — and the\nvalue of each is resolved through the vault key the agent the\ntool is spawned for binds the same name to, fed at every\nactivation. Inside the hash, sorted bytewise there. A name is\nnot empty, no two are alike, and every one is a name the\nregistering agent's template declares; a template that says\notherwise is the registration's error. Absent, the image needs\nnone.",
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        },
        "required": [
          "image",
          "memory",
          "disk",
          "arguments"
        ],
        "oneOf": [
          {
            "description": "A tool run while it is used: `\"mode\":\"lazy\"`.",
            "type": "object",
            "properties": {
              "scope": {
                "description": "How far the tool is shared, and its database with it: see\n[`Scope`]. Required.",
                "$ref": "#/$defs/Scope"
              },
              "fuse_file_mounts": {
                "description": "Files of the agent container served live into the tool\ncontainer, one each over FUSE: see [`Mount`]. The agent's\npath names a file; the tool's path is where it appears.\nOnly where `scope` is `instance`. Absent when empty.",
                "type": "array",
                "items": {
                  "$ref": "#/$defs/Mount"
                }
              },
              "fuse_directory_mounts": {
                "description": "Directories of the agent container served live into the\ntool container, one each over FUSE: see [`Mount`]. The\nagent's path names a directory, whose whole tree the tool\nsees; the tool's path is where it appears. Only where\n`scope` is `instance`. Absent when empty.",
                "type": "array",
                "items": {
                  "$ref": "#/$defs/Mount"
                }
              },
              "mode": {
                "type": "string",
                "const": "lazy"
              }
            },
            "required": [
              "mode",
              "scope"
            ]
          },
          {
            "description": "A tool the caller keeps: `\"mode\":\"eager\"`.",
            "type": "object",
            "properties": {
              "scope": {
                "description": "How far the tool is shared, and its database with it: see\n[`Scope`]. Required.",
                "$ref": "#/$defs/Scope"
              },
              "mode": {
                "type": "string",
                "const": "eager"
              }
            },
            "required": [
              "mode",
              "scope"
            ]
          }
        ]
      },
      "Image": {
        "description": "The image, by its manifest digest and the references under which\nits bytes may be fetched — what\n[`images::oci`](crate::provider::endpoints::images::oci::client::request::Frame)\nasks about, so an oci scope that came back held names an image a\nrun can ask for, with nothing to translate between them. The same\nvalue is the whole of the shared\n[`oci::has`](crate::shared::containers::oci::has) request a\nprovider asks a caller on a run, and the image every\n[`oci::prove`](crate::shared::containers::oci::prove) ask on a\nscope naming it is over.\n\n# The digest is the image\n\nA digest cannot be repointed at different content, so an image\nmeans the same bytes every time, wherever they come from. That is\nthe whole of what the wire asserts: the provider runs the image\nthe digest names and no other. The references are not the image:\nthe same bytes sit under different names on different registries,\nand an id that hashes an image hashes it with none — see\n[`hashed`](Self::hashed).\n\n# Where the bytes come from\n\nA provider looks in the caller's own Store first, by digest alone —\nthe images the caller\n[built](crate::provider::endpoints::images::build) or\n[pushed](crate::provider::endpoints::images::push) on it, and no\nother caller's. Failing that it asks any referenced registry its\nown policy lists, with the credential it holds for it, and ignores\na reference naming one it does not; and failing that it takes the\nimage from the caller — asking on the run scope whether the caller\nholds it, naming this image, and, when it does, for its manifest\nand blobs, then for the proof that it holds it, see\n[`oci`](crate::shared::containers::oci). The order, and which\nanswered, is stated on\n[`Source`](crate::shared::containers::response::Source), which the\nrun's answer carries. A caller need not hold what it names; one\nthat does answers so when asked.",
        "type": "object",
        "properties": {
          "digest": {
            "description": "The manifest digest, `<algorithm>:<hex>`.\n\nWhat identifies the image: a runtime hashes what it pulls, so\na source serving other bytes under it fails before anything\nruns.",
            "type": "string"
          },
          "references": {
            "description": "Where the bytes may be fetched, each a registry and a path\nthere: see [`Reference`]. Absent when there are none, and then\nthe provider has only what it holds and what the caller holds.",
            "type": "array",
            "items": {
              "$ref": "#/$defs/Reference"
            }
          }
        },
        "required": [
          "digest"
        ]
      },
      "Reference": {
        "description": "One place the bytes an image's digest names may be fetched from: a\nregistry, and the repository path there. A reference locates and\ndoes not identify — the digest does — so an image may carry none,\none or several, and two images with one digest are one image\nwhatever each lists. A provider asks a referenced registry only\nwhere its own policy lists that registry, with the credential it\nholds for it, and ignores a reference naming one it does not.",
        "type": "object",
        "properties": {
          "registry": {
            "description": "The registry's host, as a reference names it before its first\n`/`: `docker.io`, `ghcr.io`, `registry.example.com:5000`. No\nscheme and no path.",
            "type": "string"
          },
          "name": {
            "description": "The repository path at that registry — `library/nginx`,\n`myorg/myimage`. No host and no tag: the image is\n`<registry>/<name>@<digest>` there.\n\nIt lands in a reference by concatenation wherever a provider\npulls from, so a provider refuses one that is not a repository\npath and normalizes nothing.",
            "type": "string"
          }
        },
        "required": [
          "registry",
          "name"
        ]
      },
      "Mount": {
        "description": "A path of the agent container, mounted at a path of the tool\ncontainer over FUSE: source before destination, the order a mount\nreads in everywhere else. Which of the two it is — a file or a\ndirectory — is which list of the [`Template`](super::Template) it\nis on.\n\nThe caller serves the agent container's path live — a\n`containers::fuse_serve` of it, or of the directory holding a file —\nand makes the mount in the tool container as a FUSE mount answered\nfrom that FUSE serve, so every read, write, listing, removal, rename and\nnew directory the tool makes is one ask answered from the agent's\nown filesystem, where the agent sees it at once. Nothing is copied:\nthe two containers share the path for as long as both run.",
        "type": "object",
        "properties": {
          "agent_path": {
            "description": "Where it is in the agent container, as path components from the\nagent container's root — the shape every path in this crate\ntakes. No component is empty, `.` or `..`, and the path is not\nthe root: a file on the file list, a directory on the directory\nlist.",
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "tool_path": {
            "description": "Where it appears in the tool container, as path components from\nthe tool container's root, under the rules a\n[`Container`](crate::shared::containers::request::Container)\nstates for its mounts: not the root, no other mount's, inside\nno other mount's, and no other mount's inside it.",
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        },
        "required": [
          "agent_path",
          "tool_path"
        ]
      },
      "Scope": {
        "description": "What a dependency tool is one of: one per parent agent, one per\nparent agent template, or one for every agent that declares it,\nwhatever template they are made from. The scope names the tool —\nthe same template declared under the same scope key is the same\ntool, joined and not made again — and names its database scope\nwith it, so what a tool holds is shared exactly as far as the tool\nis. Both modes say one: see\n[`Mode`](super::Mode). Snake case on the wire: `\"instance\"`,\n`\"template\"`, `\"global\"`.",
        "oneOf": [
          {
            "description": "One tool per parent agent: the tool deployed for one agent is\nits own, with a database of its own, and two agents' tools of\nthis template never see each other's. The one scope under\nwhich a lazy tool mounts the agent's files.",
            "type": "string",
            "const": "instance"
          },
          {
            "description": "One tool per parent agent template: every agent made from the\nsame agent template that declares this template is attached\nto one tool, which holds one database, so what one writes the\nothers read.",
            "type": "string",
            "const": "template"
          },
          {
            "description": "One tool for the template, full stop: every agent that\ndeclares it, from whatever agent template, is attached to the\none tool, and its database is one for all of them.",
            "type": "string",
            "const": "global"
          }
        ]
      }
    }
  }
}
---
# Hermes

An agent container running a Hermes agent: `hermes gateway` laid down
from the arguments and driven over its API.

Secrets: the provider and the toolsets chosen select them by name;
each structure's `SECRET:` paragraph in the schema's descriptions
names its secret and its mode.
