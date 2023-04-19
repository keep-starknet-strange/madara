## Description

<a href="http://www.typescriptlang.org" target="_blank">TypeScript</a> type
definitions that can be used to decorate the
<a href="https://www.npmjs.com/package/@polkadot/api" target="_blank">@polkadot/api</a>.

## Installation

```bash
npm i @keep-starknet-strange/madara-api-augment
```

> :warning: `@polkadot/api` should be installed in your project!

## Usage

Add to your codebase entry point before any imports from the API itself.

- `import '@keep-starknet-strange/madara-api-augment'` - applies Madara types
  and endpoint augmentation

## Docs

- [TS type generation]("https://polkadot.js.org/docs/api/examples/promise/typegen/")
- [TypeScript augmentation since 7.x]("https://polkadot.js.org/docs/api/FAQ/#since-upgrading-to-the-7x-series-typescript-augmentation-is-missing")
- [TypeScript interfaces]("https://polkadot.js.org/docs/api/start/typescript")

## Publish

Update package version.

```bash
npm version --no-git-tag-version 0.1500.0
```

Generate new types.

```bash
npm run generate
```

`The version change and new generated types should be merged to master.`

Build the package.

```bash
npm run build
```

`This will build the package and copy necessary files to the build folder.`

```bash
npm run publish
```

`This will publish content of the build folder.`
