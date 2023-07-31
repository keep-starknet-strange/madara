![thee BEEAAST](https://imgur.com/EBwBNnB.jpg)

# Aproveitando a Besta - Madara e a Revolução das Starknet Appchains

**Jul 20,2023** · 1 min read

<font size=5>_De Grande Reduções de Custos a Controlo Personalizado, Descobre o Futuro da Insfraestrutura de Blockchain_</font>

---

## TL;DR

- Madara é um Starknet sequencer de alta performance, que oferece o poder de criar 
    [Appchains](https://www.starknet.io/en/posts/ecosystem/the-starknet-stacks-growth-spurt) customizáveis e eficientes.

- Ao usar a framework Substrate, o Madara amplifica as capacidades do Cairo VM, levando a programas provados, seguros e flexíveis.
- A sua implementação oferece inúmeros beneficios como uma infraestrutura escalável, alto rendimento e um controlo sem precedente sobre aplicações.
- As características únicas do Madara incluem suporte para potencial privacidade on-chain,
inoperabilidade simplificada entre várias chains e uma execução robusta.
- Madara está a construir o caminho em desenvolvimento dApp ao oferecer soluções custo benefício, escaláveis e customizáveis no domínio da blockchain. 

## Introdução

Imagina ter a capacidade para fazer uma blockchain especificamente para os requerimentos únicos da tua aplicação - é exatamente isto que uma appchain oferece.
Appchains são blockchains especificas para uma aplicação que oferecem aos desenvolvedores flexibilidade para melhorar aspetos das chains para encaixar nas necessidades das suas aplicações, como escolher uma função de hash diferente ou customizar o algoritmo de consenso. A melhor parte? Appchains herdam a segurança de onde as robustas blockchains L1 e L2 são construídas, disponibilizando o melhor dos dois mundos para os desenvolvedores.

Introduzimos o Madara, um sequencer inovador que combina flexibilidade com uma performance ultra-rápida. Sequencers são entidades responsáveis por executar transações e agrupá-las em lotes.
Madara abre um leque de possibilidades como nunca para experimentação no ecossistema Starknet, agindo como uma porta de entrada para lançar a tua própria Starknet appchain.

Antes de mergulharmos nas capacidades fascinantes do Madara em habilitar Starknet appchains, é importante perguntar o porquê os desenvolvedores possam optar por construir appchains em cima da Starknet ao invés de utilizar o [Starknet Validity Rollups](https://starkware.co/resource/scaling-ethereum-navigating-the-blockchain-trilemma/#:~:text=top%20of%20them.-,Validity%20Rollups,-Validity%20rollups%2C%20also)
diretamente. Alguém poderá questionar se a Starknet é suficiente para a maioria dos paradigmas.

Primeiro vamos aprender porque é que as appchains são uma extensão atrativa para o ecossistema Starknet.

## Porquê Appchains

Madara, desenvolvido pela StarkWare Exploration Team, também conhecida por
[Keep Starknet Strange](https://github.com/keep-starknet-strange), é desenhada especificamente para perceber a [visão fractal scaling](https://medium.com/starkware/fractal-scaling-from-l2-to-l3-7fe238ecfb4f) da Starknet.
Existem inúmeras razões interessantes para os desenvolvedores escolherem em estabelecer uma appchain da Starknet ou L3 ao invés de depender diretamente da Starknet. 

### Escalabilidade

Desenvolvedores enfrentam desafios significantes em termos de escalabilidade nas infraestruturas de blockchain existentes. Escalabilidade abrange dois aspetos cruciais: alta velocidade e taxas baixas. Ao implementar uma redução de custo de 1,000x em cada camada, desenvolvedores conseguem alcançar uma redução de custo significativa de L1 até L3, potencialmente alcançando 1,000,000x. A capacidade de processamento não é afetada pela atividade de aplicações de terceiros pois a aplicação tem uma blockchain dedicada e não compete por recursos. Isto garante uma experiência suave e consistente.

### Customização

Chains de uso geral como Starknet e Ethereum tẽm múltiplas medidas no que toca a garantit que a rede é utilizável por todos, levando a um ambiente constragido. Com appchains, desenvolvedores podem afinar vários aspetos das suas aplicações e infraestruturas, criando soluções à medida. Não gostas de um recurso do Cairo VM? Elimina-o da tua appchain.

### Inovação

A personalização de appchains permite aos desenvolvedores trabalhar com recursos que estão indisponíveis ou arriscados atualmente em ambientes como o Starknet. Appchains oferecem a cada equipa a autonomia para escrever e autorizar quaisquer dicas de código desejadas. Isto permite appchains desbloquearem muitos casos de uso, como ser capaz de reforçar on-chain KYC sem vazar informação privada.

## Efeito do Madara na Pilha da Appchain

1. **Execução:** A camada de execução define a execução dos blocos e geração de diferença de estado. Madara oferece a flexibilidade para mudar entre dois caixotes de execução, [Blockifier por StarkWare](https://github.com/starkware-libs/blockifier) e [Starknet_in_rust por LambdaClass](https://github.com/lambdaclass/starknet_in_rust).
Independente da crate escolhida, a estrutura subjacente utiliza a Cairo VM. A linguagem Cairo facilita a criação de programas prováveis, permitindo a demonstração da execução correta do cálculo.
2. **Settlement:** Como um settlement rollup, o estado da appchain do Madara pode ser reconstruído exclusivamente ao       examinar a sua settlement layer. Ao se estabelecer mais frequentemente na Starknet L2, uma appchain L3 consegue alcançar uma     finalidade dura mais rápido, enquanto a descentralização da camada de sequenciamento permite uma finalidade mais     robusta e suave. Assim, a settlement é reforçada em ambas as frentes (finalidade dura e suave).
3. **Sequenciamento:** O Madara assume o controlo do processo de sequenciação, que pode ser alterado em função das necessidades da aplicação - seja o simples FCFS, PGA ou esquemas mais complexos como Narwhall & Bullshark. Certas appchains podem escolher implementar mempools encriptados para garantir uma ordenação justa e mitigar o impacto do MEV.
4. **Disponibilidade de Dados:**
   A Disponibilidade de dados garante o acesso à árvore de estados completa, oferecendo aos utilizadores a confiança de que podem provar a posse dos seus fundos mesmo se o Madara sofrer uma interrupção.
   Madara irá oferecer aos desenvolvedores uma data de esquemas de disponibilidade de dados para escolherem.
5. **Controlo:** 
   Cada appchain do Madara pode escolher o seu modelo de controlo.
   [Snapshot X](https://twitter.com/SnapshotLabs) oferece um sistema de controlo totalmente on-chain que depende de provas de armazenamento. Mecanismos de controlo alternativos também estão sob exploração, como o native substrate
   governance pallet. Controlo on-chain é um valor fundamental para o Madara.

![come come](https://lh4.googleusercontent.com/i7bXi2IPV-LTLzEgueA2SPHGULUFDj1OX4IznOQr5BeZe0hcey-VXA5TOV6q9XaVqBGAcYiie7u7uxw7q1ByZxjkPQKHERqKJTxhdDdTSgBQy8smyNO3jEHiNJv7Eqh8BMxjj4fFlQAW6gm-hQMzyIU)

## Enter: Madara

No Madara, a Cairo VM está a ser aprimorada utilizando a framework Substrate e integrando a Cairo VM para executar programas Cairo e smart contracts da Starknet. Substrate é uma framework do Rust open-source para construir blockchains personalizáveis, que é conhecida pela sua flexibilidade. Enquanto isso, a Cairo VM é desenhada especificamente para gerar Provas de Validade eficientemente para execução de programas. Ao implementar rastreamento de estado e um smart contract para verificar essas provas na L2, a appchain garante uma integração segura com a Starknet. Desta maneira, o Madara aproveita o poder do Cairo para permitir a comprovação da execução do programa.  

A natureza modular inerente da framework Substrate permite ao desenvolvedores personalizar a appchain com facilidade. Nenhuma suposição é imposta, permitindo que você incorpore seus
protocolo de consenso próprio, funções hash, esquema de assinatura, layout de armazenamento –
o que quer que seu aplicativo exija, enquanto utiliza o Cairo para gerar provas. Sem limites no que os desenvolvedores podem fazer enquanto ainda são demonstráveis, herdando o
segurança da cadeia subjacente - seja Starknet ou Ethereum.

Inicialmente, Madara terá uma forte semelhança com o Starknet, permitindo a capacidade de composição de smart contracts no ecossistema da Starknet. Encontram-se guardados planos maiores para o futuro à medida que a Starknet se integra com o  [Herodotus](https://www.herodotus.dev/) para alavancar [provas de armazenamento](https://starkware.medium.com/what-are-storage-proofs-and-how-can-they-improve-oracles-e0379108720a) para alcançar interoperabilidade. A integração de provas de armazenamento também permitirá às appchains do Madara considerarem o estado e liquidez de outras chains.

Prepara-te para testemunhar um novo espaço de possibilidades no universo Starklings, fornecido pelo Madara.
