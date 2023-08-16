![thee BEEAAST](https://imgur.com/EBwBNnB.jpg)

# Harnessing the Beast - Madara and the Revolution of Starknet Appchains

**Jul 20,2023** · 1 min read

<font size=5>_From Massive Cost Reductions to Personalized Control, Discover the
Future of Blockchain Infrastructure_</font>

---

## TL;DR

- Madara is a high-performance Starknet sequencer, providing the power to create
  customizable and efficient
  [Appchains](https://www.starknet.io/en/posts/ecosystem/the-starknet-stacks-growth-spurt).
- By using the Substrate framework, Madara amplifies the capabilities of the
  Cairo VM, leading to provable, secure, and flexible programs.
- Its implementation offers numerous benefits such as scalable infrastructure,
  high throughput, and unprecedented control over applications.
- Unique features of Madara include support for potential on-chain privacy,
  streamlined interoperability across various chains, and robust execution.
- Madara is paving the way in dApp development by delivering cost-effective,
  scalable, and customizable solutions in the blockchain domain.

## Introduction

Imagine having the power to tailor-make a blockchain specifically for your
application’s unique requirements – that’s exactly what appchains offer.
Appchains are application-specific blockchains that offer developers the
flexibility to fine-tune aspects of the chains to suit their applications’
needs, like choosing a different hash function or customizing the consensus
algorithm. The best part? Appchains inherit the security of the robust L1 or L2
blockchains on which they are built, providing developers with the best of both
worlds.

Introducing Madara, a game-changing sequencer that combines flexibility and
lightning-fast performance. Sequencers are entities responsible for executing
transactions and grouping them into batches. Acting as a gateway to launching
your very own Starknet appchain, Madara opens up a realm of possibilities for
experimentation in the Starknet ecosystem like never before.

Before we delve into the fascinating capabilities of Madara in enabling Starknet
appchains, it’s important to address the question of why developers would opt to
build appchains on top of Starknet rather than utilizing the
[Starknet Validity Rollups](https://starkware.co/resource/scaling-ethereum-navigating-the-blockchain-trilemma/#:~:text=top%20of%20them.-,Validity%20Rollups,-Validity%20rollups%2C%20also)
directly. One might wonder if Starknet is already sufficient for most scenarios.

Let’s first learn why appchains are a compelling extension to the Starknet
ecosystem.

## Why Appchains

Madara, developed by the StarkWare Exploration Team, also known as
[Keep Starknet Strange](https://github.com/keep-starknet-strange), is
specifically designed to realize StarkWare 's fractal scaling vision
[fractal Scaling vision](https://medium.com/starkware/fractal-scaling-from-l2-to-l3-7fe238ecfb4f).
There are numerous compelling reasons why developers might choose to establish a
Starknet appchain or L3 instead of directly relying on Starknet.

### Throughput

App developers face significant challenges in terms of scalability within the
existing blockchain infrastructure. Scalability encompasses two crucial aspects:
high speed and low fees. By implementing a 1,000x cost reduction at each layer,
developers can achieve a remarkable overall cost reduction from L1 to L3,
potentially reaching up to 1,000,000x. The throughput remains unaffected by the
activity of third-party applications as the app has a dedicated blockchain and
is not competing for resources. This ensures a consistently smooth experience.

### Customization

General-purpose chains like Starknet and Ethereum have multiple measures in
place to ensure the network is usable by everyone, leading to a constrained
environment. With appchains, developers can fine-tune various aspects of their
applications and infrastructure, creating tailored solutions. Don’t like a
feature of the Cairo VM? Eliminate it in your appchain.

### Innovation

The customizability of appchains also allows developers to work with features
that are currently unavailable or risky in environments like Starknet. Appchains
will offer each team the autonomy to write and authorize any desired code hints.
This allows appchains to unlock many use cases, like being able to enforce
on-chain KYC without leaking private information.

## Madara's Effect on the Appchain Stack

1. **Execution:** The execution layer defines the execution of blocks and
   generation of state difference. Madara offers the flexibility to switch
   between two execution crates,
   [Blockifier by StarkWare](https://github.com/starkware-libs/blockifier) and
   [Starknet_in_rust by LambdaClass](https://github.com/lambdaclass/starknet_in_rust).
   Regardless of the crate chosen, the underlying framework utilizes the Cairo
   VM. The Cairo language facilitates the creation of provable programs,
   enabling the demonstration of correct computation execution.
2. **Settlement:** As a Validity Rollup, a Madara appchain's state can be
   reconstructed solely by examining its settlement layer. By settling more
   frequently on Starknet L2, an L3 appchain can achieve faster hard finality,
   while decentralizing the sequencing layer enables more robust soft finality.
   Hence, settlement is enhanced on both fronts (hard and soft finality).
3. **Sequencing:** Madara takes charge of the sequencing process, which can be
   altered to suit the application’s needs – be it simple FCFS, PGA or more
   complex schemes like Narwhall & Bullshark. Certain appchains can choose to
   deploy encrypted mempools to ensure fair ordering and mitigate the impact of
   MEV.
4. **Data Availability:** Data availability guarantees that the complete state
   tree remains accessible, providing users with the confidence that they can
   prove ownership of their funds even if Madara experiences a disruption.
   Madara will offer developers a range of data availability (DA) schemes to
   choose from.
5. **Governance:** Each Madara appchain can choose its governance model.
   [Snapshot X](https://twitter.com/SnapshotLabs) offers a fully on-chain
   governance system that relies on storage proofs. Alternative governance
   mechanisms are also under exploration, such as the native substrate
   governance pallet. On-chain governance stands as a core value for Madara.

![come come](https://lh4.googleusercontent.com/i7bXi2IPV-LTLzEgueA2SPHGULUFDj1OX4IznOQr5BeZe0hcey-VXA5TOV6q9XaVqBGAcYiie7u7uxw7q1ByZxjkPQKHERqKJTxhdDdTSgBQy8smyNO3jEHiNJv7Eqh8BMxjj4fFlQAW6gm-hQMzyIU)

## Enter: Madara

In Madara, the Cairo VM is being enhanced by utilizing the Substrate framework
and integrating the Cairo VM for executing Cairo programs and Starknet smart
contracts. Substrate is an open-source Rust framework to build customizable
blockchains, that is known for its flexibility. Meanwhile, the Cairo VM is
specifically designed to efficiently generate Validity Proofs for program
execution. By employing state tracking and a smart contract to verify these
proofs on L2, appchain ensures secure integration with Starknet. This way,
Madara leverages Cairo’s power to enable the provability of program execution.

The Substrate framework’s inherent modular nature lets developers customize the
appchain with ease. No assumptions are imposed, allowing you to incorporate your
own consensus protocol, hash function, signature scheme, storage layout –
whatever your app requires, all while utilizing Cairo to generate proofs. No
limits on what developers can do while still being provable, inheriting the
security of the underlying chain – be it Starknet or Ethereum.

Initially, Madara will bear a strong resemblance to Starknet, enabling the
composability of smart contracts within the Starknet ecosystem. There are bigger
plans in store for the future as Starknet integrates with
[Herodotus](https://www.herodotus.dev/) to leverage
[storage proofs](https://starkware.medium.com/what-are-storage-proofs-and-how-can-they-improve-oracles-e0379108720a)
to achieve interoperability. The integration of storage proofs will also let
Madara appchains consider state and liquidity from other chains.

Prepare to witness a new space of possibilities in the Starklings universe,
enabled by Madara.
