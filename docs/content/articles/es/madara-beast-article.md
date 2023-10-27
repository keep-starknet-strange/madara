![thee BEEAAST](https://imgur.com/EBwBNnB.jpg)

# Aprovechando a la Bestia - Madara y la Revolución de las Appchains de Starknet

**20 de julio de 2023** · 1 min de lectura

<font size=5>_Desde Reducciones Masivas de Costos hasta Control Personalizado,
Descubre el Futuro de la Infraestructura Blockchain_</font>

---

## TL;DR

- Madara es un secuenciador de alto rendimiento de Starknet, que proporciona el
  poder para crear
  [Appchains](https://www.starknet.io/en/posts/ecosystem/the-starknet-stacks-growth-spurt)
  personalizadas y eficientes.
- Al utilizar el framework Substrate, Madara amplía las capacidades de la
  Máquina Virtual Cairo, lo que conduce a programas demostrables, seguros y
  flexibles.
- Su implementación ofrece numerosos beneficios como infraestructura escalable,
  alto rendimiento y un control sin precedentes sobre las aplicaciones.
- Las características únicas de Madara incluyen soporte para la privacidad
  potencial _on-chain_, interoperabilidad simplificada entre diversas cadenas y
  una ejecución robusta.
- Madara está abriendo el camino en el desarrollo de dApps al ofrecer soluciones
  rentables, escalables y personalizables en el ámbito de _blockchain_.

## Introducción

Imagina tener el poder de personalizar una _blockchain_ específicamente para los
requisitos únicos de tu aplicación; eso es exactamente lo que ofrecen las
appchains. Las appchains son cadenas de bloques específicas para aplicaciones
que ofrecen a los desarrolladores la flexibilidad para ajustar aspectos de las
cadenas para adaptarlas a las necesidades de sus aplicaciones, como elegir una
función hash diferente o personalizar el algoritmo de consenso. ¿La mejor parte?
Las appchains heredan la seguridad de las robustas cadenas L1 o L2 en las que se
basan, proporcionando a los desarrolladores lo mejor de ambos mundos.

Te presentamos a Madara, un secuenciador revolucionario que combina flexibilidad
y un rendimiento ultrarrápido. Los secuenciadores son entidades responsables de
ejecutar transacciones y agruparlas en lotes. Actuando como una puerta de
entrada para lanzar tu propia appchain Starknet, Madara abre un mundo de
posibilidades para la experimentación en el ecosistema de Starknet como nunca
antes.

Antes de adentrarnos en las fascinantes capacidades de Madara para habilitar las
appchains de Starknet, es importante abordar la pregunta de por qué los
desarrolladores optarían por construir appchains sobre Starknet en lugar de
utilizar los
[Starknet Validity Rollups](https://starkware.co/resource/scaling-ethereum-navigating-the-blockchain-trilemma/#:~:text=top%20of%20them.-,Validity%20Rollups,-Validity%20rollups%2C%20also)
directamente. Uno podría preguntarse si Starknet ya es suficiente para la
mayoría de los escenarios.

Primero, aprendamos por qué las appchains son una extensión convincente del
ecosistema de Starknet.

## Por qué Appchains

Madara, desarrollada por el Equipo de Exploración de StarkWare, también conocido
como [Keep Starknet Strange](https://github.com/keep-starknet-strange), está
diseñada específicamente para realizar la
[visión de escalabilidad fractal](https://medium.com/starkware/fractal-scaling-from-l2-to-l3-7fe238ecfb4f)
de StarkWare. Existen numerosas razones convincentes por las cuales los
desarrolladores podrían optar por establecer una appchain Starknet o L3 en lugar
de depender directamente de Starknet.

### Rendimiento

Los desarrolladores de aplicaciones enfrentan desafíos significativos en
términos de escalabilidad dentro de la infraestructura de _blockchain_
existente. La escalabilidad abarca dos aspectos cruciales: alta velocidad y
bajos costos. Al implementar una reducción de costos de 1,000 veces en cada
capa, los desarrolladores pueden lograr una reducción de costos general notable
de L1 a L3, potencialmente alcanzando hasta 1,000,000 veces. La velocidad de
procesamiento no se ve afectada por la actividad de aplicaciones de terceros, ya
que la aplicación tiene su propia _blockchain_ dedicada y no compite por
recursos. Esto garantiza una experiencia constantemente fluida.

### Personalización

Cadenas de propósito general como Starknet y Ethereum tienen múltiples medidas
para garantizar que la red sea utilizable por todos, lo que lleva a un entorno
limitado. Con las appchains, los desarrolladores pueden ajustar varios aspectos
de sus aplicaciones e infraestructura, creando soluciones a medida. ¿No te gusta
una característica de la Máquina Virtual Cairo? Elimínala en tu appchain.

### Innovación

La capacidad de personalización de las appchains también permite a los
desarrolladores trabajar con características que actualmente no están
disponibles o son riesgosas en entornos como Starknet. Las appchains ofrecerán a
cada equipo la autonomía para escribir y autorizar cualquier pista de código
deseada. Esto permite a las appchains desbloquear muchos casos de uso, como la
capacidad de aplicar KYC _on-chain_ sin divulgar información privada.

## Efecto de Madara en el Stack de Appchains

1. **Ejecución:** La capa de ejecución define la ejecución de bloques y la
   generación de la diferencia de estado. Madara ofrece la flexibilidad para
   cambiar entre dos "crates" de ejecución,
   [Blockifier de StarkWare](https://github.com/starkware-libs/blockifier) y
   [Starknet_in_rust de LambdaClass](https://github.com/lambdaclass/starknet_in_rust).
   Independientemente del "crate" elegido, el framework subyacente utiliza la
   Máquina Virtual Cairo. El lenguaje Cairo facilita la creación de programas
   demostrables, lo que permite la demostración de la ejecución correcta del
   cálculo.
2. **Liquidación:** Como Validity Rollup, el estado de una appchain Madara se
   puede reconstruir únicamente examinando su capa de liquidación. Al liquidar
   más frecuentemente en Starknet L2, una appchain L3 puede lograr una finalidad
   más rápida y descentralizar la capa de secuenciación para lograr una
   finalidad suave más sólida. Por lo tanto, la liquidación se mejora en ambos
   frentes (finalidad dura y suave).
3. **Secuenciación:** Madara se encarga del proceso de secuenciación, que se
   puede alterar para satisfacer las necesidades de la aplicación, ya sea un
   simple FCFS, PGA o esquemas más complejos como Narwhall & Bullshark. Ciertas
   appchains pueden optar por implementar "mempools" encriptados para garantizar
   un orden justo y mitigar el impacto de MEV.
4. **Disponibilidad de Datos:** La disponibilidad de datos garantiza que el
   árbol de estado completo siga siendo accesible, proporcionando a los usuarios
   la confianza de que pueden demostrar la propiedad de sus fondos incluso si
   Madara experimenta una interrupción. Madara ofrecerá a los desarrolladores
   una variedad de esquemas de disponibilidad de datos (DA) para elegir.
5. **Gobernanza:** Cada appchain Madara puede elegir su modelo de gobernanza.
   [Snapshot X](https://twitter.com/SnapshotLabs) ofrece un sistema de
   gobernanza completamente _on-chain_ que se basa en pruebas de almacenamiento.
   También se están explorando mecanismos de gobernanza alternativos, como el
   "governance pallet" nativo de Substrate. La gobernanza _on-chain_ se presenta
   como un valor fundamental para Madara.

![come come](https://lh4.googleusercontent.com/i7bXi2IPV-LTLzEgueA2SPHGULUFDj1OX4IznOQr5BeZe0hcey-VXA5TOV6q9XaVqBGAcYiie7u7uxw7q1ByZxjkPQKHERqKJTxhdDdTSgBQy8smyNO3jEHiNJv7Eqh8BMxjj4fFlQAW6gm-hQMzyIU)

## Entra: Madara

En Madara, la Máquina Virtual Cairo se está mejorando mediante la utilización
del framework Substrate e integrando la Máquina Virtual Cairo para ejecutar
programas Cairo y contratos inteligentes de Starknet. Substrate es un framework
Rust de código abierto para construir cadenas de bloques personalizables, que es
conocido por su flexibilidad. Mientras tanto, la Máquina Virtual Cairo está
diseñada específicamente para generar de manera eficiente Pruebas de Validez
para la ejecución de programas. Al utilizar seguimiento de estado y un contrato
inteligente para verificar estas pruebas en L2, la appchain asegura una
integración segura con Starknet. De esta manera, Madara aprovecha el poder de
Cairo para habilitar la demostración de la ejecución de programas.

La naturaleza modular inherente del framework Substrate permite a los
desarrolladores personalizar la appchain con facilidad. No se imponen
suposiciones, lo que te permite incorporar tu propio protocolo de consenso,
función hash, esquema de firma, distribución de almacenamiento, lo que sea que
tu aplicación requiera, todo mientras utilizas Cairo para generar pruebas. No
hay límites en lo que los desarrolladores pueden hacer mientras siguen siendo
demostrables, heredando la seguridad de la cadena subyacente, ya sea Starknet o
Ethereum.

Inicialmente, Madara tendrá un fuerte parecido a Starknet, lo que permitirá la
composición de contratos inteligentes dentro del ecosistema de Starknet. Hay
planes más grandes en el futuro a medida que Starknet se integra con
[Herodotus](https://www.herodotus.dev/) para aprovechar
[pruebas de almacenamiento](https://starkware.medium.com/what-are-storage-proofs-and-how-can-they-improve-oracles-e0379108720a)
para lograr interoperabilidad. La integración de pruebas de almacenamiento
también permitirá que las appchains de Madara consideren el estado y la liquidez
de otras cadenas.

Prepárate para presenciar un nuevo espacio de posibilidades en el universo de
Starknet, habilitado por Madara.
