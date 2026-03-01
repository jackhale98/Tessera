

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.ygMzXKPD.js","_app/immutable/chunks/94XHwtQv.js","_app/immutable/chunks/DTvea1KV.js","_app/immutable/chunks/Bjo0Zqw0.js","_app/immutable/chunks/TMoQNRJs.js","_app/immutable/chunks/yeE0pw05.js","_app/immutable/chunks/hh8xNIHH.js","_app/immutable/chunks/D5RceYED.js","_app/immutable/chunks/QXPDUHtK.js","_app/immutable/chunks/aZlnXb7K.js","_app/immutable/chunks/BTgxoEJy.js","_app/immutable/chunks/CYrVXQwc.js"];
export const stylesheets = ["_app/immutable/assets/MobileHeader.D3XcAB95.css","_app/immutable/assets/0.D69GAwD3.css"];
export const fonts = [];
