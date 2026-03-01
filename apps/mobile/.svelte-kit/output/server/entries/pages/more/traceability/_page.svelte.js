import { d as attr } from "../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/exports.js";
import "../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/state.svelte.js";
/* empty css                                                            */
import "../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../chunks/MobileHeader.js";
import { S as Search } from "../../../../chunks/search.js";
import { N as Network } from "../../../../chunks/network.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let searchId = "";
    let loading = false;
    MobileHeader($$renderer2, { title: "Traceability", backHref: "/more" });
    $$renderer2.push(`<!----> <div class="trace-page svelte-1qp37ak"><div class="search-section svelte-1qp37ak"><div class="search-bar svelte-1qp37ak">`);
    Search($$renderer2, { size: 16, class: "search-icon" });
    $$renderer2.push(`<!----> <input type="text" placeholder="Enter entity ID (e.g. REQ-01HQ3K...)"${attr("value", searchId)} class="search-input svelte-1qp37ak"/></div> <button class="trace-btn svelte-1qp37ak"${attr("disabled", loading, true)}>`);
    {
      $$renderer2.push("<!--[!-->");
      Network($$renderer2, { size: 16 });
    }
    $$renderer2.push(`<!--]--> <span>Trace</span></button></div> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="empty-state svelte-1qp37ak">`);
      Network($$renderer2, { size: 48, strokeWidth: 1 });
      $$renderer2.push(`<!----> <p class="empty-title svelte-1qp37ak">Trace Entity Links</p> <p class="empty-desc svelte-1qp37ak">Enter an entity ID above to trace its dependencies and relationships through the project.</p></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
