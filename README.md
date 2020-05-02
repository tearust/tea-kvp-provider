# Tea Project Key-Value Pair Provider Supporting Binary and Sorted Vector
This WASCC provider is an enhanced version of Kevin Hoffman's original [Key-Value Pair Provider example](https://github.com/wascc/examples/tree/master/keyvalue-provider) with the following enhancedments:
- Values are stored in Vec<u8> instead of String
- New added Sorted Vec type. It will sort tuple values by the first element when insert. 

## The Tea Project
Tea Project (Trusted Execution & Attestation) is a Wasm runtime build on top of RoT(Root of Trust) from both trusted hardware environment and blockchain technologies. Developer, Host and Consumer do not have to trust any others to not only protecting privacy but also preventing cyber attacks. The execution environment under remoted attestation can be verified by blockchain consensys. Crypto economy is used as motivation that hosts are willing run trusted computing nodes. This platform can be used by CDN providers, IPFS Nodes or existing cloud providers to enhance existing infrastructure to be more secure and trustless. 

Introduction [blog post](https://medium.com/@pushbar/0-of-n-cover-letter-of-the-trusted-webassembly-runtime-on-ipfs-12a4fd8c4338)

Project [repo](http://github.com/tearust). More and more repo will be exposed soon.

Yet to come project site [( not completed yet) http://www.t-rust.com/](http://www.t-rust.com/)

Contact: kevin.zhang.canada_at_gmail_dot_com.

We are just started, all kinds of help are welcome!

## Motivation
WaSCC Actors are supposed to be stateless. Global variable and any kind of storage across handler calls are not recommended (although technically doable). Host provided key-value pair is one of the handy storage shared across handler functions. 

WaSCC provided Redis provider and a sample key-value pair provider. There are a few reasons I cannot use them:

- Redis is over kill to me.
- Existing key-value pair provider use String, I prefer to use Vec<u8>
- Writing the code direct call from actor is kind of cumbersome. Need an additional actor utility layer in between.

So I made this library to scrach on my own itch. It can probably help you as well.

## Build

Make sure you also clone the tea-codec repo at the same level because it is one of the dependencies.
```
tea-codec = { path = "../tea-codec"}
```
Then
``` 
cargo build
```
For unit testing
```
cargo test
```

## Usage

The same as https://github.com/wascc/examples/tree/master/keyvalue-provider with a few changes
- New CAPABILITY_ID
```
const CAPABILITY_ID: &str = "tea:keyvalue";
```
- New Sorted Vec KeyValueItem
```
pub enum KeyValueItem {
    Atomic(i32),
    Scalar(Vec<u8>),
    List(Vec<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    SortedVec(KeyVec<i32, Vec<u8>>),
}
```
- A few bunch of new functions (omit here)

## Comments are welcome! Happy coding!

