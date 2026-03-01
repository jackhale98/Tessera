import { h as derived, c as store_get, u as unsubscribe_stores } from "../../../../../chunks/root.js";
import { p as page } from "../../../../../chunks/stores.js";
/* empty css                                                               */
import "../../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../../chunks/MobileHeader.js";
import "../../../../../chunks/EntityCard.svelte_svelte_type_style_lang.js";
import "clsx";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let entityId = derived(() => store_get($$store_subs ??= {}, "$page", page).params.id);
    let entityPrefix = derived(() => entityId().split("-")[0]);
    let backHref = derived(() => `/browse/${entityPrefix().toLowerCase()}`);
    MobileHeader($$renderer2, { title: "Loading...", backHref: backHref() });
    $$renderer2.push(`<!----> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading-container svelte-1sc16x7"><div class="loading-spinner svelte-1sc16x7"></div></div>`);
    }
    $$renderer2.push(`<!--]-->`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
