![thee BEEAAST](https://imgur.com/EBwBNnB.jpg)

# Dominando a Besta - Madara e a Revolução das Appchains da Starknet

**Jul 20,2023** · 1 minuto de leitura

<font size=5>_De Reduções de Custos Massivas ao Controlo Personalizado, Descobre
o Futuro da Insfraestrutura da Blockchain_</font>

---

## TL;DR

- O Madara é um sequencer para a Starknet de alta performance, que oferece o
  poder de criar
  [Appchains](https://www.starknet.io/en/posts/ecosystem/the-starknet-stacks-growth-spurt)
  personalizáveis e eficientes.

- Ao usar a framework Substrate, o Madara amplifica as capacidades do Cairo VM,
  levando a programas comprováveis, seguros e flexíveis.
- A sua implementação oferece beneficios como uma infraestrutura escalável, alto
  rendimento e um controlo sem precedente sobre aplicações.
- As características únicas do Madara incluem suporte para potencial privacidade
  on-chain, interoperabilidade simplificada entre várias chains e uma execução
  robusta.
- O Madara está a construir o caminho no desenvolvimento de dApps ao oferecer
  soluções de baixo custo, escaláveis e personalizáveis no domínio da
  blockchain.

## Introdução

Imagina ter a capacidade para fazer uma blockchain especificamente para os
requisitos únicos da tua aplicação - é exatamente isto que uma appchain oferece.
Appchains são blockchains específicas para uma aplicação que oferecem aos
desenvolvedores flexibilidade de ajustar aspetos das chains para se adequarem às
necessidades das suas aplicações, como escolher uma função de hash diferente ou
personalizar o algoritmo de consenso. A melhor parte? Appchains herdam a
segurança de onde as robustas blockchains L1 e L2 são construídas,
disponibilizando o melhor dos dois mundos para os desenvolvedores.

Introduzimos o Madara, um sequencer inovador que combina flexibilidade com uma
performance ultra-rápida. Sequencers são entidades responsáveis por executar
transações e agrupá-las em lotes. O Madara abre um leque de possibilidades como
nunca para experimentação no ecossistema da Starknet, agindo como uma porta de
entrada para lançar a tua própria appchain na Starknet.

Antes de mergulharmos nas capacidades fascinantes do Madara em habilitar
appchains na Starknet, é importante perguntar o porquê dos desenvolvedores
optarem por construir appchains em cima da Starknet ao invés de utilizar os
[Starknet Validity Rollups](https://starkware.co/resource/scaling-ethereum-navigating-the-blockchain-trilemma/#:~:text=top%20of%20them.-,Validity%20Rollups,-Validity%20rollups%2C%20also)
diretamente. Podemos questionar se a Starknet é suficiente para a maioria dos
cenários.

Primeiro vamos aprender porque é que as appchains são uma extensão atrativa para
o ecossistema da Starknet.

## Porquê Appchains

O Madara, desenvolvido pela StarkWare Exploration Team, também conhecida por
[Keep Starknet Strange](https://github.com/keep-starknet-strange), é desenhado
especificamente para concretizar a
[visão de fractal scaling](https://medium.com/starkware/fractal-scaling-from-l2-to-l3-7fe238ecfb4f)
da Starknet. Existem inúmeras razões interessantes para os desenvolvedores
optarem por criar uma appchain da Starknet ou L3 ao invés de depender
diretamente da Starknet.

### Throughput

Desenvolvedores enfrentam desafios significantes em termos de escalabilidade nas
infraestruturas de blockchain existentes. A escalabilidade abrange dois aspetos
cruciais: alta velocidade e taxas baixas. Ao implementar uma redução de custo de
1,000x em cada camada, desenvolvedores conseguem alcançar uma redução de custo
significativa de L1 até à L3, potencialmente alcançando 1,000,000x. A capacidade
de processamento não é afetada pela atividade de aplicações de terceiros pois a
aplicação tem uma blockchain dedicada e não compete por recursos. Isto garante
uma experiência suave e consistente.

### Personalização

Chains de uso geral como a Starknet e o Ethereum têm múltiplas medidas no que
toca a garantit que a rede é utilizável por todos, levando a um ambiente
constragido. Com appchains, desenvolvedores podem afinar vários aspetos das suas
aplicações e infraestruturas, criando soluções à medida. Não gostas de uma
funcionalidade da Cairo VM? Elimina-a da tua appchain.

### Inovação

A personalização de appchains permite aos desenvolvedores trabalharem com
funcionalidades que estão indisponíveis ou arriscadas em ambientes como a
Starknet. Appchains oferecem a cada equipa a autonomia para escrever e autorizar
quaisquer dicas de código desejadas. Isto permite às appchains desbloquearem
muitos casos de uso, como ser capaz de reforçar on-chain KYC sem divulgar
informação privada.

## Efeito do Madara no Stack para Appchain

1. **Execução:** A camada de execução define a execução dos blocos e geração de
   diferença de estado. O Madara oferece a flexibilidade para mudar entre dois
   crates de execução,
   [Blockifier da StarkWare](https://github.com/starkware-libs/blockifier) e
   [Starknet_in_rust da LambdaClass](https://github.com/lambdaclass/starknet_in_rust).
   Independente do crate escolhido, a estrutura subjacente utiliza a Cairo VM. A
   linguagem Cairo facilita a criação de programas comprováveis, permitindo a
   demonstração da execução correta do cálculo.
2. **Settlement:** Como um validity rollup, o estado da appchain do Madara pode
   ser reconstruído exclusivamente ao examinar a sua settlement layer. Ao fazer
   um settlement mais frequentemente na Starknet L2, uma appchain L3 consegue
   alcançar uma finalidade mais rápida, enquanto a descentralização da camada de
   sequenciamento permite uma finalidade mais robusta e suave. Assim, o
   settlement é reforçado em ambas as frentes (finalidade dura e suave).
3. **Sequencer:** O Madara assume o controlo do processo de sequenciação, que
   pode ser alterado em função das necessidades da aplicação - seja o simples
   FCFS, PGA ou esquemas mais complexos como Narwhall & Bullshark. Certas
   appchains podem escolher implementar mempools encriptadas para garantir uma
   ordenação justa e mitigar o impacto do MEV.
4. **Disponibilidade de Dados:** A Disponibilidade de dados garante o acesso à
   árvore de estados completa, oferecendo aos utilizadores a confiança de que
   podem provar a posse dos seus fundos mesmo se o Madara sofrer uma
   interrupção. O Madara irá oferecer aos desenvolvedores vários esquemas de
   disponibilidade de dados para escolherem.
5. **Governança:** Cada appchain do Madara pode escolher o seu modelo de
   governança. [Snapshot X](https://twitter.com/SnapshotLabs) oferece um sistema
   de governança totalmente on-chain que depende de provas de armazenamento.
   Mecanismos de governança alternativos também estão sob exploração, como o
   native substrate governance pallet. Governança on-chain é um valor
   fundamental para o Madara.

![come come](https://lh4.googleusercontent.com/i7bXi2IPV-LTLzEgueA2SPHGULUFDj1OX4IznOQr5BeZe0hcey-VXA5TOV6q9XaVqBGAcYiie7u7uxw7q1ByZxjkPQKHERqKJTxhdDdTSgBQy8smyNO3jEHiNJv7Eqh8BMxjj4fFlQAW6gm-hQMzyIU)

## Enter: Madara

No Madara, a Cairo VM está a ser aprimorada utilizando a framework Substrate e
integrando a Cairo VM para executar programas Cairo e smart contracts da
Starknet. O Substrate é uma framework em Rust open-source para construir
blockchains personalizáveis, que é conhecida pela sua flexibilidade. Enquanto
isso, a Cairo VM é desenhada especificamente para gerar Provas de Validade
eficientemente a execução de programas. Ao implementar rastreamento de estado e
um smart contract para verificar essas provas na L2, a appchain garante uma
integração segura com a Starknet. Desta maneira, o Madara aproveita o poder do
Cairo para permitir a comprovação da execução do programa.

A natureza modular inerente da framework Substrate permite ao desenvolvedores
personalizar a appchain com facilidade. Nenhuma suposição é imposta, permitindo
a incorporação dos teus protocolos de consenso, funções hash, esquema de
assinatura, layout de armazenamento – independente das necessidades da
aplicação, utilizando o Cairo para gerar provas. Sem limites no que os
desenvolvedores podem fazer enquanto ainda são comprováveis, herdando a
segurança da chain - seja Starknet ou Ethereum.

Inicialmente, o Madara terá uma forte semelhança com a Starknet, permitindo a
capacidade de composição de smart contracts no ecossistema da Starknet. Existem
planos maiores para o futuro à medida que a Starknet se integra com o
[Herodotus](https://www.herodotus.dev/) para alavancar
[provas de armazenamento](https://starkware.medium.com/what-are-storage-proofs-and-how-can-they-improve-oracles-e0379108720a)
para alcançar interoperabilidade. A integração de provas de armazenamento também
permitirá às appchains do Madara considerarem o estado e liquidez de outras
chains.

Prepara-te para testemunhar um novo espaço de possibilidades no universo
Starklings, fornecido pelo Madara.
