# Optimus - Gitpod Community Discord Bot

This repo contains the code that runs the Gitpod Community Discord Bot.

Community contribuitions are welcome! 🧡 Please create an issue and open a Gitpod workspace from that context.

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/gitpod-io/optimus)

## Contributing

You wanna contribute!? That sounds awesome! Thank you for taking the step to contribute towards this project :)

## Getting started

### Creating a Bot application on Discord's dev portal

- Login on https://discord.com/developers/applications
- Create a new app by clicking on `New Application` on the top right
- Inside your bot page, click on 🧩 `Bot` from the left sidebar and then `Add Bot` button
  - In the same page, toggle on the following options: `Presence Intent`, `Server Members Intent` and `Message Content Intent`
- Go to **OAuth2 > URL Generator** from your left sidebar
  - Tick `Scopes: bot, application.commands` and `Bot permissions: Adminstrator`. It should look like below:
    ![OAuth2 example](/.assets/oauth2_example.png)
  - Scroll to the bottom of this page and copy paste the **GENERATED-URL** into your browser tab to add the bot to a discord server. I recommend creating a new Discord server for bot development purposes.

### Running the BOT from Gitpod for development

- If you don't have `./BotConfig.toml` file, create it by copying [./ExampleBotConfig.toml](./ExampleBotConfig.toml). So, you could run: `cp ExampleBotConfig.toml BotConfig.toml`

- Update/fill-up `./BotConfig.toml` with:
  - The token from your 🧩 `Bot` page on discord dev portal. You might need to reset it to see.
    ![bot token](/.assets/bot_token_example.png)
  - The **Application ID** from the `General Information` section in your left sidebar
- To persist the config changes accross development Gitpod workspaces:

```bash
# Run this every time after making changes to ./BotConfig.toml
gp env DISCORD_BOT_CONFIG_ENCODED="$(base64 -w0 BotConfig.toml)"
```

- In Gitpod terminal, run the BOT in the following manner:

```bash
cargo run
```

> **Note**
> You can also explicitly specify a path for your config.
> For example:

```bash
# From cargo
cargo run -- BotConfig.toml

# From a release binary
./target/release/optimus /some/arbitrary/path/to/your_config.toml
```

## Meilisearch, GitHub and OpenAI integration

Some (optional) features use Meilisearch, GitHub and OpenAI API.

Note: This part is undocumented, will be done soon.

## Deploying a release build to production server

Minimal resource requirements:

- optimus bot: 13-20MB RAM
- [optional] meilisearch: 100MB RAM (for indexing)

In conclusion, a server with 128MB RAM (+SWAP), shared CPU and 1GB storage will do.

### barebones

- WIP (will be written soon)

### systemd

- WIP (will be written soon)

### Docker

Docker would come handy in case you want something that JustWorks™️.

**Preparing the config:**

```bash
# Get the sample config
curl -o BotConfig.toml -L https://raw.githubusercontent.com/gitpod-io/optimus/main/ExampleBotConfig.toml

# Update the config with your bot token and application ID
vim BotConfig.toml # or nano, or any other editor
```

**Starting the container:**

```bash
# You can also start in detached mode by passing `-d` to `run` subcommand
docker run --init --name optimus -v $(pwd)/BotConfig.toml:/app/BotConfig.toml -t ghcr.io/gitpod-io/optimus:main
# You can press Ctrl+c to stop
# If using meilisearch, its database will be stored at `/app/data.ms` inside the `optimus` container
```

**Restarting the container:**

```bash
# Ensure it's stopped
docker container stop -t 2 optimus

# Starts and attach to it
docker container start -a optimus
# Press Ctrl+c to detach

# To attach back
docker attach optimus

# In case you want to stop again:
docker container stop -t 2 optimus
```

If you are hardware resource constrained, you can use the [barebones](#barebones) or [systemd](#systemd) method instead.

### Docker compose

The [docker](#docker) method is enough, using docker-compose for this would be overkill in terms of hardware resources 🌳

### GCP

- Create a **f1-micro** (~600MB RAM) Linux VM from **Compute Engine > VM Instances**. For example, with `gcloud` CLI:

```bash
gcloud compute instances create discord-optimus-bot \
    --zone=us-central1-a \
    --machine-type=f1-micro \
    --network-interface=network-tier=PREMIUM,subnet=default \
    --maintenance-policy=MIGRATE \
    --provisioning-model=STANDARD \
    --create-disk=auto-delete=no,boot=yes,device-name=discord-optimus-bot,image=projects/ubuntu-os-cloud/global/images/ubuntu-minimal-2204-jammy-v20230302,mode=rw,size=10 \
    --no-shielded-secure-boot \
    --no-shielded-vtpm \
    --no-shielded-integrity-monitoring \
    --labels=ec-src=vm_add-gcloud \
    --reservation-affinity=any
```

- WIP (will be written soon)

## Also see

- https://github.com/gitpod-io/gitpod-qa (AI powered chatbot)
