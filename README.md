# Quick XML to JS Parser

## WTH is this?

It's an XML to JSON parser written entirely in Rust, using the [Neon rust-to-js bindings](https://neon-bindings.com/). It's just a silly project of mine that I started because **1)** I've been learning Rust for the past few days and wanted to do something interesting with it **2)** since I'm most of the time coding in JavaScript, I thought it would make sense to learn how to write native modules for NodeJs using Rust (for whenever needed the extra boost in performance) **3)** I'm started to get interested into systems level programming (with Rust at the moment) **4)** why not?

Again, this is just a silly learning project of mine...

## More details...

I ended up implementing the parser two times, just because I didn't know how to tackle the problem initially, so I explored one posible solution and from there iterated into the second: as result there are two exported named functions, which are documented below:

### indirectParse: `(xmlString: string) => Object`

Initially I literally had no idea how to implement the parsing logic, so after thinking for a while I decided to simplify the problem by converting the xml string to a Rust struct first, so that I could represent the input as something easier to inspect and debug. For this first pass the process ended up looking like this:

`String -> Rust struct -> JsObject`.

The struct on the Rust side has the following signature:

```rust
struct Node {
  name: String,
  attributes: Vec<(String, String)>,
  content: Primitive,
  children: HashMap<String, ChildValue>,
}

enum Primitive {
  Null,
  Float(f64),
  Integer(i32),
  Boolean(bool),
  String(String),
}

enum ChildValue {
  Object(Node),
  Array(Vec<Node>),
}
```

See full implementation in the [xml_node](./native/src/xml_node/mod.rs) module which is used in [xml_to_struct_to_js_object](./native/src/xml_to_struct_to_js_object/mod.rs).

#### Usage

```js
const assert = require('assert');
const quickXml2Js = require('./lib');

const obj = quickXml2Js.indirectParse('<foo hello="world"><bar>Foo Bar</bar></foo>');

assert.deepStrictEqual(obj, {
  foo: {
    $: {
      hello: 'world'
    },
    bar: 'Foo Bar'
  }
});
```

### directParse: `(xmlString: string) => Object`

After doing the first pass described before and getting more experience with the problem, I decided to get rid of the intermediary step and aimed for a direct `string` to `object` translation, by using just the Neon native types. The algorithm that I used to solve this problem is pretty much the same that I used in the first implementation, basically, I create a temporary traversal stack, which is increased with every new xml tag that is parsed and then decreased whenever that tag is closed, and apply the accumulated object into its parent (while keeping track of things such as arrays of tags, empty tags, etc).

See full implementation in the [xml_to_js_object](./native/src/xml_to_js_object/mod.rs) module.

#### Usage (Same)

```js
const assert = require('assert');
const quickXml2Js = require('./lib');

const obj = quickXml2Js.directParse('<foo hello="world"><bar>Foo Bar</bar></foo>');

assert.deepStrictEqual(obj, {
  foo: {
    $: {
      hello: 'world'
    },
    bar: 'Foo Bar'
  }
});
```

### Wanna play with it?

#### Setup

```bash
npm i # this will install the dependencies as well as build the project, generating a native module in native/index.node
```

Open [playground.js](./playground.js), play with it and that's it.

### Testing?

Null at the moment, I'll try to add some later...

### Benchmarks?

Do your own 😄
