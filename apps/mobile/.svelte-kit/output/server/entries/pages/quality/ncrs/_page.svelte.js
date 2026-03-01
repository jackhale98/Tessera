import { d as attr, e as ensure_array_like, f as attr_class, g as escape_html } from "../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/exports.js";
import "../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/state.svelte.js";
/* empty css                                                            */
import "../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../chunks/MobileHeader.js";
import "../../../../chunks/EntityCard.svelte_svelte_type_style_lang.js";
import "clsx";
import { S as Search } from "../../../../chunks/search.js";
import { P as Plus } from "../../../../chunks/plus.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let search = "";
    let activeFilter = "all";
    const filters = [
      { id: "all", label: "All" },
      { id: "open", label: "Open" },
      { id: "critical", label: "Critical" }
    ];
    MobileHeader($$renderer2, { title: "NCRs", backHref: "/quality" });
    $$renderer2.push(`<!----> <div class="page svelte-efubw7"><div class="search-bar svelte-efubw7">`);
    Search($$renderer2, { size: 16, class: "search-icon" });
    $$renderer2.push(`<!----> <input type="text" placeholder="Search NCRs..."${attr("value", search)} class="search-input svelte-efubw7"/></div> <div class="filter-chips no-scrollbar svelte-efubw7"><!--[-->`);
    const each_array = ensure_array_like(filters);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let filter = each_array[$$index];
      $$renderer2.push(`<button${attr_class("chip svelte-efubw7", void 0, { "active": activeFilter === filter.id })}>${escape_html(filter.label)}</button>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading-state svelte-efubw7"><div class="loading-spinner svelte-efubw7"></div> <p>Loading NCRs...</p></div>`);
    }
    $$renderer2.push(`<!--]--></div> <a href="/quality/ncrs/new" class="fab svelte-efubw7" aria-label="Create NCR">`);
    Plus($$renderer2, { size: 24 });
    $$renderer2.push(`<!----></a>`);
  });
}
export {
  _page as default
};
