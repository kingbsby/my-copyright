# my-copyright
near demo picture store

## concept

## Getting Started

1、clone this repo
```shell
git clone https://github.com/kingbsby/my-copyright
```

2、run frontend

```shell
cd frontend
yarn install
yarn dev
```

If all goes well, you'll be able to play by accessing address http://localhost:3000/my-copyright in your browser.

> [Option]
> 
> If you want to deploy a new dev contract, you need to do the following steps before run frontend:
> 
> ```
>   cd contract
>   rm -rf neardev
>   cargo build --target wasm32-unknown-unknown --release
>   cp target/wasm32-unknown-unknown/release/my_copyright.wasm res/
>   near dev-deploy res/my_copyright.wasm
>   get the dev account(e.g. dev-1646378813317-27296479436263), Then modify the variables  `ContractName` in file frontend/src/App.js.
>   near call $new-contract-id new --account-id $new-contract-id
> ```
> 
