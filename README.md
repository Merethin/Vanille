# Vanille

A manual recruitment bot, inspired by Asperta and older manual recruitment bots i.e. Scroll (and my own Moonlark, which is now discontinued and replaced by this project).

Named after RWBY's Neo (aka Trivia Vanille), as per Makasta's suggestion.

## Modes

Vanille has two recruitment modes:
- Oneshot: Simply gives you a list of nations to telegram when you press the button, using an ephemeral message, like Asperta does. The message is deleted shortly after your telegram cooldown expires.
- Stream: Starts a recruitment session and continuously sends nations to telegram through DMs. The minimum delay between telegrams can be a fixed time, or it can be automatic, calculated from your nation's age and the number of nations last telegrammed, with 10 seconds of buffer time added.

## Nations & Templates

The queue is updated live, with nations added as soon as they're founded / refounded, using SSE. Vanille supports both newfounds and refounds, and each user can use different templates for each (separating batches of nations to telegram depending on their origin), or a common template, in which case the user receives a mixed batch of nations to telegram.

Nations with names ending in numbers or roman numerals are excluded. Certain spawn regions can be filtered out on each individual queue.

Several templates for each category can be used, if you want to do A/B testing, in which case each batch will have a randomly picked template. If some of your templates are specific to either newfounds or refounds but you also have a common template, all mixed batches will pick the common template, and if you get a batch of just newfounds or just refounds, there will be a chance (!) for the specific templates to be picked, but the common template might get picked as well. Therefore, it's better to either have specific templates or joint templates, but not to mix both, as the specific templates will be used way less.

## Statistics

Vanille tracks certain data about every single telegram sent, including time the nation was added to the queue, region where it spawned, sender nation, time the telegram was sent at, telegram template, etc. for each recipient. If a nation that was sent a telegram moves to the queue's region, that is tracked as well, including the move event's timestamp.

These statistics can then be accessed by using the "CSV" export button on Vanille's statistics menu, opening the door to more advanced data analysis on recruitment. A traditional recruitment leaderboard is available as well.

## Reminders

Vanille can optionally send out pings in a separate channel if the queue reaches a certain amount of nations AND no telegrams have been sent in a certain amount of time. These pings are capped to a minimum 6-hour interval per queue, meaning that no matter what the criteria is or how many times it is reached, you won't get pings every 15 minutes.

## Requirements

A running PostgreSQL database (with tables already created, the code for them is in the [sql](sql/) folder) and a running [Akari](https://github.com/Merethin/Akari) instance connected to RabbitMQ.

## Configuration

The config file (located at `config/vanille.toml`) has two sections:

#### Input
```
[input]
url = "amqp://guest:guest@0.0.0.0:5672"
exchange_name = "akari_events"
```

For the input section, specify the URL of the RabbitMQ instance as well as the exchange name to listen for Akari events on.

#### Database
```
[database]
url = "postgres://postgres:postgres@127.0.0.1"
```
For the database section, specify the URL of the Postgres database to connect to.

## Setup

**Make sure to use the `--recursive` flag when cloning the repository or download submodules before building!**

Run `cargo build --release` to compile the program. You'll need a recent version of Rust.

Run it with `NS_USER_AGENT=[YOUR MAIN NATION NAME] ./target/release/vanille`.

Alternatively, you can set up a Docker container.

Building it: `docker build --tag vanille .`

Running it: `docker run -e NS_USER_AGENT=[YOUR MAIN NATION NAME] vanille`

Note: to pass your config file over to Vanille, you must bind mount the directory it is in:

`docker run -e NS_USER_AGENT=[YOUR MAIN NATION NAME] -v ./config:/config vanille`

Inside Docker, Vanille looks for the config file in `/config/vanille.toml`. If it isn't behaving like you expect, make sure the file is present/mounted in some way.