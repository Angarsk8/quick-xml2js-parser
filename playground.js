const quickXml2Js = require('./lib');

const xmlString = `
<items>
  <item id="0001" type="donut">
    <name>Cake</name>
    <ppu>0.55</ppu>
    <batters>
      <batter id="1001">Regular</batter>
      <batter id="1002">Chocolate</batter>
      <batter id="1003">Blueberry</batter>
    </batters>
    <topping id="5001">None</topping>
    <topping id="5002">Glazed</topping>
    <topping id="5005">Sugar</topping>
    <topping id="5006">Sprinkles</topping>
    <topping id="5003">Chocolate</topping>
    <topping id="5004">Maple</topping>
  </item>
  <item id="0002" type="brownie">
    <name>Cake</name>
    <ppu>0.8</ppu>
    <batters>
      <batter id="1001">Regular</batter>
      <batter id="1002">Chocolate</batter>
    </batters>
    <topping id="5001">None</topping>
    <topping id="5005">Sugar</topping>
    <topping id="5003">Chocolate</topping>
    <topping id="5004">Maple</topping>
  </item>
</items>
`;

const object = quickXml2Js.directParse(xmlString);

console.log(JSON.stringify(object, null, 4));
