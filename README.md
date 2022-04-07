# Gitpod Community Discord Bot

This repo contains the code that runs the Gitpod Community Discord Bot. Initially a hackathon project built by [AXON](https://github.com/axonasif).

This bot does not use any traditional database structure but this could be improved at some point in the future. Currently, it is powered by a flat file database implementation.

Community contribuitions are welcome! 🧡 Please create an issue and open a Gitpod workspace from that context.

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/gitpod-io/optimus)

# Contributing

You wanna contribute!? That sounds awesome! Thank you for taking the step to contribute towards this project :)

## Getting started

> Creating the Bot application on Discord's dev portal
- Login on https://discord.com/developers/applications
- Create a new app by clicking on `New Application` on the top right
- Inside your bot page, click on 🧩 `Bot` from the left sidebar and then `Add Bot` button
    - In the same page, toggle on the following options: `Presence Intent`, `Server Members Intent` and `Message Content Intent`
- Go to **OAuth2 > URL Generator** from your left sidebar
    - Tick `Scopes: bot, application.commands` and `Bot permissions: Adminstrator`. It should look like below:
    ![OAuth2 example](/.assets/oauth2_example.png)
    - Scroll to the bottom of this page and copy paste the **GENERATED-URL** into your browser tab to add the bot to a discord server. I recommend creating a new Discord server for bot development perposes.

> Running the BOT from Gitpod

- Grab the token from your 🧩 `Bot` page on discord dev portal. You might need to reset it to see.
![bot token](/.assets/bot_token_example.png)
- Grab the **Application ID** from the `General Information` section in your left sidebar
- Get the **Guild ID**
    - In Discord app, open your User Settings by clicking the Settings Cog next to your user name on the bottom.
    - Go to `Appearance` and enable Developer Mode under the Advanced section, then close User Settings.
    - Right-click on your Discord server name where you invited the BOT, then select `Copy ID`
- In Gitpod terminal, run the BOT in the following manner:
```bash
DISCORD_TOKEN='yOuR.t0KeN.hErE' APPLICATION_ID='your-id-here-123456' GUILD_ID='your-discord-server-id-123456' cargo run
```
