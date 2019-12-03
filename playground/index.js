const fs = require('fs');
const assert = require('assert');
const fastXmlParser = require('fast-xml-parser');
const xml2js = require('xml2js');

const quickXml2Js = require('../lib');

const xmlString = fs.readFileSync('./playground/data.xml').toString('utf8');

console.time('Rust quick-xml2js-parser:direct');
const rustDirect = quickXml2Js.directParse(xmlString);
console.timeEnd('Rust quick-xml2js-parser:direct');

console.time('Rust quick-xml2js-parser:indirect');
const rustIndirect = quickXml2Js.indirectParse(xmlString);
console.timeEnd('Rust quick-xml2js-parser:indirect');

console.time('Js fast-xml-parser');
let js = fastXmlParser.parse(xmlString, {
  ignoreAttributes: false,
  attributeNamePrefix: '',
  attrNodeName: '$',
  textNodeName: '_'
});
console.timeEnd('Js fast-xml-parser');

assert.deepStrictEqual(rustIndirect, js, 'Not equal!');
assert.deepStrictEqual(rustDirect, js, 'Not equal!');
assert.deepStrictEqual(rustIndirect, rustDirect, 'Not equal!');

console.time('Js xml2js');
xml2js.parseString(xmlString, (_, result) => {
  if (result) {
    console.timeEnd('Js xml2js');
  }
});

console.log(JSON.stringify(rustDirect, null, 4));
