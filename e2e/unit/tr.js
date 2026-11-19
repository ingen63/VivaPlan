// #170: The scripts of the user interface take their texts from the text table (`tr`, in the browser from i18n.js).
// The unit tests run without a browser and use the German table directly.
const de = require("../../crates/app/ui/i18n/de.json");
global.tr = (key, params = {}) => {
  if (!(key in de)) throw new Error(`unknown key ${key}`);
  return de[key].replace(/\{(\w+)\}/g, (m, k) => (k in params ? String(params[k]) : m));
};
module.exports = { de };
