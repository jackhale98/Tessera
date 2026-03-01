import "clsx";
import "@sveltejs/kit/internal";
import "../../../../../chunks/exports.js";
import "../../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../../chunks/root.js";
import "../../../../../chunks/state.svelte.js";
/* empty css                                                               */
import "../../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../../chunks/MobileHeader.js";
import "../../../../../chunks/EntityCard.svelte_svelte_type_style_lang.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    MobileHeader($$renderer2, { title: "Loading...", backHref: "/quality/ncrs" });
    $$renderer2.push(`<!----> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading-container svelte-11cv3z"><div class="loading-spinner svelte-11cv3z"></div></div>`);
    }
    $$renderer2.push(`<!--]-->`);
  });
}
export {
  _page as default
};
