import "clsx";
import "@sveltejs/kit/internal";
import "../../../chunks/exports.js";
import "../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../chunks/root.js";
import "../../../chunks/state.svelte.js";
/* empty css                                                         */
import "../../../chunks/project.js";
import { M as MobileHeader } from "../../../chunks/MobileHeader.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    MobileHeader($$renderer2, { title: "Quality" });
    $$renderer2.push(`<!----> <div class="quality-page svelte-5zwhru">`);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading-state svelte-5zwhru"><div class="loading-spinner svelte-5zwhru"></div></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
