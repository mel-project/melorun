# Melorun: Melodeon multitool

## Installation

Install with `cargo`:

```
cargo install --locked melorun
```

## Usage

`melorun -h` gives usage information:

```
melorun -h
melorun 0.7.2

USAGE:
    melorun [FLAGS] [OPTIONS] [input]

FLAGS:
    -c, --compile        Compiles the program dumps to stdout the hash and hex-encoded MelVM bytecode
    -h, --help           Prints help information
    -i, --interactive    Starts the interactive REPL
    -V, --version        Prints version information

OPTIONS:
    -s, --spend-ctx <spend-ctx>    An optional spend context YAML file

ARGS:
    <input>    The Melodeon program to run
```

## Spend context

The spend context referred to above is a YAML file that lets you ad-hoc specify certain facts about the environment the covenant is to be tested in, without spinning up a whole actual blockchain.

The format of the YAML file is as follows:

```yaml
# (OPTIONAL) Kind of the spending trasnsaction. Defaults to 0 (Normal)
spender_txkind: 0x00
# (OPTIONAL) Inputs of the spending transaction, other than the coin containing this covenant. Format is a map between integer indices and CoinID structs
spender_other_inputs:
  0:
    txhash: ...
    index: ...
# (MANDATORY) What input index is the covenant-locked coin being spent at? e.g. 0 means that the covenant-locked coin is the first input to the spending transaction
spender_index: 0
# (OPTIONAL) Data field of the spending transaction, in hexadecimal
spender_data: deadbeef
# (OPTIONAL) Outputs of the spending trransaction. A map between integer indices and CoinData structs
spender_outputs:
  0:
    covhash: t1jt0tc9z5zh8j3s7qymvvrd8sh7qq70h9g7q8cn6vcdjyrqm84vcg
    value: 12345 # integer representing microunits
    denom: MEL
    additional_data: deadbeef # hex
# (OPTIONAL) Value of the covenant-locked coin being spent, in microunits. Defaults to 0
parent_value: 123
# (OPTIONAL) Denomination of the coin being spent. Defaults to MEL
parent_denom: MEL
# (OPTIONAL) Additional data of the coin being spent, in hexadecimal. Defaults to the empty string.
additional_data: "cafebabed00d"
# Private keys (in hex) that the signed transaction is signed by. A mapping between the index of the signature and the private key. For example the map {0: K} indicates that the first signature is signed by the private key K.
ed25519_signers:
  0: ...
  1: ...
```
