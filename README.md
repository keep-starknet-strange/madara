<!-- markdownlint-disable -->
<div align="center">
  <img src="docs/images/logo/PNGs/Madara logo - Red - Duotone.png" height="128">
</div>
<div align="center">
  <img src="docs/images/logo/PNGs/Madara wordmark - Red.png" height="128">
</div>
<div align="center">
<br />
<!-- markdownlint-restore -->

[![Project license](https://img.shields.io/github/license/keep-starknet-strange/madara.svg?style=flat-square)](LICENSE)
[![Pull Requests welcome](https://img.shields.io/badge/PRs-welcome-ff69b4.svg?style=flat-square)](https://github.com/keep-starknet-strange/madara/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
<a href="https://twitter.com/MadaraStarknet">
<img src="https://img.shields.io/twitter/follow/MadaraStarknet?style=social"/>
</a> <a href="https://github.com/keep-starknet-strange/madara">
<img src="https://img.shields.io/github/stars/keep-starknet-strange/madara?style=social"/>
</a> <a href="https://docs.madara.zone/">
<img src="https://img.shields.io/badge/Documentation-Website-yellow"/> </a>

<a href="https://www.youtube.com/playlist?list=PL1yL2_t7cTuJtzmMQWk4UZkmMpdNF-quN">
<img src="https://img.shields.io/badge/Community%20calls-Youtube-red?logo=youtube"/>
</a>

<a href="https://github.com/keep-starknet-strange/madara/blob/main/docs/contributor-starter-pack.md">
<img src="https://img.shields.io/badge/Contributor%20starter%20pack-Doc-green?logo=github"/>
</a>

<a href="https://keep-starknet-strange.github.io/madara/pallet_starknet/index.html">
<img src="https://img.shields.io/badge/Rust%20doc-%F0%9F%A6%80-pink?logo=rust"/>
</a>

<a href="https://keep-starknet-strange.github.io/madara/dev/bench/">
<img src="https://img.shields.io/badge/Benchmark-Performance-blue?logo=github-actions"/>
</a>

[![Exploration_Team](https://img.shields.io/badge/Exploration_Team-29296E.svg?&style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz48c3ZnIGlkPSJhIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxODEgMTgxIj48ZGVmcz48c3R5bGU+LmJ7ZmlsbDojZmZmO308L3N0eWxlPjwvZGVmcz48cGF0aCBjbGFzcz0iYiIgZD0iTTE3Ni43Niw4OC4xOGwtMzYtMzcuNDNjLTEuMzMtMS40OC0zLjQxLTIuMDQtNS4zMS0xLjQybC0xMC42MiwyLjk4LTEyLjk1LDMuNjNoLjc4YzUuMTQtNC41Nyw5LjktOS41NSwxNC4yNS0xNC44OSwxLjY4LTEuNjgsMS44MS0yLjcyLDAtNC4yN0w5Mi40NSwuNzZxLTEuOTQtMS4wNC00LjAxLC4xM2MtMTIuMDQsMTIuNDMtMjMuODMsMjQuNzQtMzYsMzcuNjktMS4yLDEuNDUtMS41LDMuNDQtLjc4LDUuMThsNC4yNywxNi41OGMwLDIuNzIsMS40Miw1LjU3LDIuMDcsOC4yOS00LjczLTUuNjEtOS43NC0xMC45Ny0xNS4wMi0xNi4wNi0xLjY4LTEuODEtMi41OS0xLjgxLTQuNCwwTDQuMzksODguMDVjLTEuNjgsMi4zMy0xLjgxLDIuMzMsMCw0LjUzbDM1Ljg3LDM3LjNjMS4zNiwxLjUzLDMuNSwyLjEsNS40NCwxLjQybDExLjQtMy4xMSwxMi45NS0zLjYzdi45MWMtNS4yOSw0LjE3LTEwLjIyLDguNzYtMTQuNzYsMTMuNzNxLTMuNjMsMi45OC0uNzgsNS4zMWwzMy40MSwzNC44NGMyLjIsMi4yLDIuOTgsMi4yLDUuMTgsMGwzNS40OC0zNy4xN2MxLjU5LTEuMzgsMi4xNi0zLjYsMS40Mi01LjU3LTEuNjgtNi4wOS0zLjI0LTEyLjMtNC43OS0xOC4zOS0uNzQtMi4yNy0xLjIyLTQuNjItMS40Mi02Ljk5LDQuMyw1LjkzLDkuMDcsMTEuNTIsMTQuMjUsMTYuNzEsMS42OCwxLjY4LDIuNzIsMS42OCw0LjQsMGwzNC4zMi0zNS43NHExLjU1LTEuODEsMC00LjAxWm0tNzIuMjYsMTUuMTVjLTMuMTEtLjc4LTYuMDktMS41NS05LjE5LTIuNTktMS43OC0uMzQtMy42MSwuMy00Ljc5LDEuNjhsLTEyLjk1LDEzLjg2Yy0uNzYsLjg1LTEuNDUsMS43Ni0yLjA3LDIuNzJoLS42NWMxLjMtNS4zMSwyLjcyLTEwLjYyLDQuMDEtMTUuOGwxLjY4LTYuNzNjLjg0LTIuMTgsLjE1LTQuNjUtMS42OC02LjA5bC0xMi45NS0xNC4xMmMtLjY0LS40NS0xLjE0LTEuMDgtMS40Mi0xLjgxbDE5LjA0LDUuMTgsMi41OSwuNzhjMi4wNCwuNzYsNC4zMywuMTQsNS43LTEuNTVsMTIuOTUtMTQuMzhzLjc4LTEuMDQsMS42OC0xLjE3Yy0xLjgxLDYuNi0yLjk4LDE0LjEyLTUuNDQsMjAuNDYtMS4wOCwyLjk2LS4wOCw2LjI4LDIuNDYsOC4xNiw0LjI3LDQuMTQsOC4yOSw4LjU1LDEyLjk1LDEyLjk1LDAsMCwxLjMsLjkxLDEuNDIsMi4wN2wtMTMuMzQtMy42M1oiLz48L3N2Zz4=)](https://github.com/keep-starknet-strange)

</div>

# ⚡ Madara App Chain Stack

Welcome to **Madara**, the modular stack to build chains using
[Cairo](https://book.cairo-lang.org/title-page.html) and the
[Starknet](https://www.starknet.io/) technology. Apps like dYdX V3, Immutable
and Sorare have been using StarkEx for scaling for a while and now with Madara,
it's open source for everyone to use.

Madara is built on the [Substrate](https://substrate.io/) framework which not
only makes it modular but also gives it access to years of dev tooling,
libraries and a strong developer community. It is specifically helpful if you
want to own more of the stack and get more control over your chain.

## 📚 Documentation

Get started with our comprehensive documentation, which covers everything from
project structure and architecture to benchmarking and running Madara:

- [Getting Started Guide](./docs/getting-started.md)
- [Architecture Overview](./docs/architecture.md)
- [Chain Genesis Information](./docs/genesis.md)
- [Project Structure](./docs/project-structure.md)
- [Run benchmark yourself](./benchmarking/README.md)

## 📣 Building App Chains

For many use cases, you do not need to fork this repo to build your app chain.
By adding changes using forking, you will have to periodically rebase (and solve
conflicts) to remain updated with the latest version of Madara. Madara by
default provides

- `pallet_starknet`: Adds the CairoVM to Substrate which allows you to deploy
  and execute Cairo contracts.
- `Starknet RPC`: Adds all the Starknet RPC calls to your chain so that it's
  compatible with all RPC tools like starknet-js, wallets, etc.
- `DA Interface`: A general interface which allows you to use any DA layer like
  `Avail`, `Celestia`, `Ethereum` etc.
- `Proving`: Running the Starknet OS which is the runtime logic in Cairo so that
  it can be proven on the L1.

So for many use cases where you want to change common things like

- Configuration
  [parameters](https://github.com/keep-starknet-strange/madara/blob/d90cb1a3ce5389f5f530ee5eaf3c3d4c96561b12/crates/runtime/src/pallets.rs#L33)
  for example block time, maximum steps etc.
- DA layer
- Genesis state
- Add new off chain workers
- Add new pallets

you don't need to fork the Madara repo. Instead, you can import the relevant
code as crates/pallets. We have created an
[app-chain-template](https://github.com/keep-starknet-strange/madara-app-chain-template)
which imports Madara as a library to show an example and would recommend you
start from here. For other more detailed use cases like

- Adding a new syscall to the cairo VM
- Changing the runtime logic to deviate from Starknet's logic

You should consider forking parts of Madara.

## 📣 Peripheral repositories

- [Madara Docsite](https://github.com/keep-starknet-strange/madara-docs): The
  source code of the Madara documentation website. Deployed on
  `https://docs.madara.zone`.
- [Stark Compass Explorer](https://github.com/lambdaclass/madara_explorer) by
  the [LambdaClass](https://lambdaclass.com/) team : An open source block
  explorer for Starknet based chains.
- [Madara Infra](https://github.com/keep-starknet-strange/madara-infra): A
  collection of scripts and tools to deploy and manage Madara on different
  environments (e.g. AWS, docker, ansible, etc.). It also contains the
  [Starknet Stack](https://github.com/keep-starknet-strange/madara-infra/blob/main/starknet-stack/docker-compose.yml)
  demo `docker-compose` file.
- [Madara Tsukuyomi](https://github.com/keep-starknet-strange/madara-tsukuyomi):
  The source code of the Madara Desktop App. A friendly GUI to start a Madara
  node and interact with it.
- [App Chain Template](<(https://github.com/keep-starknet-strange/madara-app-chain-template)>):
  A ready to use template that allows you to easily start an app chain.

## 🌟 Features

- Starknet sequencer 🐺
- Built on Substrate 🌐
- Rust-based for safety and performance 🏎️
- Custom FRAME pallets for Starknet functionality 🔧
- Comprehensive documentation 📚
- Active development and community support 🤝

## 🏗️ Build & Run

Want to dive straight in? Check out our
[Getting Started Guide](./docs/getting-started.md) for instructions on how to
build and run Madara on your local machine.

## Benchmarking

Benchmarking is an essential process in our project development lifecycle, as it
helps us to track the performance evolution of Madara over time. It provides us
with valuable insights into how well Madara handles transaction throughput, and
whether any recent changes have impacted performance.

You can follow the evolution of Madara's performance by visiting our
[Benchmark Page](https://keep-starknet-strange.github.io/madara/dev/bench/).

However, it's important to understand that the absolute numbers presented on
this page should not be taken as the reference or target numbers for a
production environment. The benchmarks are run on a self-hosted GitHub runner,
which may not represent the most powerful machine configurations in real-world
production scenarios.

Therefore, these numbers primarily serve as a tool to track the _relative_
performance changes over time. They allow us to quickly identify and address any
performance regressions, and continuously optimize the system's performance.

In other words, while the absolute throughput numbers may not be reflective of a
production environment, the relative changes and trends over time are what we
focus on. This way, we can ensure that Madara is always improving, and that we
maintain a high standard of performance as the project evolves.

One can use flamegraph-rs to generate flamegraphs and look for the performance
bottlenecks of the system by running the following :

```bash
./target/release/madara setup
flamegraph --root --open  -- ./target/release/madara --dev
```

In parallel to that, run some transactions against your node (you can use
[Gomu Gomu no Gatling](https://github.com/keep-starknet-strange/gomu-gomu-no-gatling)
benchmarker). Once you stop the node, the flamegraph will open in your browser.

## 🌐 Connect to the dev webapp

Once your Madara node is up and running, you can connect to our Dev Frontend App
to interact with your chain. [Connect here!](https://explorer.madara.zone/)

## 🤝 Contribute

We're always looking for passionate developers to join our community and
contribute to Madara. Check out our [contributing guide](./docs/CONTRIBUTING.md)
for more information on how to get started.

## 📖 License

This project is licensed under the **MIT license**.

See [LICENSE](LICENSE) for more information.

Happy coding! 🎉

## Contributors ✨

Thanks goes to these wonderful people
([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/abdelhamidbakhta"><img src="https://avatars.githubusercontent.com/u/45264458?v=4?s=100" width="100px;" alt="Abdel @ StarkWare "/><br /><sub><b>Abdel @ StarkWare </b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=abdelhamidbakhta" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/tdelabro"><img src="https://avatars.githubusercontent.com/u/34384633?v=4?s=100" width="100px;" alt="Timothée Delabrouille"/><br /><sub><b>Timothée Delabrouille</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=tdelabro" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/EvolveArt"><img src="https://avatars.githubusercontent.com/u/12902455?v=4?s=100" width="100px;" alt="0xevolve"/><br /><sub><b>0xevolve</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=EvolveArt" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/LucasLvy"><img src="https://avatars.githubusercontent.com/u/70894690?v=4?s=100" width="100px;" alt="Lucas @ StarkWare"/><br /><sub><b>Lucas @ StarkWare</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=LucasLvy" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/DavideSilva"><img src="https://avatars.githubusercontent.com/u/2940022?v=4?s=100" width="100px;" alt="Davide Silva"/><br /><sub><b>Davide Silva</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=DavideSilva" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.finiam.com/"><img src="https://avatars.githubusercontent.com/u/58513848?v=4?s=100" width="100px;" alt="Finiam"/><br /><sub><b>Finiam</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=finiam" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ZePedroResende"><img src="https://avatars.githubusercontent.com/u/17102689?v=4?s=100" width="100px;" alt="Resende"/><br /><sub><b>Resende</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=ZePedroResende" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/drspacemn"><img src="https://avatars.githubusercontent.com/u/16685321?v=4?s=100" width="100px;" alt="drspacemn"/><br /><sub><b>drspacemn</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=drspacemn" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/tarrencev"><img src="https://avatars.githubusercontent.com/u/4740651?v=4?s=100" width="100px;" alt="Tarrence van As"/><br /><sub><b>Tarrence van As</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=tarrencev" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://home.cse.ust.hk/~shanaj/"><img src="https://avatars.githubusercontent.com/u/47173566?v=4?s=100" width="100px;" alt="Siyuan Han"/><br /><sub><b>Siyuan Han</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=hsyodyssey" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://zediogoviana.github.io/"><img src="https://avatars.githubusercontent.com/u/25623039?v=4?s=100" width="100px;" alt="Zé Diogo"/><br /><sub><b>Zé Diogo</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=zediogoviana" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Matth26"><img src="https://avatars.githubusercontent.com/u/9798638?v=4?s=100" width="100px;" alt="Matthias Monnier"/><br /><sub><b>Matthias Monnier</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=Matth26" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/glihm"><img src="https://avatars.githubusercontent.com/u/7962849?v=4?s=100" width="100px;" alt="glihm"/><br /><sub><b>glihm</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=glihm" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/0xEniotna"><img src="https://avatars.githubusercontent.com/u/101047205?v=4?s=100" width="100px;" alt="Antoine"/><br /><sub><b>Antoine</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=0xEniotna" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://www.linkedin.com/in/clementwalter/"><img src="https://avatars.githubusercontent.com/u/18620296?v=4?s=100" width="100px;" alt="Clément Walter"/><br /><sub><b>Clément Walter</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=ClementWalter" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Eikix"><img src="https://avatars.githubusercontent.com/u/66871571?v=4?s=100" width="100px;" alt="Elias Tazartes"/><br /><sub><b>Elias Tazartes</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=Eikix" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/xJonathanLEI"><img src="https://avatars.githubusercontent.com/u/19556359?v=4?s=100" width="100px;" alt="Jonathan LEI"/><br /><sub><b>Jonathan LEI</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=xJonathanLEI" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/greged93"><img src="https://avatars.githubusercontent.com/u/82421016?v=4?s=100" width="100px;" alt="greged93"/><br /><sub><b>greged93</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=greged93" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/dubzn"><img src="https://avatars.githubusercontent.com/u/58611754?v=4?s=100" width="100px;" alt="Santiago Galván (Dub)"/><br /><sub><b>Santiago Galván (Dub)</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=dubzn" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ftupas"><img src="https://avatars.githubusercontent.com/u/35031356?v=4?s=100" width="100px;" alt="ftupas"/><br /><sub><b>ftupas</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=ftupas" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/phklive"><img src="https://avatars.githubusercontent.com/u/42912740?v=4?s=100" width="100px;" alt="Paul-Henry Kajfasz"/><br /><sub><b>Paul-Henry Kajfasz</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=phklive" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/chirag-bgh"><img src="https://avatars.githubusercontent.com/u/76247491?v=4?s=100" width="100px;" alt="chirag-bgh"/><br /><sub><b>chirag-bgh</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=chirag-bgh" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/danilowhk"><img src="https://avatars.githubusercontent.com/u/12735159?v=4?s=100" width="100px;" alt="danilowhk"/><br /><sub><b>danilowhk</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=danilowhk" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/bajpai244"><img src="https://avatars.githubusercontent.com/u/41180869?v=4?s=100" width="100px;" alt="Harsh Bajpai"/><br /><sub><b>Harsh Bajpai</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=bajpai244" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/amanusk"><img src="https://avatars.githubusercontent.com/u/7280933?v=4?s=100" width="100px;" alt="amanusk"/><br /><sub><b>amanusk</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=amanusk" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/dpinones"><img src="https://avatars.githubusercontent.com/u/30808181?v=4?s=100" width="100px;" alt="Damián Piñones"/><br /><sub><b>Damián Piñones</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=dpinones" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/marioiordanov"><img src="https://avatars.githubusercontent.com/u/102791638?v=4?s=100" width="100px;" alt="marioiordanov"/><br /><sub><b>marioiordanov</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=marioiordanov" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/dbejarano820"><img src="https://avatars.githubusercontent.com/u/58019353?v=4?s=100" width="100px;" alt="Daniel Bejarano"/><br /><sub><b>Daniel Bejarano</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=dbejarano820" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/sparqet"><img src="https://avatars.githubusercontent.com/u/37338401?v=4?s=100" width="100px;" alt="sparqet"/><br /><sub><b>sparqet</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=sparqet" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/robinstraub"><img src="https://avatars.githubusercontent.com/u/17799181?v=4?s=100" width="100px;" alt="Robin Straub"/><br /><sub><b>Robin Straub</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=robinstraub" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/edisontim"><img src="https://avatars.githubusercontent.com/u/76473430?v=4?s=100" width="100px;" alt="tedison"/><br /><sub><b>tedison</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=edisontim" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/lana-shanghai"><img src="https://avatars.githubusercontent.com/u/31368580?v=4?s=100" width="100px;" alt="lanaivina"/><br /><sub><b>lanaivina</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=lana-shanghai" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://droak.sh/"><img src="https://avatars.githubusercontent.com/u/5263301?v=4?s=100" width="100px;" alt="Oak"/><br /><sub><b>Oak</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=d-roak" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/rkdud007"><img src="https://avatars.githubusercontent.com/u/76558220?v=4?s=100" width="100px;" alt="Pia"/><br /><sub><b>Pia</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=rkdud007" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/apoorvsadana"><img src="https://avatars.githubusercontent.com/u/95699312?v=4?s=100" width="100px;" alt="apoorvsadana"/><br /><sub><b>apoorvsadana</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=apoorvsadana" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://ceccon.me/"><img src="https://avatars.githubusercontent.com/u/282580?v=4?s=100" width="100px;" alt="Francesco Ceccon"/><br /><sub><b>Francesco Ceccon</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=fracek" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ptisserand"><img src="https://avatars.githubusercontent.com/u/544314?v=4?s=100" width="100px;" alt="ptisserand"/><br /><sub><b>ptisserand</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=ptisserand" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/zizou0x"><img src="https://avatars.githubusercontent.com/u/111426680?v=4?s=100" width="100px;" alt="Zizou"/><br /><sub><b>Zizou</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=zizou0x" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/makluganteng"><img src="https://avatars.githubusercontent.com/u/74396818?v=4?s=100" width="100px;" alt="V.O.T"/><br /><sub><b>V.O.T</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=makluganteng" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/MdTeach"><img src="https://avatars.githubusercontent.com/u/19630321?v=4?s=100" width="100px;" alt="Abishek Bashyal"/><br /><sub><b>Abishek Bashyal</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=MdTeach" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/kariy"><img src="https://avatars.githubusercontent.com/u/26515232?v=4?s=100" width="100px;" alt="Ammar Arif"/><br /><sub><b>Ammar Arif</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=kariy" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/lambda-0x"><img src="https://avatars.githubusercontent.com/u/87354252?v=4?s=100" width="100px;" alt="lambda-0x"/><br /><sub><b>lambda-0x</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=lambda-0x" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/exp-table"><img src="https://avatars.githubusercontent.com/u/76456212?v=4?s=100" width="100px;" alt="exp_table"/><br /><sub><b>exp_table</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=exp-table" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Pilouche"><img src="https://avatars.githubusercontent.com/u/26655725?v=4?s=100" width="100px;" alt="Pilou"/><br /><sub><b>Pilou</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=Pilouche" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/hel-kame"><img src="https://avatars.githubusercontent.com/u/117039823?v=4?s=100" width="100px;" alt="hithem"/><br /><sub><b>hithem</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=hel-kame" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/clexmond"><img src="https://avatars.githubusercontent.com/u/706094?v=4?s=100" width="100px;" alt="Chris Lexmond"/><br /><sub><b>Chris Lexmond</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=clexmond" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Tidus91"><img src="https://avatars.githubusercontent.com/u/65631603?v=4?s=100" width="100px;" alt="Tidus91"/><br /><sub><b>Tidus91</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=Tidus91" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/nikania"><img src="https://avatars.githubusercontent.com/u/54669079?v=4?s=100" width="100px;" alt="Veronika S"/><br /><sub><b>Veronika S</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=nikania" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/0xAsten"><img src="https://avatars.githubusercontent.com/u/114395459?v=4?s=100" width="100px;" alt="Asten"/><br /><sub><b>Asten</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=0xAsten" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ben2077"><img src="https://avatars.githubusercontent.com/u/22720665?v=4?s=100" width="100px;" alt="ben2077"/><br /><sub><b>ben2077</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=ben2077" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/m-kus"><img src="https://avatars.githubusercontent.com/u/44951260?v=4?s=100" width="100px;" alt="Michael Zaikin"/><br /><sub><b>Michael Zaikin</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=m-kus" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.linkedin.com/in/jo%C3%A3o-pereira-91a087230/"><img src="https://avatars.githubusercontent.com/u/77340776?v=4?s=100" width="100px;" alt="João Pereira"/><br /><sub><b>João Pereira</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=joaopereira12" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/kasteph"><img src="https://avatars.githubusercontent.com/u/3408478?v=4?s=100" width="100px;" alt="kasteph"/><br /><sub><b>kasteph</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=kasteph" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ayushtom"><img src="https://avatars.githubusercontent.com/u/41674634?v=4?s=100" width="100px;" alt="Ayush Tomar"/><br /><sub><b>Ayush Tomar</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=ayushtom" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/tchataigner"><img src="https://avatars.githubusercontent.com/u/9974198?v=4?s=100" width="100px;" alt="tchataigner"/><br /><sub><b>tchataigner</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=tchataigner" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/kalaninja"><img src="https://avatars.githubusercontent.com/u/18083464?v=4?s=100" width="100px;" alt="Alexander Kalankhodzhaev"/><br /><sub><b>Alexander Kalankhodzhaev</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=kalaninja" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/antiyro"><img src="https://avatars.githubusercontent.com/u/74653697?v=4?s=100" width="100px;" alt="antiyro"/><br /><sub><b>antiyro</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=antiyro" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/azurwastaken"><img src="https://avatars.githubusercontent.com/u/30268138?v=4?s=100" width="100px;" alt="azurwastaken"/><br /><sub><b>azurwastaken</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=azurwastaken" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the
[all-contributors](https://github.com/all-contributors/all-contributors)
specification. Contributions of any kind welcome!
