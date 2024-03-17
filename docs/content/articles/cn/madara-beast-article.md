![thee BEEAAST](https://imgur.com/EBwBNnB.jpg)

# 驾驭巨兽 - Madara 和 Starknet 应用链的革新

**2023 年 7 月 20 日** · 1 分钟阅读时间

<font size=5>_探索区块链技术的未来, 从大幅降低成本到个性化控制_</font>

---

## 概述

- Madara 是一个高性能的 Starknet 排序器，提供了创建定制化和高<!--
  -->效[应用链](https://www.starknet.io/en/posts/ecosystem/the-starknet-stacks-growth-spurt)的<!--
  -->能力。
- 通过使用 Substrate 框架，Madara 强化了 Cairo VM 的能力，从而实现可证明、安全且
  灵活<!--
  -->的程序。
- 实施它可以带来诸多好处，譬如可扩展的基础设施、高吞吐量和对应用程序前所未有的
  控<!--
  -->制。
- Madara 有包括支持潜在的链上隐私、流畅的跨链互操作性以及强大的执行能力这类独特
  的<!--
  -->功能。
- Madara 正向区块链领域提供具有高效成本、可扩展和可定制的解决方案，来推动 dApp
  的开<!--
  -->发迈向前所未有的领域。

## 引言

试想一下，为你的应用程序的特殊需求来量身定制一条区块链——这正是应用链可以提供的
功<!--
-->能。应用链是面向特定应用程序的区块链，开发人员可以灵活调整链的各个方面，从而满
足<!--
-->其应用的需求，例如选择不同的哈希函数或自定义共识算法。最棒的是，由于应用链建
立在<!--
-->L1 或 L2 区块链之上，可以继承其强大的安全性，为开发人员提供了两全其美的解决方案。

介绍下 Madara，这是一个将灵活性和极速性能相结合的划时代的排序器。排序器这一组件
负<!--
-->责执行交易并将它们分组到批次中。作为通往属于你的 Starknet 应用链的入口，Madara
为在<!--
-->Starknet 生态系统中进行前所未有的实验开辟了广阔的可能性。

在我们深入探讨 Madara 如何为 Starknet 应用链带来强大的能力前，有必要解决一个问
题：为<!--
-->什么开发人员会选择在 Starknet 上构建应用链，而不是直接使<!--
-->用[Starknet 有效性 Rollups](https://starkware.co/resource/scaling-ethereum-navigating-the-blockchain-trilemma/#:~:text=top%20of%20them.-,Validity%20Rollups,-Validity%20rollups%2C%20also)。
有人可能会想，Starknet 是否已经足以应对大多数情况。

首先让我们了解下为什么应用链是 Starknet 生态系统中引人注目的扩展方式。

## 为什么选择应用链

Madara 是由 StarkWare 探索团队，也称<!--
-->为[Keep Starknet Strange](https://github.com/keep-starknet-strange)开发的，专门<!--
-->设计用于实现 StarkWare<!--
-->的[分形缩放](https://medium.com/starkware/fractal-scaling-from-l2-to-l3-7fe238ecfb4f)愿<!--
-->景。有许多令人信服的原因让开发人员选择创建一个 Starknet 应用链或 L3，而不是直
接依赖<!--
-->于 Starknet。

### 吞吐量

在现有的区块链基础设施中，应用开发人员在可扩展性上面临重大挑战。可扩展性包括两
个<!--
-->关键点：高速度和低费用。通过在每一层降低一千倍成本，开发人员可以显著降低从 L1
到 L3<!--
-->的整体成本，最高可达一百万倍。由于应用程序建立在其专用区块链上，从而无需与其
他<!--
-->应用竞争链上资源，吞吐量不受第三方应用活动的影响，这确保了持续平稳的流畅体验。

### 定制化

像 Starknet 和 Ethereum 等通用链采取了多项措施来确保网络对所有人可用，但这导致了
一种<!--
-->受限的环境。通过应用链，开发人员可以微调其应用和基础设施的各个方面，创建量身定
制<!--
-->的解决方案。不喜欢 Cairo VM 的某个特性？可以在你的应用链中将其排除掉。

### 创新

应用链的可定制性还允许开发人员可以使用目前在 Starknet 中不可用或存在风险的功能。
应<!--
-->用链赋予每个团队自主权，允许他们编写和授权任何所需的代码 hints。这使得应用链能
够<!--
-->解锁许多用例，譬如可以在不泄露个人隐私的情况下执行链上 KYC。

## Madare 对应用链堆栈的影响

一起来看看构成应用链的不同层级间的相互作用，以及 Madara 的用武之地。

1. **执行:** 执行层定义了区块的执行和状态差异的生成。Madara 提供了在两种执行工
   具<!--
   -->包（StarkWare 的 [blockifier](https://github.com/starkware-libs/blockifier)<!--
   -->和 LambdaClass 的<!--
   -->[starknet_in_rust](https://github.com/lambdaclass/starknet_in_rust)）之间切
   换的灵活性。无论选择了哪个执行工具包，底层框架都使用 Cairo VM。Cairo 语言有助于
   创建可证明的程序，这样就能证明计算被正确执行。
2. **结算:** 作为有效性 Rollup，Madara 应用链的状态可以仅通过检查其结算层来重
   建。通过在 Starknet L2 上更频繁的结算，L3 应用链可以实现更快的硬最终性，而去
   中心化的<!--
   -->排序层实现更强大的软最终性，因此，在这两方面(硬和软终结性)，结算都得到了增<!--
   -->强。
3. **排序:** Madara 负责排序过程，可以根据应用的需求进行调整，无论是简单的 FCFS
   或<!--
   -->PGA，还是像 Narwhall 和 Bullshark 这类更复杂的方案。一些应用链可以选择部署加
   密内<!--
   -->存池，以确保公平排序并减轻 MEV 的影响。
4. **数据可用性:** 数据可用性保证始终可访问完整的状态树，借此向用户提供信心，
   即<!--
   -->使 Madara 发生故障的情况下，他们也能证明自己拥有资产的所有权。Madara 将为开
   发者<!--
   -->提供多种可供选择的数据可用性方案。
5. **治理:** 每个 Madara 应用链可以选择其治理模
   型。[Snapshot X](https://twitter.com/SnapshotLabs)提供了一个依赖于存储证明
   并<!--
   -->完全基于链上的治理系统。其他治理机制也在探索中，譬如原生的 Substrate 治理面
   板。链上治理是 Madara 的核心价值所在。

![come come](https://lh4.googleusercontent.com/i7bXi2IPV-LTLzEgueA2SPHGULUFDj1OX4IznOQr5BeZe0hcey-VXA5TOV6q9XaVqBGAcYiie7u7uxw7q1ByZxjkPQKHERqKJTxhdDdTSgBQy8smyNO3jEHiNJv7Eqh8BMxjj4fFlQAW6gm-hQMzyIU)

## 进入: Madara

在 Madara 中，通过利用 Substrate 框架并整合 Cairo VM 来执行 Cairo 程序和
Starknet 智能合<!--
-->约，从而增强了 Cairo VM。Substrate 是一个开源 Rust 框架，以其灵活性而闻名，并用
于构<!--
-->建可定制的区块链。与此同时，Cairo VM 专门设计用于高效生成程序执行的有效性证
明。通<!--
-->过在 L2 上使用状态跟踪和智能合约来验证这些证明，应用链确保集成了 Starknet 的安
全性。这样，Madara 利用 Cairo 的强大功能实现了程序执行的可证明性。

Substrate 框架固有的模块化特性使开发者可以轻松地定制应用链。没有任何强加的假设，
允许你自行整合共识协议、哈希函数、签名方案、存储布局 - 无论你的应用需要什么，
都<!--
-->可以利用 Cairo 来生成证明。无论是 Starknet 还是 Ethereum 上，开发者都可以在继承
底层链<!--
-->安全性的同时，不受限制的操作，并可被证明。

起初，Madara 将与 Starknet 非常相似，使智能合约可以在 Starknet 生态系统内进行组
合。未<!--
-->来将有更宏伟的计划，因为 Starknet 将与[Herodotus](https://www.herodotus.dev/)集<!--
-->成，利用 [存储证明](https://book.starknet.io/chapter_8/storage_proofs.html)实
现<!--
-->互操作性。存储证明的整合还将使 Madara 应用链能够考虑来自其他链的状态和流动性。

准备好见证由 Madara 开启的 Starknet 新纪元吧。
