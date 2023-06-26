<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<div align="center">
  <img src="docs/images/madara-no-bg.png" height="256">
</div>

<div align="center">
<br />
<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

[![GitHub Workflow Status](https://github.com/keep-starknet-strange/madara/actions/workflows/test.yml/badge.svg)](https://github.com/keep-starknet-strange/madara/actions/workflows/test.yml)
[![Project license](https://img.shields.io/github/license/keep-starknet-strange/madara.svg?style=flat-square)](LICENSE)
[![Pull Requests welcome](https://img.shields.io/badge/PRs-welcome-ff69b4.svg?style=flat-square)](https://github.com/keep-starknet-strange/madara/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
<a href="https://twitter.com/MadaraStarknet">
<img src="https://img.shields.io/twitter/follow/MadaraStarknet?style=social"/>
</a>
<a href="https://github.com/keep-starknet-strange/madara">
<img src="https://img.shields.io/github/stars/keep-starknet-strange/madara?style=social"/>
</a>
</div>

# ⚡ Madara: Starknet Sequencer on Substrate 🦀

<a href="https://docs.madara.wtf/">
<img src="https://img.shields.io/badge/Documentation-Website-yellow"
 height="50" />
</a>

<a href="https://www.youtube.com/playlist?list=PL1yL2_t7cTuJtzmMQWk4UZkmMpdNF-quN">
<img src="https://img.shields.io/badge/Community%20calls-Youtube-red?logo=youtube"
 height="50" />
</a>

<a href="https://github.com/keep-starknet-strange/madara/blob/main/docs/contributor-starter-pack.md">
<img src="https://img.shields.io/badge/Contributor%20starter%20pack-Doc-green?logo=github"
 height="50" />
</a>

<a href="https://github.com/keep-starknet-strange/madara/blob/main/docs/madara-coding-principles.md">
<img src="https://img.shields.io/badge/Coding%20principles-Doc-green?logo=github"
 height="50" />
</a>

<a href="https://keep-starknet-strange.github.io/madara/pallet_starknet/index.html">
<img src="https://img.shields.io/badge/Rust%20doc-%F0%9F%A6%80-pink?logo=rust"
 height="50" />
</a>

<a href="https://keep-starknet-strange.github.io/madara/dev/bench/">
<img src="https://img.shields.io/badge/Benchmark-Performance-blue?logo=github-actions"
 height="50" />
</a>

Welcome to **Madara**, a blazing fast ⚡ [Starknet](https://www.starknet.io/) sequencer
 designed to make your projects soar!

Built on the robust Substrate framework and fast, thanks to Rust 🦀,
Madara delivers unmatched performance and scalability to power
 your Starknet-based Validity Rollup chain.

Dive into the world of Madara and join our passionate community of contributors!
Together, we're pushing the boundaries of what's possible within the Starknet ecosystem.

🚀 Discover the unparalleled flexibility and might of Madara,
your gateway to launching your very own Starknet appchain or L3.
Harness the prowess of Cairo, while maintaining complete control
over your custom appchain, tailored to your specific requirements.
Madara is designed to empower a multitude of projects, fueling
growth within the Starknet ecosystem.

## 🌟 Features

- Starknet sequencer 🐺
- Built on Substrate 🌐
- Rust-based for safety and performance 🏎️
- Custom FRAME pallets for Starknet functionality 🔧
- Comprehensive documentation 📚
- Active development and community support 🤝

## 📚 Documentation

Get started with our comprehensive documentation, which covers everything from
project structure and architecture to benchmarking and running Madara:

- [Architecture Overview](./docs/architecture.md)
- [Project Structure](./docs/project-structure.md)
- [Getting Started Guide](./docs/getting-started.md)
- [Run benchmark yourself](./benchmarking/README.md)

## 🏗️ Build & Run

Want to dive straight in? Check out our
[Getting Started Guide](./docs/getting-started.md) for instructions on how to
build and run Madara on your local machine.

## Benchmarking

Benchmarking is an essential process in our project development lifecycle,
as it helps us to track the performance evolution of Madara over time.
It provides us with valuable insights into how well Madara handles transaction throughput,
 and whether any recent changes have impacted performance.

You can follow the evolution of Madara's performance by visiting our [Benchmark Page](https://keep-starknet-strange.github.io/madara/dev/bench/).

However, it's important to understand that the absolute numbers presented
on this page should not be taken as the reference or target numbers
for a production environment.
The benchmarks are run on a self-hosted GitHub runner,
 which may not represent the most powerful machine configurations in real-world
 production scenarios.

Therefore, these numbers primarily serve as a tool to track
the _relative_ performance changes over time.
They allow us to quickly identify and address any performance regressions,
and continuously optimize the system's performance.

In other words, while the absolute throughput numbers may not be reflective of
 a production environment, the relative changes and trends over time
 are what we focus on. This way, we can ensure that Madara is always improving,
 and that we maintain a high standard of performance as the project evolves.

## 🌐 Connect to the dev webapp

Once your Madara node is up and running, you can connect to the Polkadot-JS Apps
front-end to interact with your chain.
[Connect here!](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944)

You can also connect to our customized fork of the Polkadot-JS Apps front-end,
 deployed on [Madara dev webapp](https://starknet-madara.netlify.app/#/explorer).

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
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the
[all-contributors](https://github.com/all-contributors/all-contributors)
specification. Contributions of any kind welcome!
