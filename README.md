# What is this

A first attempt at a CosmWasm smart contract. It's nothing special, just the implementation of a simple option from the CosmWasm website.

I wanted to try out contract upload and instantiation myself, to see if I understood the process. These are the commands I've used thus far, and the result. 

## Building it

In order to debug things effectively (and as a safety habit), it seems good to get used to running reproducible builds when uploading to the chain. So we use the following Docker command for building:

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.9.0
```

## Simplistic run in console

Start the console:

```
npx @cosmjs/cli@^0.22 --init https://raw.githubusercontent.com/CosmWasm/testnets/master/coralnet/cli_helper.ts 
```

In the console, execute the following commands. 

You'll need to do it line by line, as anything marked `await` won't wait in console mode. 

```
const seed = loadOrCreateMnemonic("fred.key");
const {address: fredAddr, client: client} = await connect(seed, {});

 // wait for this to finish 
let account = await client.getAccount() // make sure we've got funds in the account

// if you don't have enough funds:
hitFaucet(defaultFaucetUrl, fredAddr, "SHELL");

console.log(account.balance) // make sure there's some cash available
const wasm = fs.readFileSync('contract.wasm');

// wait for this to finish 
const up = await client.upload(wasm, { builder: "cosmwasm/rust-optimizer:0.9.0"}); // upload the contract

console.log(up);
const { codeId } = up;
const initMsg = {counter_offer: [{amount: "40", denom: "ushell"}], expires: 2000000};

// wait for this to finish 
const { contractAddress } = await client.instantiate(codeId, initMsg, "Simple option", { memo: "memo", transferAmount: [{denom: "ushell", amount: "5000"}]});

console.log(contractAddress);

client.getContract(contractAddress)
client.getAccount(contractAddress)

const key = new Uint8Array([0, 6, ...toAscii("config")]);

// wait for this to finish 
const raw = await client.queryContractRaw(contractAddress, key);
JSON.parse(fromUtf8(raw))


// This works, but you need to instantiate a contract with expire quite soon in the future (use `client.getHeight()`) before you try it
const burn = {burn:{}};
client.execute(contractAddress, burn);

// To execute on the option, send an `execute` message along with funds to match the option strike price
const bid = {execute: {}};
client.execute(contractAddress, {execute:{}}, "memo",  [{denom: "ushell", amount: "40"}]);
```

