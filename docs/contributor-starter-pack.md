# Contributor Starter Pack for Madara

Welcome to the Contributor Starter Pack for Madara on Starknet!

Whether you're a blockchain enthusiast, a Rust developer, or simply curious
about contributing to open-source projects, this starter pack is designed to
provide you with the essential resources and guidance to dive into the exciting
world of blockchain development.

Madara, built in Rust and based on the Substrate framework, offers a unique
opportunity to contribute to the advancement of Ethereum scaling and
decentralized technologies.

This comprehensive starter pack will walk you through the foundational concepts
of blockchain, introduce you to Substrate, help you get started with Rust
programming, and provide insights to Starknet and Madara.

**Important note**: To ensure an enjoyable and confident journey, we encourage
you to dedicate time to explore the contents of this starter pack, empowering
you with the knowledge and skills needed to contribute effectively and to enjoy
the exciting process of being a Starknet builder!

## Table of Contents

1. [Contributor Mindset](#contributor-mindset)
2. [Learning Rust](#learning-rust)
3. [Understanding blockchain basics](#blockchain-basics)
4. [What is Starknet](#what-is-starknet)
5. [Substrate](#understanding-substrate)
   - [Client](#substrate-client)
   - [Runtime](#substrate-runtime)
   - [Primitives](#substrate-primitives)
6. [Madara](#madara-dive)
7. [OnlyDust contributions](#onlydust)

## Contributor Mindset <a name="contributor-mindset"/>

As a new contributor to the Madara project and more widely to the Starknet
ecosystem, it's important to adopt a positive and collaborative mindset. Here
are some key aspects of the mindset that can help you navigate your contribution
journey:

- _Openness to Learning_\
  Embrace a mindset of continuous learning and be open to acquiring new knowledge
  and skills. Starknet is a recent ecosystem and does have its own unique concepts
  and principles. Stay curious, ask questions (there are no dumb questions), and
  be willing to explore and understand new concepts.

- _Patience and Perseverance_\
  Contributing to a complex or quickly evolving project takes time and effort. Be
  patient with yourself and the learning process. Expect challenges along the way,
  but persevere through them. Building expertise and making meaningful contributions
  often requires persistence and determination.

- _Collaboration and Communication_\
  Engage with the community, ask for guidance when needed, and seek feedback on your
  contributions. Actively participate in GitHub discussions, issues or the chat channels.
  Be respectful and constructive with other builders.

- _Respect for Existing Contributors_\
  Recognize and respect the work of existing contributors and maintainers. Appreciate
  the efforts of others and collaborate with them in a respectful and inclusive manner.

With this mindset, you'll participate to Madara and Starknet projects in a
collaborative and productive atmosphere. It's not everything about code but also
about being part of a community working towards a shared goal.

> As many contributors, you may have commitments and full-time jobs. Your
> valuable contributions, regardless of their frequency, greatly contribute to
> the progress and collaborative spirit of the Madara's project and Starknet
> ecosystem.

## Learning Rust Programming <a name="learning-rust"/>

Rust is a modern programming language known for its focus on safety and
performance. Rust's key features include a strong static type system, ownership
and borrowing concepts, which allow errors such as null pointer dereferences and
data races to be detected at compile time.

Rust is the programming language used for Madara and Substrate, and here are
some valuable and helpful resources.

[The Rust Programming Language Book](https://doc.rust-lang.org/book/): read this
first\
[Rust by Example](https://doc.rust-lang.org/rust-by-example/): practical approach\
[Rustlings](https://github.com/rust-lang/rustlings): Educative and interactive
learning

## Understanding Blockchain Basics <a name="blockchain-basics"/>

Resources in this section provide an introduction to blockchain technology, its
key concepts, and how it works. It's essential to grasp the fundamentals of
blockchain before diving into specific blockchain development.

Briefly, a blockchain is a decentralized and immutable digital ledger that
records transactions across multiple computers or nodes in a network. It enables
secure and transparent data storage and transaction validation without relying
on a central authority. Each block is linked to the previous one, creating a
chain of blocks that form a tamper-resistant record of data.

A Layer 2 (e.g. Starknet) solution is a scaling technique designed to enhance
the scalability and performance of a blockchain network. It operates "on top" of
an existing blockchain, leveraging its security and decentralization while
improving transaction throughput and reducing fees. Layer 2 solutions address
the scalability limitations of blockchains, enabling them to handle a higher
volume of transactions and improving the overall user experience.

[Blockchain
Explained](https://blockgeeks.com/guides/what-is-blockchain-technology/)\
[Introduction to Blockchain Concepts](https://www.ibm.com/topics/blockchain)\
[How Does Blockchain
Work?](https://www.investopedia.com/terms/b/blockchain.asp)\
[Topics on Dev.to](https://dev.to/t/blockchain)

The following two are a bit more technical, but very fundamental:\
[Ethereum white paper](https://ethereum.org/en/whitepaper/)\
[Bitcoin paper](https://bitcoin.org/bitcoin.pdf)

## What is Starknet <a name="what-is-starknet"/>

Starknet is an innovative Layer 2 scaling solution designed specifically for
Ethereum. It aims to address the scalability challenges of the Ethereum network
by enabling fast and cost-effective execution of decentralized applications
(dApps). Starknet achieves this by utilizing zk-rollup technology, which
aggregates multiple transactions on Starknet chain and submits a single proof to
Ethereum for verification. This approach significantly reduces transaction costs
and increases scalability while maintaining the security and decentralization of
the Ethereum network.

[Starknet getting started](https://www.starknet.io/en/what-is-starknet)\
[Using Starknet with the Starknet book](https://book.starknet.io/)\
[Starknet documentation](https://docs.starknet.io/documentation/)\
[Starknet article](https://medium.com/starkware/exploring-the-use-cases-of-cheap-computation-1ab6254e7895)

## What Substrate is? And why is Madara using it?

<a name="understanding-substrate"/>

Substrate is a Software Development Kit (SDK) that allows you to build
application-specific blockchains that can run as standalone services or in
parallel with other chains with the shared security provided by the Polkadot
ecosystem. The SDK is designed to be fully modular and flexible, giving
developers a high degree of control and creativity over the applications they
build.

Some key features of Substrate are:

- _Modular Framework_\
  Substrate provides a modular framework that allows developers to easily customize
  and configure various components of a blockchain network.

- _Efficiency and Scalability_\
  Substrate leverages advanced techniques such as in its transaction queue management
  to ensure high performance and the ability to handle a large number of transactions.

- _Runtime Upgradability_\
  Substrate allows for seamless runtime upgrades, enabling the introduction of new
  features or bug fixes in a live blockchain network without requiring a hard fork.
  This feature enhances the upgradability and maintainability of the blockchain system.

Substrate achieves its modularity through three key components: the **client**,
the **runtime**, and the **primitives**. Those are key concepts of Substrate
_you must understand_ to navigate easily in the Madara code base.

### Client <a name="substrate-client"/>

The client in Substrate refers to the software that interacts with the
blockchain network. It handles network activity such as peer discovery, managing
transaction requests, reaching consensus with peers, and responding to RPC
calls. The client provides the user interface and functionality for interacting
with the blockchain.

As a naming convention, every rust library used for the client implementation is
prefixed with **sc\_** (for Substrate Client, in the Substrate crates). Madara
sticks to this by having **mc\_** prefix.

As an example, [the storage package for Madara's
client](https://github.com/keep-starknet-strange/madara/blob/main/crates/client/storage/Cargo.toml#L3).

### Runtime <a name="substrate-runtime"/>

The runtime determines whether transactions are valid or invalid and is
responsible for handling changes to the blockchain state, it serves as the core
of any blockchain built using Substrate.

It encapsulates the fundamental logic, rules, and functions of the blockchain.
For instance, the processing of transactions and the construction of blocks are
all defined within the runtime. Substrate offers a framework known as FRAME to
supply a modular structure for the runtime, with the modules referred to as
**pallets**. Some common use cases of customizable business logic Pallets can
provide are managing account balances, voting on proposals, staking, and
consensus​​.

As a naming convention, every rust library used for the runtime implementation
is prefixed with **frame\_** or **pallet\_** (in the Substrate crates). Madara
sticks to this by having **pallet\_** prefix.

As an example: [the
pallet_starknet](https://github.com/keep-starknet-strange/madara/blob/main/crates/pallets/starknet/Cargo.toml#L2).

### Primitives <a name="substrate-primitives"/>

At the lowest level of the Substrate architecture, there are primitive libraries
that give control over underlying operations and enable communication between
the core client services and the runtime. The primitive libraries provide the
lowest level of abstraction to expose interfaces that the core client or the
runtime can use to perform operations or interact with each other.

As a naming convention, every rust library used as a primitive is prefixed with
**sp\_** (for Substrate Primitives, in the Substrate crates). Madara sticks to
this using the **mp\_** prefix.

As an example: [the mp_starknet
package](https://github.com/keep-starknet-strange/madara/blob/main/crates/primitives/starknet/Cargo.toml#L2).

---

While Substrate may have a learning curve for newcomers to blockchain
development, it provides a powerful and flexible framework for building
blockchain networks. Its understanding is required to contribute to Madara.

When you explore the Madara codebase, the structure of the previously mentioned
architecture becomes apparent in the organization of the `crates` folder.

```bash
├── client
│   ├── db
│   ├── mapping-sync
│   ├── rpc
│   ├── rpc-core
│   └── storage
├── node
├── pallets
│   └── starknet
├── primitives
│   ├── digest-log
│   └── starknet
│       └── src
│           ├── block
│           ├── crypto
│           │   ├── commitment
│           │   ├── hash
│           │   └── merkle_patricia_tree
│           ├── execution
│           ├── fees
│           ├── starknet_serde
│           ├── state
│           ├── storage
│           ├── tests
│           ├── traits
│           └── transaction
└── runtime
```

Even if Substrate was first designed for Polkadot chain and parachain
development, it's still a very useful framework for creating a state-of-the-art
blockchain, even without connecting it to Polkadot. This is referred to as
"Solo-Chain" in Polkadot terminology.

[Substrate docs](https://docs.substrate.io/)\
[Substrate architecture](https://docs.substrate.io/learn/architecture/)\
[Substrate and
Polkadot](https://medium.com/polkadot-network/a-brief-summary-of-everything-substrate-and-polkadot-f1f21071499d)

For a more practical learning experience, these [Substrate
Tutorials](https://github.com/rusty-crewmates/substrate-tutorials) would be a
great place to start.

## Diving into Madara <a name="madara-dive"/>

Now that you have a better understanding of the context, what is Madara?

Madara is a powerful sequencer for Starknet, an innovative Layer 2 scaling
solution for Ethereum. Madara plays a crucial role in enabling efficient and
scalable transaction processing on the Starknet network. Developed in Rust and
built upon the robust Substrate framework, Madara leverages the potential of
decentralized technologies to enhance the scalability of Ethereum.

A sequencer is a vital component within blockchain layer 2 solutions that plays
a critical role in processing and ordering transactions on the Starknet network.
As the backbone of transaction sequencing, Madara ensures that transactions are
organized and executed in a secure and deterministic manner.

You can find Madara documentation (work in progress)
[here](https://docs.madara.wtf/).\
You can contribute to this documentation [here](https://github.com/keep-starknet-strange/madara-docs).

How to contribute?

1. Head to the [Madara github
   repository](https://github.com/keep-starknet-strange/madara) and fork the
   repository.
2. Search for [issues](https://github.com/keep-starknet-strange/madara/issues)
   with the `good first issue` label.
3. Work on your fork, in a branch dedicated to the issue you are working on.
4. Push your changes to your fork, and submit a pull request (PR) on Madara
   official repository.
5. If change is non trivial and require some time to complete we suggest opening
   a Draft PR. In case you need any help you can ask in [madara's
   telegram](https://t.me/MadaraStarknet) channel and link to the relevant code
   path.

Exciting stuff, right? Join the community of Starknet builders!

Joining the community is crucial for engaging with fellow contributors, seeking
help, and staying up-to-date with Madara developments. Join us in building the
future of Ethereum scaling!

[GitHub contributor
guide](https://docs.github.com/en/get-started/quickstart/hello-world)\
[Madara GitHub repository](https://github.com/keep-starknet-strange/madara)\
[Madara Telegram](https://t.me/MadaraStarknet)\
[Starknet Discord](https://discord.gg/qypnmzkhbc) (Or search for Starknet in discord's
servers browser)

## Contribution rewards on OnlyDust <a name="onlydust"/>

Starkware, which is the company at the root of Starknet innovation, is rewarding
the contributors via [OnlyDust](https://www.onlydust.xyz/). This is an amazing
initiative and opportunity that reward early builders of the Starknet ecoystem.

How it works? Simple:

1. Head to [OnlyDust app](https://app.onlydust.xyz/).
2. Create an account linked to your GitHub profile.
3. Contributions that are taken in account and candidate for rewards are any PR
   you have open, that was merged by a maintainer.
4. It's important to consider that every project has limited funds, and the
   retributions are dispatched by the maintainers, in a very transparent manner.
